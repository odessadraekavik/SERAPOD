#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod model;
mod native;
mod platform;
mod storage;
mod updater;
use model::Config;
use tauri::Manager;

#[tauri::command]
fn configure(window: tauri::WebviewWindow, config: Config) -> Result<(), String> {
    if window.label() != "main" {
        return Err("unauthorized".into());
    }
    native::invalidate_config();
    config.validate()?;
    native::configure(config)
}
#[tauri::command]
fn status() -> native::Status {
    native::status()
}
#[tauri::command]
fn overlay_ready(window: tauri::WebviewWindow) {
    if window.label() == "overlay" {
        // The transparent window starts visible so WebView2 initializes even
        // when the game is foreground. Hide only after the JS listener exists.
        let _ = window.hide();
        native::overlay_ready();
    }
}
#[tauri::command]
fn restart_overlay(window: tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "main" {
        return Err("unauthorized".into());
    }
    native::overlay_loading();
    if let Some(overlay) = window.app_handle().get_webview_window("overlay") {
        overlay.show().map_err(|e| e.to_string())?;
        overlay.reload().map_err(|e| e.to_string())?;
    }
    Ok(())
}
#[tauri::command]
fn system_locale() -> String {
    platform::system_locale()
}
#[tauri::command]
fn test_overlay(window: tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "main" {
        return Err("unauthorized".into());
    }
    native::test_overlay()
}
fn main() {
    if updater::bootstrap().is_err() {
        // The old instance stays open and reports a failed handoff.
        return;
    }
    if let Err(error) = storage::migrate() {
        storage::show_error(&error);
        return;
    }
    tauri::Builder::default()
        .setup(|app| {
            if let Some(w) = app.get_webview_window("overlay") {
                w.set_ignore_cursor_events(true)?;
            }
            native::start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            configure,
            status,
            overlay_ready,
            system_locale,
            restart_overlay,
            test_overlay,
            updater::check_update,
            updater::install_update,
            updater::finish_update
        ])
        .on_window_event(|window, event| {
            if window.label() == "main"
                && matches!(event, tauri::WindowEvent::CloseRequested { .. })
            {
                native::shutdown();
                window.app_handle().exit(0);
            }
        })
        .run(tauri::generate_context!())
        .expect("Impossible de lancer SERAPOD");
}
