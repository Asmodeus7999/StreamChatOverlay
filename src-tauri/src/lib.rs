// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod logging;
mod shortcuts;
mod sidecar;
mod tray;
mod window_state;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(windows)]
    std::env::set_var(
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
        "--disable-features=Vulkan,DefaultANGLEVulkan,VulkanFromANGLE,TrackingPrevention,msTrackingPrevention,msWebview2TrackingPrevention --use-angle=d3d11",
    );
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Second instance launched — focus the existing window instead
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .manage(window_state::AppState::default())
        .manage(sidecar::ServerHandle::default())
        .invoke_handler(tauri::generate_handler![
            commands::set_locked,
            commands::toggle_devtools,
            commands::save_bounds,
            commands::load_bounds,
        ])
        .setup(|app| {
            logging::init(app.handle())?;

            // Restore saved window bounds before showing
            restore_window_bounds(app.handle());

            // ── Windows-only: make the overlay invisible to the input chain ──
            // WS_EX_NOACTIVATE prevents the overlay from ever stealing focus,
            // which fixes cursor coordinate mismatches in raw-input games
            // (e.g. osu!, CS2) caused by DWM compositor interference.
            #[cfg(windows)]
            if let Some(win) = app.get_webview_window("main") {
                use raw_window_handle::{HasWindowHandle, RawWindowHandle};
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE,
                };
                if let Ok(handle) = win.window_handle() {
                    if let RawWindowHandle::Win32(h) = handle.as_raw() {
                        let hwnd = h.hwnd.get() as windows_sys::Win32::Foundation::HWND;
                        unsafe {
                            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                            SetWindowLongPtrW(
                                hwnd,
                                GWL_EXSTYLE,
                                ex_style | WS_EX_NOACTIVATE as isize,
                            );
                        }
                    }
                }
            }

            sidecar::spawn_server(app.handle())?;
            shortcuts::register_all(app.handle())?;
            tray::setup_tray(app.handle())?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let app = window.app_handle();
                // Persist bounds before closing (get WebviewWindow from app handle)
                if let Some(win) = app.get_webview_window("main") {
                    commands::save_bounds_sync(&win).ok();
                }
                // Unregister shortcuts
                shortcuts::unregister_all(app);
                // Kill backend
                sidecar::kill_server(app);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Applies saved window bounds (position + size) to the main window.
/// Falls back to tauri.conf.json defaults if no saved bounds exist.
fn restore_window_bounds(app: &tauri::AppHandle) {
    use window_state::WindowBounds;

    let bounds: WindowBounds = app
        .path()
        .app_local_data_dir()
        .ok()
        .and_then(|dir| std::fs::read_to_string(dir.join("window-bounds.json")).ok())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();

    if let Some(win) = app.get_webview_window("main") {
        // Validate position is on-screen before applying
        let position_ok = bounds.x.is_some() && bounds.y.is_some();
        if position_ok {
            let x = bounds.x.unwrap();
            let y = bounds.y.unwrap();
            // Only apply if position is not wildly off-screen (basic sanity check)
            if x > -2000 && y > -2000 {
                let _ = win.set_position(tauri::PhysicalPosition::new(x, y));
            }
        }
        // Enforce minimum size so the window is always usable
        // (prevents restoring a tiny overlay-sized window on the setup screen)
        let defaults = WindowBounds::default();
        let width = if bounds.width >= 300 { bounds.width } else { defaults.width };
        let height = if bounds.height >= 200 { bounds.height } else { defaults.height };
        let _ = win.set_size(tauri::PhysicalSize::new(width, height));
    }
}
