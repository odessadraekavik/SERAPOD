use std::collections::HashSet;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Mutex;
pub static DETECTION_DETAIL: Mutex<String> = Mutex::new(String::new());
static LAST_GAME_WINDOW: AtomicIsize = AtomicIsize::new(0);
use windows_sys::Win32::{
    Foundation::*, Globalization::*, System::Diagnostics::ToolHelp::*, UI::WindowsAndMessaging::*,
};

pub fn is_game_executable(name: &str) -> bool {
    name.eq_ignore_ascii_case("helldivers2.exe")
}

struct WindowSearch {
    pids: HashSet<u32>,
    window: isize,
    area: i64,
    enumerated: usize,
    matched: usize,
    thread_pid: u32,
}
fn effective_owner(reported_pid: u32, enumerated_thread_pid: u32) -> u32 {
    if reported_pid == 0 { enumerated_thread_pid } else { reported_pid }
}
unsafe extern "system" fn find_window(hwnd: HWND, data: LPARAM) -> i32 {
    let search = &mut *(data as *mut WindowSearch);
    search.enumerated += 1;
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);
    // EnumThreadWindows already establishes ownership through the thread
    // snapshot, even when GetWindowThreadProcessId returns no owner.
    pid = effective_owner(pid, search.thread_pid);
    if search.pids.contains(&pid) {
        search.matched += 1;
    }
    if search.pids.contains(&pid) && IsWindowVisible(hwnd) != 0 {
        let mut rect = RECT::default();
        if GetClientRect(hwnd, &mut rect) == 0 {
            GetWindowRect(hwnd, &mut rect);
        }
        let area = (rect.right - rect.left) as i64 * (rect.bottom - rect.top) as i64;
        if search.window == 0 || area > search.area {
            search.window = hwnd as isize;
            search.area = area;
        }
    }
    1
}
// ToolHelp reads process metadata without opening the protected game process.
pub fn detect_game() -> (bool, isize) {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return (false, 0);
        }
        let mut entry = PROCESSENTRY32W::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut search = WindowSearch {
            pids: HashSet::new(),
            window: 0,
            area: 0,
            enumerated: 0,
            matched: 0,
            thread_pid: 0,
        };
        let mut ok = Process32FirstW(snapshot, &mut entry);
        while ok != 0 {
            let end = entry
                .szExeFile
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(entry.szExeFile.len());
            if is_game_executable(&String::from_utf16_lossy(&entry.szExeFile[..end])) {
                search.pids.insert(entry.th32ProcessID);
            }
            ok = Process32NextW(snapshot, &mut entry);
        }
        CloseHandle(snapshot);
        if !search.pids.is_empty() {
            SetLastError(0);
            let ok = EnumWindows(Some(find_window), &mut search as *mut _ as LPARAM);
            let enum_error = if ok == 0 { GetLastError() } else { 0 };
            let mut thread_count = 0;
            if search.window == 0 {
                let threads = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
                if threads != INVALID_HANDLE_VALUE {
                    let mut entry = THREADENTRY32::default();
                    entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
                    let mut more = Thread32First(threads, &mut entry);
                    while more != 0 {
                        if search.pids.contains(&entry.th32OwnerProcessID) {
                            thread_count += 1;
                            search.thread_pid = entry.th32OwnerProcessID;
                            EnumThreadWindows(entry.th32ThreadID, Some(find_window), &mut search as *mut _ as LPARAM);
                            search.thread_pid = 0;
                        }
                        more = Thread32Next(threads, &mut entry);
                    }
                    CloseHandle(threads);
                }
            }
            // Fullscreen transitions can temporarily omit a game window from
            // enumeration. The foreground handle still has to belong to the
            // exact executable PID; never trust a title match.
            let foreground = GetForegroundWindow();
            let foreground = GetAncestor(foreground, GA_ROOT);
            let mut foreground_pid = 0;
            GetWindowThreadProcessId(foreground, &mut foreground_pid);
            if search.pids.contains(&foreground_pid) {
                find_window(foreground, &mut search as *mut _ as LPARAM);
            }
            if search.window == 0 {
                let previous = LAST_GAME_WINDOW.load(Ordering::SeqCst) as HWND;
                let mut pid = 0;
                GetWindowThreadProcessId(previous, &mut pid);
                if IsWindow(previous) != 0 && search.pids.contains(&pid) {
                    search.window = previous as isize;
                }
            }
            *DETECTION_DETAIL.lock().unwrap()=format!("PID {:?} · windows {} · matches {} · threads {} · foreground PID {} · enumeration {} · error {}",search.pids,search.enumerated,search.matched,thread_count,foreground_pid,ok,enum_error);
        } else {
            DETECTION_DETAIL.lock().unwrap().clear();
        }
        LAST_GAME_WINDOW.store(search.window, Ordering::SeqCst);
        (!search.pids.is_empty(), search.window)
    }
}
pub fn system_locale() -> String {
    unsafe {
        let (mut count, mut size) = (0, 0);
        GetUserPreferredUILanguages(
            MUI_LANGUAGE_NAME,
            &mut count,
            std::ptr::null_mut(),
            &mut size,
        );
        if size > 0 && size < 65536 {
            let mut buffer = vec![0u16; size as usize];
            if GetUserPreferredUILanguages(
                MUI_LANGUAGE_NAME,
                &mut count,
                buffer.as_mut_ptr(),
                &mut size,
            ) != 0
            {
                let end = buffer.iter().position(|c| *c == 0).unwrap_or(buffer.len());
                let locale = String::from_utf16_lossy(&buffer[..end]);
                if !locale.is_empty() {
                    return locale;
                }
            }
        }
        "en".into()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn thread_owner_fallback_requires_known_ownership() {
        assert_eq!(effective_owner(0, 27000), 27000);
        assert_eq!(effective_owner(0, 0), 0);
        // A conflicting reported owner must not be replaced with the game PID.
        assert_eq!(effective_owner(123, 27000), 123);
        assert_eq!(effective_owner(27000, 0), 27000);
    }
    #[test]
    fn only_exact_game_process() {
        assert!(is_game_executable("HELLDIVERS2.EXE"));
        assert!(!is_game_executable("helldivers2.exe.bak"));
        assert!(!is_game_executable("stratcom.exe"));
    }
    #[test]
    fn reads_windows_language() {
        let locale = system_locale();
        assert!(!locale.is_empty());
        assert!(locale.len() < 100);
    }
    #[test]
    #[ignore = "requires a running game; read-only window diagnostics"]
    fn live_game_window_diagnostic() {
        unsafe extern "system" fn inspect(hwnd: HWND, _: LPARAM) -> i32 {
            let mut title = [0u16; 512];
            let len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);
            let title = String::from_utf16_lossy(&title[..len.max(0) as usize]);
            if title.to_lowercase().contains("helldivers") {
                let mut pid = 0;
                GetWindowThreadProcessId(hwnd, &mut pid);
                let mut rect = RECT::default();
                let client = GetClientRect(hwnd, &mut rect);
                println!(
                    "Game HWND={:?} pid={} visible={} client_ok={} client={}x{} error={}",
                    hwnd,
                    pid,
                    IsWindowVisible(hwnd),
                    client,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    GetLastError()
                );
            }
            1
        }
        println!("Detection: {:?}", detect_game());
        println!(
            "Worker detection: {:?}",
            std::thread::spawn(detect_game).join().unwrap()
        );
        unsafe {
            EnumWindows(Some(inspect), 0);
        }
    }
}
