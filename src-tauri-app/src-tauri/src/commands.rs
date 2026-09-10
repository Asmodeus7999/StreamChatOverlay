// src/commands.rs
// #[tauri::command] functions exposed to the frontend via invoke().

use tauri::{AppHandle, Manager, WebviewWindow};
use crate::window_state::{AppState, WindowBounds};

/// Lock or unlock the overlay (click-through / chrome visibility).
/// Sets ignore-cursor-events on the window and updates AppState.
#[tauri::command]
pub async fn set_locked(
    window: WebviewWindow,
    app: AppHandle,
    locked: bool,
) -> Result<(), String> {
    window
        .set_ignore_cursor_events(locked)
        .map_err(|e| e.to_string())?;

    if !locked {
        // Restore focus and always-on-top when unlocking
        window.set_always_on_top(true).ok();
        window.show().ok();
        window.set_focus().ok();
    }

    *app.state::<AppState>().locked.lock().unwrap() = locked;
    crate::logging::write_line(&app, "commands", &format!("Overlay locked={}", locked));
    Ok(())
}

/// Toggle DevTools on the webview.
#[tauri::command]
pub async fn toggle_devtools(window: WebviewWindow) -> Result<(), String> {
    if window.is_devtools_open() {
        window.close_devtools();
    } else {
        window.open_devtools();
    }
    Ok(())
}

/// Persist current window bounds to a JSON file in app_data_dir.
#[tauri::command]
pub async fn save_bounds(window: WebviewWindow, app: AppHandle) -> Result<(), String> {
    let bounds = collect_bounds(&window)?;
    write_bounds(&app, &bounds).map_err(|e| e.to_string())
}

/// Load persisted window bounds (returns default if file is missing / corrupt).
#[tauri::command]
pub async fn load_bounds(app: AppHandle) -> Result<WindowBounds, String> {
    Ok(read_bounds(&app).unwrap_or_default())
}

// ── helpers (also used synchronously from lib.rs CloseRequested handler) ──

pub fn save_bounds_sync(window: &WebviewWindow) -> tauri::Result<()> {
    let bounds = collect_bounds(window).map_err(|e| {
        tauri::Error::Anyhow(anyhow::anyhow!(e))
    })?;
    write_bounds(window.app_handle(), &bounds)
}

fn collect_bounds(window: &WebviewWindow) -> Result<WindowBounds, String> {
    let size = window.inner_size().map_err(|e| e.to_string())?;
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    Ok(WindowBounds {
        x: Some(pos.x),
        y: Some(pos.y),
        width: size.width,
        height: size.height,
    })
}

fn bounds_path(app: &AppHandle) -> tauri::Result<std::path::PathBuf> {
    let dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&dir).ok();
    Ok(dir.join("window-bounds.json"))
}

fn write_bounds(app: &AppHandle, bounds: &WindowBounds) -> tauri::Result<()> {
    let path = bounds_path(app)?;
    let json = serde_json::to_string(bounds).unwrap_or_default();
    std::fs::write(path, json).ok();
    Ok(())
}

fn read_bounds(app: &AppHandle) -> Option<WindowBounds> {
    let path = bounds_path(app).ok()?;
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}
