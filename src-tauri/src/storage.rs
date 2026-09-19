use std::{fs, io, path::Path};

pub fn migrate() -> io::Result<()> {
    let local = std::env::var_os("LOCALAPPDATA")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "LOCALAPPDATA unavailable"))?;
    migrate_at(Path::new(&local))
}

fn migrate_at(local: &Path) -> io::Result<()> {
    let old = local.join("app.stratcom.desktop");
    let new = local.join("app.serapod.desktop");
    // Never replace an existing SERAPOD installation's data.
    if new.try_exists()? || !old.try_exists()? {
        return Ok(());
    }
    // Refuse migration while the legacy WebView is using its settings database.
    let lock = old.join("EBWebView/Default/Local Storage/leveldb/LOCK");
    if lock.try_exists()? {
        use std::os::windows::fs::OpenOptionsExt;
        let probe = fs::OpenOptions::new().read(true).write(true).share_mode(0).open(lock)?;
        drop(probe);
    }
    // Same-volume rename preserves the complete WebView profile without a partial copy.
    fs::rename(old, new)
}

pub fn show_error(error: &io::Error) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
    let message = if crate::platform::system_locale().starts_with("fr") {
        format!("Impossible de migrer les réglages vers app.serapod.desktop.\nFermez les autres instances de SERAPOD/StratCom, puis relancez l'application.\nLes données existantes n'ont pas été remplacées.\n\n{error}")
    } else {
        format!("Could not migrate settings to app.serapod.desktop.\nClose other SERAPOD/StratCom instances, then restart the app.\nExisting data has not been replaced.\n\n{error}")
    };
    let message: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
    let title: Vec<u16> = "SERAPOD".encode_utf16().chain(Some(0)).collect();
    unsafe { MessageBoxW(std::ptr::null_mut(), message.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migration_preserves_data_and_does_not_overwrite_destination() {
        let root = std::env::temp_dir().join(format!("serapod-migration-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        migrate_at(&root).unwrap();
        assert!(!root.join("app.serapod.desktop").exists());
        let old = root.join("app.stratcom.desktop");
        fs::create_dir_all(&old).unwrap();
        fs::write(old.join("settings"), "profile-data").unwrap();
        migrate_at(&root).unwrap();
        assert!(!old.exists());
        let new = root.join("app.serapod.desktop");
        assert_eq!(fs::read_to_string(new.join("settings")).unwrap(), "profile-data");
        fs::create_dir_all(&old).unwrap();
        fs::write(old.join("settings"), "legacy-data").unwrap();
        migrate_at(&root).unwrap();
        assert_eq!(fs::read_to_string(new.join("settings")).unwrap(), "profile-data");
        assert_eq!(fs::read_to_string(old.join("settings")).unwrap(), "legacy-data");
        fs::remove_dir_all(root).unwrap();
    }
}
