use crate::model::{scan, sector, Config, Node, Settings};
use serde::Serialize;
use std::{
    ptr::null_mut,
    sync::{
        atomic::{AtomicBool, AtomicIsize, AtomicU32, Ordering::SeqCst},
        mpsc::{self, Sender},
        Mutex, OnceLock,
    },
    thread,
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager};
use windows_sys::Win32::{
    Foundation::*,
    System::{LibraryLoader::GetModuleHandleW, Threading::*},
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

static ENABLED: AtomicBool = AtomicBool::new(false);
static ACTIVE: AtomicBool = AtomicBool::new(false);
static HELD: AtomicBool = AtomicBool::new(false);
static BUSY: AtomicBool = AtomicBool::new(false);
static CANCEL: AtomicBool = AtomicBool::new(false);
static READY: AtomicBool = AtomicBool::new(false);
static OVERLAY_READY: AtomicBool = AtomicBool::new(false);
static HAS_CONFIG: AtomicBool = AtomicBool::new(false);
static LEFT_UP: AtomicBool = AtomicBool::new(false);
static RIGHT_UP: AtomicBool = AtomicBool::new(false);
static ESC_UP: AtomicBool = AtomicBool::new(false);
static TRIGGER: AtomicU32 = AtomicU32::new(0x3b);
static GAME: AtomicIsize = AtomicIsize::new(0);
static GAME_RUNNING: AtomicBool = AtomicBool::new(false);
static TESTING: AtomicBool = AtomicBool::new(false);
static OVERLAY_VISIBLE: AtomicBool = AtomicBool::new(false);
static STOPPED: AtomicBool = AtomicBool::new(false);
static TRIGGER_COUNT: AtomicU32 = AtomicU32::new(0);
static OPEN_COUNT: AtomicU32 = AtomicU32::new(0);
static HOOK_THREAD: AtomicU32 = AtomicU32::new(0);
static TX: OnceLock<Sender<Event>> = OnceLock::new();
static ERROR: Mutex<String> = Mutex::new(String::new());
enum Event {
    Configure(Config),
    Open,
    Test,
    Release,
    Move(f64, f64),
    Click,
    Back,
    Cancel,
    Stop,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    detection_detail: String,
    game: bool,
    game_window: bool,
    foreground: bool,
    enabled: bool,
    hook_ready: bool,
    overlay_ready: bool,
    configured: bool,
    overlay_visible: bool,
    trigger_count: u32,
    open_count: u32,
    error: String,
}
pub fn status() -> Status {
    Status {
        detection_detail: crate::platform::DETECTION_DETAIL.lock().unwrap().clone(),
        game: GAME_RUNNING.load(SeqCst),
        game_window: GAME.load(SeqCst) != 0,
        foreground: is_target(),
        enabled: ENABLED.load(SeqCst) && GAME.load(SeqCst) != 0,
        hook_ready: READY.load(SeqCst),
        overlay_ready: OVERLAY_READY.load(SeqCst),
        configured: HAS_CONFIG.load(SeqCst),
        overlay_visible: OVERLAY_VISIBLE.load(SeqCst),
        trigger_count: TRIGGER_COUNT.load(SeqCst),
        open_count: OPEN_COUNT.load(SeqCst),
        error: ERROR.lock().unwrap().clone(),
    }
}
fn emit(e: Event) {
    if let Some(tx) = TX.get() {
        let _ = tx.send(e);
    }
}
pub fn overlay_ready() {
    OVERLAY_READY.store(true, SeqCst);
    refresh_enabled();
}
pub fn overlay_loading() {
    OVERLAY_READY.store(false, SeqCst);
    CANCEL.store(true, SeqCst);
    emit(Event::Cancel);
    refresh_enabled();
}
fn refresh_enabled() {
    ENABLED.store(
        READY.load(SeqCst)
            && OVERLAY_READY.load(SeqCst)
            && HAS_CONFIG.load(SeqCst)
            && !STOPPED.load(SeqCst),
        SeqCst,
    );
}
pub fn invalidate_config() {
    HAS_CONFIG.store(false, SeqCst);
    ENABLED.store(false, SeqCst);
    CANCEL.store(true, SeqCst);
    emit(Event::Cancel);
}
pub fn test_overlay() -> Result<(), String> {
    if !HAS_CONFIG.load(SeqCst) || !OVERLAY_READY.load(SeqCst) {
        return Err("notReady".into());
    }
    if HELD.load(SeqCst) || BUSY.load(SeqCst) {
        return Err("releaseShortcut".into());
    }
    emit(Event::Test);
    Ok(())
}
pub fn configure(c: Config) -> Result<(), String> {
    ENABLED.store(false, SeqCst);
    CANCEL.store(true, SeqCst);
    emit(Event::Cancel);
    if HELD.load(SeqCst) || BUSY.load(SeqCst) {
        return Err("releaseShortcut".into());
    }
    HAS_CONFIG.store(false, SeqCst);
    emit(Event::Configure(c));
    Ok(())
}
pub fn shutdown() {
    STOPPED.store(true, SeqCst);
    ENABLED.store(false, SeqCst);
    CANCEL.store(true, SeqCst);
    emit(Event::Stop);
    // Allow RAII guards to release any injected key before process exit.
    let deadline = Instant::now() + Duration::from_millis(150);
    while BUSY.load(SeqCst) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(3));
    }
    unsafe {
        PostThreadMessageW(HOOK_THREAD.load(SeqCst), WM_QUIT, 0, 0);
    }
}
fn is_target() -> bool {
    let target = GAME.load(SeqCst);
    target != 0 && unsafe { GetForegroundWindow() as isize == target }
}
fn trigger_event(down: bool) -> bool {
    if down {
        if HELD.load(SeqCst) {
            return true;
        }
        TRIGGER_COUNT.fetch_add(1, SeqCst);
        if ENABLED.load(SeqCst) && is_target() && !BUSY.load(SeqCst) {
            HELD.store(true, SeqCst);
            ACTIVE.store(true, SeqCst);
            CANCEL.store(false, SeqCst);
            emit(Event::Open);
            return true;
        }
    } else if HELD.swap(false, SeqCst) {
        emit(Event::Release);
        return true;
    }
    false
}
unsafe extern "system" fn keyboard_hook(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code >= 0 {
        let data = &*(l as *const KBDLLHOOKSTRUCT);
        if data.flags & LLKHF_INJECTED == 0 {
            let down = w as u32 == WM_KEYDOWN || w as u32 == WM_SYSKEYDOWN;
            let up = w as u32 == WM_KEYUP || w as u32 == WM_SYSKEYUP;
            let key = data.scanCode
                | if data.flags & LLKHF_EXTENDED != 0 {
                    0x100
                } else {
                    0
                };
            if (down || up) && key == TRIGGER.load(SeqCst) && trigger_event(down) {
                return 1;
            }
            if data.vkCode == VK_ESCAPE as u32 {
                if down && (ACTIVE.load(SeqCst) || BUSY.load(SeqCst) || ESC_UP.load(SeqCst)) {
                    ESC_UP.store(true, SeqCst);
                    CANCEL.store(true, SeqCst);
                    emit(Event::Cancel);
                    return 1;
                }
                if up && ESC_UP.swap(false, SeqCst) {
                    return 1;
                }
            }
        }
    }
    CallNextHookEx(null_mut(), code, w, l)
}
unsafe extern "system" fn mouse_hook(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code >= 0 {
        let data = &*(l as *const MSLLHOOKSTRUCT);
        if data.flags & LLMHF_INJECTED == 0 {
            let msg = w as u32;
            if msg == WM_XBUTTONDOWN || msg == WM_XBUTTONUP {
                let key = 0x10000 + (data.mouseData >> 16);
                if key == TRIGGER.load(SeqCst) && trigger_event(msg == WM_XBUTTONDOWN) {
                    return 1;
                }
            }
            if msg == WM_LBUTTONUP && LEFT_UP.swap(false, SeqCst) {
                return 1;
            }
            if msg == WM_RBUTTONUP && RIGHT_UP.swap(false, SeqCst) {
                return 1;
            }
            if ACTIVE.load(SeqCst) && is_target() {
                if msg == WM_RBUTTONDOWN {
                    RIGHT_UP.store(true, SeqCst);
                    emit(Event::Back);
                    return 1;
                }
                if msg == WM_MOUSEMOVE {
                    let mut p = POINT::default();
                    GetCursorPos(&mut p);
                    emit(Event::Move(
                        (data.pt.x - p.x) as f64,
                        (data.pt.y - p.y) as f64,
                    ));
                    return 1;
                }
                if msg == WM_LBUTTONDOWN {
                    LEFT_UP.store(true, SeqCst);
                    emit(Event::Click);
                    return 1;
                }
                if msg == WM_MOUSEWHEEL || msg == WM_MOUSEHWHEEL {
                    return 1;
                }
            }
        }
    }
    CallNextHookEx(null_mut(), code, w, l)
}
#[derive(Clone, Serialize)]
struct Frame {
    cursor: [f64; 2],
    open: bool,
    items: Vec<Node>,
    selected: i32,
    title: String,
    locale: String,
    testing: bool,
    glass: bool,
    transparency: f64,
}
struct Engine {
    app: tauri::AppHandle,
    config: Option<Config>,
    stack: Vec<Node>,
    x: f64,
    y: f64,
    selected: Option<usize>,
    opened: Instant,
    scale: f64,
    pointer_dirty: bool,
    last_frame: Instant,
}
impl Engine {
    fn frame(&mut self) {
        self.pointer_dirty = false;
        self.last_frame = Instant::now();
        let node = self.stack.last();
        let _ = self.app.emit_to(
            "overlay",
            "wheel",
            Frame {
                cursor: [self.x, self.y],
                open: ACTIVE.load(SeqCst) || TESTING.load(SeqCst),
                items: node.map(|n| n.children.clone()).unwrap_or_default(),
                selected: self.selected.map(|i| i as i32).unwrap_or(-1),
                title: node.map(|n| n.name.clone()).unwrap_or_default(),
                locale: self
                    .config
                    .as_ref()
                    .map(|c| c.settings.locale.clone())
                    .unwrap_or_else(|| "en".into()),
                testing: TESTING.load(SeqCst),
                glass: self
                    .config
                    .as_ref()
                    .map(|c| c.settings.glass)
                    .unwrap_or(true),
                transparency: self
                    .config
                    .as_ref()
                    .map(|c| c.settings.transparency)
                    .unwrap_or(35.0),
            },
        );
    }
    fn close(&mut self) {
        ACTIVE.store(false, SeqCst);
        TESTING.store(false, SeqCst);
        OVERLAY_VISIBLE.store(false, SeqCst);
        if let Some(w) = self.app.get_webview_window("overlay") {
            let _ = w.hide();
        }
        self.stack.clear();
        self.selected = None;
        self.frame();
    }
    fn open(&mut self, testing: bool) {
        if self.config.is_none() || (!testing && (!ENABLED.load(SeqCst) || !is_target())) {
            self.close();
            return;
        }
        let config = self.config.as_ref().unwrap();
        self.stack = vec![config.root.clone()];
        self.x = 0.;
        self.y = 0.;
        self.selected = None;
        self.opened = Instant::now();
        TESTING.store(testing, SeqCst);
        ACTIVE.store(!testing, SeqCst);
        *ERROR.lock().unwrap() = String::new();
        if let Some(w) = self.app.get_webview_window("overlay") {
            self.scale = w.scale_factor().unwrap_or(1.0);
            let physical = (config.settings.size * self.scale) as i32;
            // Reserve space below the square wheel without moving its center.
            let window_height = physical + (48.0 * self.scale).ceil() as i32;
            unsafe {
                let mut rect = RECT::default();
                let target = if testing {
                    self.app
                        .get_webview_window("main")
                        .and_then(|m| m.hwnd().ok())
                        .map(|h| h.0 as HWND)
                        .unwrap_or(GetForegroundWindow())
                } else {
                    GAME.load(SeqCst) as HWND
                };
                if GetWindowRect(target, &mut rect) == 0 {
                    *ERROR.lock().unwrap() = "overlayFailed".into();
                    self.close();
                    return;
                }
                if let Ok(handle) = w.hwnd() {
                    // Keep the runtime's visibility and WebView bounds in sync.
                    // Showing only the HWND with SetWindowPos bypasses that state.
                    if w.set_size(tauri::PhysicalSize::new(physical as u32, window_height as u32))
                        .is_err()
                        || w.set_position(tauri::PhysicalPosition::new(
                            (rect.left + rect.right - physical) / 2,
                            (rect.top + rect.bottom - physical) / 2,
                        ))
                        .is_err()
                        || w.show().is_err()
                    {
                        *ERROR.lock().unwrap() = "overlayFailed".into();
                        self.close();
                        return;
                    }
                    let shown = SetWindowPos(
                        handle.0 as HWND,
                        HWND_TOPMOST,
                        (rect.left + rect.right - physical) / 2,
                        (rect.top + rect.bottom - physical) / 2,
                        physical,
                        window_height,
                        SWP_NOACTIVATE | SWP_SHOWWINDOW,
                    );
                    if shown == 0 {
                        *ERROR.lock().unwrap() = "overlayFailed".into();
                        self.close();
                        return;
                    }
                    OVERLAY_VISIBLE.store(w.is_visible().unwrap_or(false), SeqCst);
                    OPEN_COUNT.fetch_add(1, SeqCst);
                    self.frame();
                }
            }
        }
    }
    fn movement(&mut self, dx: f64, dy: f64) {
        if !ACTIVE.load(SeqCst) {
            return;
        }
        if !is_target() {
            self.close();
            return;
        }
        let s = &self.config.as_ref().unwrap().settings;
        let factor = s.sensitivity * 560.0 / (s.size * self.scale);
        self.x += dx * factor;
        self.y += dy * factor;
        let r = self.x.hypot(self.y);
        if r > 240.0 {
            self.x *= 240.0 / r;
            self.y *= 240.0 / r;
        }
        let next = sector(
            self.x,
            self.y,
            self.stack.last().unwrap().children.len(),
            s.deadzone,
        );
        if next != self.selected {
            self.selected = next;
            self.frame();
        } else {
            self.pointer_dirty = true;
        }
    }
    fn back(&mut self) {
        if !ACTIVE.load(SeqCst) {
            return;
        }
        if !is_target() || CANCEL.load(SeqCst) || self.stack.len() <= 1 {
            self.close();
            return;
        }
        self.stack.pop();
        self.x = 0.;
        self.y = 0.;
        self.selected = None;
        self.frame();
    }
    fn select(&mut self, click: bool) {
        if !ACTIVE.load(SeqCst) {
            return;
        }
        if !is_target() || CANCEL.load(SeqCst) {
            self.close();
            return;
        }
        let n = self
            .selected
            .and_then(|i| self.stack.last().and_then(|n| n.children.get(i)))
            .cloned();
        match n {
            Some(n) if n.kind == "folder" => {
                if click {
                    self.stack.push(n);
                    self.x = 0.;
                    self.y = 0.;
                    self.selected = None;
                    self.frame();
                } else {
                    self.close();
                }
            }
            Some(n) => {
                let settings = self.config.as_ref().unwrap().settings.clone();
                self.close();
                BUSY.store(true, SeqCst);
                if let Err(e) = send_sequence(&settings, &n.code) {
                    *ERROR.lock().unwrap() = e;
                }
                BUSY.store(false, SeqCst);
            }
            None => {
                if click && self.stack.len() > 1 {
                    self.stack.pop();
                    self.x = 0.;
                    self.y = 0.;
                    self.selected = None;
                    self.frame();
                } else {
                    self.close();
                }
            }
        }
    }
}
fn send_key(key: &str, up: bool) -> Result<(), String> {
    let (code, extended) = scan(key).ok_or("invalidKey")?;
    let flags = KEYEVENTF_SCANCODE
        | if extended { KEYEVENTF_EXTENDEDKEY } else { 0 }
        | if up { KEYEVENTF_KEYUP } else { 0 };
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: code,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let sent = unsafe { SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) };
    if sent == 1 {
        Ok(())
    } else {
        Err("inputRefused".into())
    }
}
struct HeldKey(String);
impl Drop for HeldKey {
    fn drop(&mut self) {
        let _ = send_key(&self.0, true);
    }
}
fn check_target() -> Result<(), String> {
    if !is_target() || CANCEL.load(SeqCst) || !ENABLED.load(SeqCst) {
        Err("inputCancelled".into())
    } else {
        Ok(())
    }
}
fn pause(ms: u64) -> Result<(), String> {
    let end = Instant::now() + Duration::from_millis(ms);
    while Instant::now() < end {
        check_target()?;
        thread::sleep(Duration::from_millis(3));
    }
    check_target()
}
fn tap(key: &str, ms: u64) -> Result<(), String> {
    check_target()?;
    send_key(key, false)?;
    let held = HeldKey(key.into());
    let result = pause(ms);
    drop(held);
    result
}
fn send_sequence(s: &Settings, code: &[String]) -> Result<(), String> {
    check_target()?;
    *ERROR.lock().unwrap() = String::new();
    // Do not mix a macro with keys the user is already holding.
    for key in [&s.game_key, &s.up, &s.down, &s.left, &s.right] {
        let (scan, ext) = scan(key).unwrap();
        let vk = unsafe {
            MapVirtualKeyW(
                scan as u32 | if ext { 0xe000 } else { 0 },
                MAPVK_VSC_TO_VK_EX,
            )
        };
        if unsafe { GetAsyncKeyState(vk as i32) } < 0 {
            return Err("releaseGameKeys".into());
        }
    }
    let mut held = None;
    if s.mode == "hold" {
        send_key(&s.game_key, false)?;
        held = Some(HeldKey(s.game_key.clone()));
    } else {
        tap(&s.game_key, s.press_ms)?;
    }
    pause(s.gap_ms)?;
    for direction in code {
        let key = match direction.as_str() {
            "Up" => &s.up,
            "Down" => &s.down,
            "Left" => &s.left,
            "Right" => &s.right,
            _ => return Err("invalidKey".into()),
        };
        tap(key, s.press_ms)?;
        pause(s.gap_ms)?;
    }
    drop(held);
    Ok(())
}
pub fn start(app: tauri::AppHandle) {
    let (tx, rx) = mpsc::channel();
    let _ = TX.set(tx);
    thread::spawn(move || {
        let mut e = Engine {
            app,
            config: None,
            stack: vec![],
            x: 0.,
            y: 0.,
            selected: None,
            opened: Instant::now(),
            scale: 1.,
            pointer_dirty: false,
            last_frame: Instant::now(),
        };
        loop {
            match rx.recv_timeout(Duration::from_millis(8)) {
                Ok(Event::Configure(c)) => {
                    e.close();
                    let key = if c.settings.trigger == "Mouse4" {
                        0x10001
                    } else if c.settings.trigger == "Mouse5" {
                        0x10002
                    } else {
                        let (s, x) = scan(&c.settings.trigger).unwrap();
                        s as u32 | if x { 0x100 } else { 0 }
                    };
                    TRIGGER.store(key, SeqCst);
                    e.config = Some(c);
                    HAS_CONFIG.store(true, SeqCst);
                    *ERROR.lock().unwrap() = String::new();
                    refresh_enabled();
                }
                Ok(Event::Open) => e.open(false),
                Ok(Event::Test) => e.open(true),
                Ok(Event::Release) => e.select(false),
                Ok(Event::Click) => e.select(true),
                Ok(Event::Back) => e.back(),
                Ok(Event::Move(x, y)) => e.movement(x, y),
                Ok(Event::Cancel) => e.close(),
                Ok(Event::Stop) => {
                    e.close();
                    break;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            if ACTIVE.load(SeqCst)
                && (!is_target()
                    || CANCEL.load(SeqCst)
                    || e.opened.elapsed() > Duration::from_secs(30))
            {
                e.close();
            }
            if TESTING.load(SeqCst) && e.opened.elapsed() > Duration::from_secs(8) {
                e.close();
            }
            if ACTIVE.load(SeqCst) && e.pointer_dirty && e.last_frame.elapsed() >= Duration::from_millis(8) {
                e.frame();
            }
        }
    });
    thread::spawn(|| {
        let diagnostic_log = std::env::var_os("SERAPOD_DIAGNOSTIC_LOG");
        while !STOPPED.load(SeqCst) {
            let (running, window) = crate::platform::detect_game();
            GAME_RUNNING.store(running, SeqCst);
            GAME.store(window, SeqCst);
            refresh_enabled();
            if let Some(path) = &diagnostic_log {
                if let Ok(json) = serde_json::to_string_pretty(&status()) {
                    let _ = std::fs::write(path, json);
                }
            }
            thread::sleep(Duration::from_millis(500));
        }
    });
    thread::spawn(|| unsafe {
        HOOK_THREAD.store(GetCurrentThreadId(), SeqCst);
        let module = GetModuleHandleW(std::ptr::null());
        let keyboard = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), module, 0);
        let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), module, 0);
        if keyboard.is_null() || mouse.is_null() {
            *ERROR.lock().unwrap() = "hookFailed".into();
        } else {
            READY.store(true, SeqCst);
            refresh_enabled();
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        READY.store(false, SeqCst);
        if !keyboard.is_null() {
            UnhookWindowsHookEx(keyboard);
        }
        if !mouse.is_null() {
            UnhookWindowsHookEx(mouse);
        }
    });
}
