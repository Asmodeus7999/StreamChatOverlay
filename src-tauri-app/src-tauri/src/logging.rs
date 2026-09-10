// src/logging.rs
// Debug log file management.
// Appends timestamped entries to app_data_dir/debug.log when debug mode is
// enabled by placing an `enable-debug` file next to the executable.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::{AppHandle, Manager};

static LOG_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Call once at startup from lib.rs setup().
/// Determines whether verbose logging is enabled and caches the log path.
pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_default();

    let verbose = exe_dir.join("enable-debug").exists();

    if verbose {
        let data_dir = app.path().app_data_dir()?;
        fs::create_dir_all(&data_dir).ok();
        let log_file = data_dir.join("debug.log");
        LOG_PATH.set(Some(log_file)).ok();
        write_line(app, "logging", "Debug logging enabled");
    } else {
        LOG_PATH.set(None).ok();
    }

    Ok(())
}

/// Appends a single timestamped line to the debug log (if logging is active).
pub fn write_line(_app: &AppHandle, prefix: &str, msg: &str) {
    let path = match LOG_PATH.get() {
        Some(Some(p)) => p,
        _ => return,
    };

    let timestamp = chrono_lite();
    let line = format!("[{}] [{}] {}\n", timestamp, prefix, msg);

    if let Ok(mut f) = OpenOptions::new().append(true).create(true).open(path) {
        let _ = f.write_all(line.as_bytes());
    }
}

/// Minimal timestamp without pulling in a full chrono dependency.
fn chrono_lite() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs)
}
