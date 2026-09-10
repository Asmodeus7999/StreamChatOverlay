// src/window_state.rs
// Shared application state managed by Tauri's state system.

use std::sync::Mutex;
use serde::{Deserialize, Serialize};

/// Core app state: tracks whether the overlay is locked (click-through).
#[derive(Default)]
pub struct AppState {
    pub locked: Mutex<bool>,
}

/// Window bounds persisted to disk between sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowBounds {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowBounds {
    fn default() -> Self {
        WindowBounds {
            x: None,
            y: None,
            width: 400,
            height: 520,
        }
    }
}
