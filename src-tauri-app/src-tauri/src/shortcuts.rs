// src/shortcuts.rs
// Global shortcut registration for Ctrl+Alt+L (lock toggle) and
// Ctrl+Alt+I (temporary interact while locked).
// Emits Tauri events that the frontend listens to via @tauri-apps/api/event.

use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub fn register_all(app: &AppHandle) -> tauri::Result<()> {
    let lock_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyL);
    let interact_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyI);

    app.global_shortcut()
        .on_shortcuts([lock_shortcut, interact_shortcut], move |app, shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            if shortcut == &lock_shortcut {
                crate::logging::write_line(app, "shortcuts", "Ctrl+Alt+L pressed");
                let _ = app.emit("toggle-lock", ());
            } else if shortcut == &interact_shortcut {
                crate::logging::write_line(app, "shortcuts", "Ctrl+Alt+I pressed");
                let _ = app.emit("toggle-interact", ());
            }
        })
        .map_err(|e| tauri::Error::Anyhow(e.into()))?;

    crate::logging::write_line(app, "shortcuts", "Registered Ctrl+Alt+L and Ctrl+Alt+I");
    Ok(())
}

pub fn unregister_all(app: &AppHandle) {
    app.global_shortcut().unregister_all().ok();
}
