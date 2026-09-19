//! Portable GitHub release updates. No shell commands or downloaded scripts.
use reqwest::blocking::Client;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs::{self, File, OpenOptions}, io::{Read, Write}, path::{Path, PathBuf}, process::Command,
    sync::{atomic::{AtomicBool, Ordering}, Mutex}, time::{Duration, Instant}};
use tauri::{Emitter, Manager};
use windows_sys::Win32::{Foundation::*, System::Threading::*};

const REPO: &str = "odessadraekavik/SERAPOD";
const MAX_DOWNLOAD: u64 = 200 * 1024 * 1024;
static OFFER: Mutex<Option<Offer>> = Mutex::new(None);
static INSTALLING: AtomicBool = AtomicBool::new(false);
static OLD_EXE: Mutex<Option<PathBuf>> = Mutex::new(None);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Offer {
    current_version: String,
    version: String,
    notes: String,
    #[serde(skip)] url: String,
    #[serde(skip)] size: u64,
    #[serde(skip)] sha256: String,
}
#[derive(Deserialize)]
struct Release { tag_name: String, draft: bool, prerelease: bool, body: Option<String>, assets: Vec<Asset> }
#[derive(Deserialize)]
struct Asset { name: String, browser_download_url: String, size: u64, digest: Option<String> }
#[derive(Clone, Serialize)]
struct Progress { downloaded: u64, total: u64, stage: &'static str }

fn client() -> Result<Client, String> {
    Client::builder().user_agent(concat!("SERAPOD/", env!("CARGO_PKG_VERSION")))
        .https_only(true).connect_timeout(Duration::from_secs(10)).timeout(Duration::from_secs(300))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            let host = attempt.url().host_str().unwrap_or("");
            if attempt.previous().len() < 5 && attempt.url().scheme() == "https"
                && matches!(host, "api.github.com" | "github.com" | "release-assets.githubusercontent.com" | "objects.githubusercontent.com") {
                attempt.follow()
            } else { attempt.stop() }
        })).build().map_err(|_| "updateNetwork".into())
}
fn offer_from_release(release: Release, current: &str) -> Result<Option<Offer>, String> {
    let version = Version::parse(release.tag_name.strip_prefix('v').unwrap_or(&release.tag_name))
        .map_err(|_| "updateInvalid")?;
    let installed = Version::parse(current).map_err(|_| "updateInvalid")?;
    if release.draft || release.prerelease || !version.pre.is_empty() || version <= installed { return Ok(None); }
    let name = format!("SERAPOD-{version}.exe");
    let asset = release.assets.into_iter().find(|a| a.name == name).ok_or("updateInvalid")?;
    let expected_url = format!("https://github.com/{REPO}/releases/download/{}/{name}", release.tag_name);
    let hash = asset.digest.as_deref().and_then(|s| s.strip_prefix("sha256:")).ok_or("updateInvalid")?;
    if asset.browser_download_url != expected_url || asset.size < 2 || asset.size > MAX_DOWNLOAD
        || hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) { return Err("updateInvalid".into()); }
    Ok(Some(Offer { current_version: current.into(), version: version.to_string(),
        notes: release.body.unwrap_or_default().chars().take(12000).collect(),
        url: asset.browser_download_url, size: asset.size, sha256: hash.to_lowercase() }))
}
fn fetch_offer() -> Result<Option<Offer>, String> {
    let response = client()?.get(format!("https://api.github.com/repos/{REPO}/releases/latest"))
        .header("Accept", "application/vnd.github+json").timeout(Duration::from_secs(15)).send()
        .map_err(|_| "updateNetwork")?;
    if response.status() == reqwest::StatusCode::NOT_FOUND { return Ok(None); }
    let response = response.error_for_status().map_err(|_| "updateNetwork")?;
    let mut bytes = Vec::new();
    response.take(2 * 1024 * 1024 + 1).read_to_end(&mut bytes).map_err(|_| "updateNetwork")?;
    if bytes.len() > 2 * 1024 * 1024 { return Err("updateInvalid".into()); }
    let release = serde_json::from_slice(&bytes).map_err(|_| "updateInvalid")?;
    offer_from_release(release, env!("CARGO_PKG_VERSION"))
}
#[tauri::command]
pub async fn check_update(window: tauri::WebviewWindow) -> Result<Option<Offer>, String> {
    if window.label() != "main" { return Err("unauthorized".into()); }
    if INSTALLING.load(Ordering::SeqCst) { return Err("updateBusy".into()); }
    let offer = tauri::async_runtime::spawn_blocking(fetch_offer).await.map_err(|_| "updateNetwork")??;
    *OFFER.lock().unwrap() = offer.clone();
    Ok(offer)
}
struct BusyGuard;
impl Drop for BusyGuard { fn drop(&mut self) { INSTALLING.store(false, Ordering::SeqCst); } }
struct PartialFile { path: PathBuf, file: Option<File> }
impl Drop for PartialFile { fn drop(&mut self) { drop(self.file.take()); let _ = fs::remove_file(&self.path); } }
fn verify_file(path: &Path, offer: &Offer) -> Result<(), String> {
    let mut file = File::open(path).map_err(|_| "updateWrite")?;
    if file.metadata().map_err(|_| "updateWrite")?.len() != offer.size { return Err("updateIntegrity".into()); }
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop { let n = file.read(&mut buffer).map_err(|_| "updateWrite")?; if n == 0 { break; } hash.update(&buffer[..n]); }
    if format!("{:x}", hash.finalize()) != offer.sha256 { return Err("updateIntegrity".into()); }
    Ok(())
}
fn download(app: &tauri::AppHandle, offer: &Offer, old: &Path) -> Result<PathBuf, String> {
    let folder = old.parent().ok_or("updateWrite")?;
    let target = folder.join(format!("SERAPOD-{}.exe", offer.version));
    if target.exists() { verify_file(&target, offer)?; return Ok(target); }
    let partial = folder.join(format!(".SERAPOD-{}-{}.part", offer.version, std::process::id()));
    let file = OpenOptions::new().write(true).create_new(true).open(&partial).map_err(|_| "updateWrite")?;
    let mut cleanup = PartialFile { path: partial.clone(), file: Some(file) };
    let mut response = client()?.get(&offer.url).send().map_err(|_| "updateNetwork")?
        .error_for_status().map_err(|_| "updateNetwork")?;
    let mut downloaded = 0;
    let mut buffer = [0u8; 65536];
    let mut last = Instant::now() - Duration::from_secs(1);
    loop {
        let n = response.read(&mut buffer).map_err(|_| "updateNetwork")?;
        if n == 0 { break; }
        downloaded += n as u64;
        if downloaded > offer.size { return Err("updateIntegrity".into()); }
        cleanup.file.as_mut().unwrap().write_all(&buffer[..n]).map_err(|_| "updateWrite")?;
        if last.elapsed() >= Duration::from_millis(75) {
            let _ = app.emit_to("main", "update-progress", Progress { downloaded, total: offer.size, stage: "downloading" });
            last = Instant::now();
        }
    }
    cleanup.file.as_ref().unwrap().sync_all().map_err(|_| "updateWrite")?;
    drop(cleanup.file.take());
    let _ = app.emit_to("main", "update-progress", Progress { downloaded, total: offer.size, stage: "verifying" });
    verify_file(&partial, offer)?;
    fs::rename(&partial, &target).map_err(|_| "updateWrite")?;
    Ok(target)
}
#[tauri::command]
pub async fn install_update(window: tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "main" { return Err("unauthorized".into()); }
    if INSTALLING.swap(true, Ordering::SeqCst) { return Err("updateBusy".into()); }
    let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = BusyGuard;
        let offer = OFFER.lock().unwrap().clone().ok_or("updateInvalid")?;
        let old = std::env::current_exe().and_then(fs::canonicalize).map_err(|_| "updateWrite")?;
        let target = download(&app, &offer, &old)?;
        let ready = ready_path(&old, std::process::id())?;
        if ready.exists() { return Err("updateRestart".into()); }
        let mut child = Command::new(&target).arg("--serapod-update").arg(std::process::id().to_string()).arg(&old)
            .spawn().map_err(|_| "updateRestart")?;
        let deadline = Instant::now() + Duration::from_secs(15);
        while !ready.exists() {
            if Instant::now() >= deadline || child.try_wait().map_err(|_| "updateRestart")?.is_some() {
                let _ = child.kill();
                let _ = fs::remove_file(&ready);
                return Err("updateRestart".into());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = fs::remove_file(&ready);
        let _ = app.emit_to("main", "update-progress", Progress { downloaded: offer.size, total: offer.size, stage: "restarting" });
        crate::native::shutdown();
        app.exit(0);
        Ok(())
    }).await.map_err(|_| "updateRestart".to_string())?
}
fn ready_path(old: &Path, pid: u32) -> Result<PathBuf, String> {
    Ok(old.parent().ok_or("updateRestart")?.join(format!(".serapod-update-{pid}.ready")))
}
fn valid_old_path(old: &Path, current: &Path, version: &str) -> bool {
    if old == current || old.parent() != current.parent() { return false; }
    let name = old.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let old_version = name.strip_prefix("SERAPOD-").and_then(|s| s.strip_suffix(".exe")).and_then(|s| Version::parse(s).ok());
    old_version.is_some_and(|old| Version::parse(version).is_ok_and(|new| old < new))
}
struct ProcessHandle(HANDLE);
impl Drop for ProcessHandle { fn drop(&mut self) { unsafe { CloseHandle(self.0); } } }
/// Run before Tauri creates any WebViews. The replacement waits for the old
/// process to exit and share the existing WebView profile without concurrent writes.
pub fn bootstrap() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.get(1).is_none_or(|s| s != "--serapod-update") { return Ok(()); }
    if args.len() != 4 { return Err("Invalid update handoff".into()); }
    let pid: u32 = args[2].to_str().and_then(|s| s.parse().ok()).ok_or("Invalid update process")?;
    let old = fs::canonicalize(PathBuf::from(&args[3])).map_err(|_| "Old executable missing")?;
    let current = std::env::current_exe().and_then(fs::canonicalize).map_err(|_| "New executable missing")?;
    if !valid_old_path(&old, &current, env!("CARGO_PKG_VERSION")) { return Err("Invalid update paths".into()); }
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE, 0, pid);
        if process.is_null() { return Err("Could not wait for previous version".into()); }
        let process = ProcessHandle(process);
        let mut name = vec![0u16; 32768];
        let mut len = name.len() as u32;
        if QueryFullProcessImageNameW(process.0, 0, name.as_mut_ptr(), &mut len) == 0 { return Err("Could not verify previous version".into()); }
        let actual = PathBuf::from(String::from_utf16_lossy(&name[..len as usize]));
        if fs::canonicalize(actual).ok().as_ref() != Some(&old) { return Err("Update process mismatch".into()); }
        let ready = ready_path(&old, pid)?;
        OpenOptions::new().write(true).create_new(true).open(&ready).and_then(|mut f| f.write_all(b"ready"))
            .map_err(|_| "Cannot prepare update handoff")?;
        if WaitForSingleObject(process.0, 30000) != WAIT_OBJECT_0 {
            let _ = fs::remove_file(ready);
            return Err("Previous version did not exit; it has been kept".into());
        }
        let _ = fs::remove_file(ready);
    }
    *OLD_EXE.lock().unwrap() = Some(old);
    Ok(())
}
#[tauri::command]
pub fn finish_update(window: tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "main" { return Err("unauthorized".into()); }
    if let Some(old) = OLD_EXE.lock().unwrap().take() {
        // Only the path validated against the previous live process can be removed.
        fs::remove_file(old).map_err(|_| "updateCleanup".to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn release(version: &str) -> Release {
        Release { tag_name: format!("v{version}"), draft: false, prerelease: false, body: Some("Release notes".into()),
            assets: vec![Asset { name: format!("SERAPOD-{version}.exe"), browser_download_url: format!("https://github.com/{REPO}/releases/download/v{version}/SERAPOD-{version}.exe"), size: 2, digest: Some(format!("sha256:{}", "a".repeat(64))) }] }
    }
    #[test] fn semver_and_stable_releases_only() {
        assert!(offer_from_release(release("0.10.0"), "0.9.0").unwrap().is_some());
        assert!(offer_from_release(release("0.8.0"), "0.8.0").unwrap().is_none());
        assert!(offer_from_release(release("0.7.3"), "0.8.0").unwrap().is_none());
        assert!(offer_from_release(release("0.9.0-beta.1"), "0.8.0").unwrap().is_none());
        let mut r = release("0.9.0"); r.draft = true;
        assert!(offer_from_release(r, "0.8.0").unwrap().is_none());
    }
    #[test] fn rejects_wrong_origin_missing_digest_and_size() {
        let mut r = release("0.9.0"); r.assets[0].browser_download_url = "https://example.com/update.exe".into();
        assert!(offer_from_release(r, "0.8.0").is_err());
        let mut r = release("0.9.0"); r.assets[0].digest = None;
        assert!(offer_from_release(r, "0.8.0").is_err());
        let mut r = release("0.9.0"); r.assets[0].size = MAX_DOWNLOAD + 1;
        assert!(offer_from_release(r, "0.8.0").is_err());
    }
    #[test] fn handoff_never_deletes_unrelated_or_newer_files() {
        let current = Path::new(r"C:\Apps\SERAPOD-0.9.0.exe");
        assert!(valid_old_path(Path::new(r"C:\Apps\SERAPOD-0.8.0.exe"), current, "0.9.0"));
        for path in [r"C:\Other\SERAPOD-0.8.0.exe", r"C:\Apps\notes.txt", r"C:\Apps\SERAPOD-1.0.0.exe", r"C:\Apps\SERAPOD-0.9.0.exe"] {
            assert!(!valid_old_path(Path::new(path), current, "0.9.0"));
        }
    }
    #[test] fn integrity_rejects_corruption_and_truncation() {
        let path = std::env::temp_dir().join(format!("serapod-integrity-{}.tmp", std::process::id()));
        let mut offer = offer_from_release(release("0.9.0"), "0.8.0").unwrap().unwrap();
        offer.sha256 = format!("{:x}", Sha256::digest(b"MZ"));
        fs::write(&path, b"MZ").unwrap(); assert!(verify_file(&path, &offer).is_ok());
        fs::write(&path, b"XX").unwrap(); assert_eq!(verify_file(&path, &offer).unwrap_err(), "updateIntegrity");
        fs::write(&path, b"M").unwrap(); assert!(verify_file(&path, &offer).is_err());
        fs::remove_file(path).unwrap();
    }
}
