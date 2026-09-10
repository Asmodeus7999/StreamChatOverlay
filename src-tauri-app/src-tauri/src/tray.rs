// src/tray.rs
// System tray icon and menu for the desktop app.
// Provides quick actions for toggling the overlay lock, refreshing the UI, and exiting.

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let lock_item = MenuItem::with_id(app, "lock", "🔓 Overlay: UNLOCKED (Click to Lock)", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let refresh_item = MenuItem::with_id(app, "refresh", "Refresh Overlay", true, None::<&str>)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Exit StreamChat", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&lock_item, &separator, &refresh_item, &separator2, &quit_item])?;

    let icon = load_icon(app);

    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("StreamChat Overlay")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "lock" => {
                let _ = app.emit("toggle-lock", ());
            }
            "refresh" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.eval("location.reload()");
                }
            }
"quit" => {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.close();
    } else {
        app.exit(0);
    }
}
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                tray.app_handle().emit("tray-click", ()).ok();
            }
        })
        .build(app)?;

    Ok(())
}

/// Load the app icon, returning an owned Image<'static>.
fn load_icon(app: &AppHandle) -> tauri::image::Image<'static> {
    // Try to use Tauri's bundled default icon (from bundle.icon in tauri.conf.json).
    // We convert it to an owned Image<'static> by extracting the raw RGBA bytes.
    if let Some(icon) = app.default_window_icon() {
        let rgba = icon.rgba().to_vec();
        let (w, h) = (icon.width(), icon.height());
        return tauri::image::Image::new_owned(rgba, w, h);
    }

    // 1×1 transparent RGBA fallback
    tauri::image::Image::new_owned(vec![0u8; 4], 1, 1)
}
