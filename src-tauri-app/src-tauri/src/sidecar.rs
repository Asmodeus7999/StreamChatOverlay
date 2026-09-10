// src/sidecar.rs
// Spawns the portable Node.exe backend, polls for readiness,
// then shows the main window. Kills the process on app exit.

use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;

/// Holds the child process handle so we can kill it on exit.
pub struct ServerHandle(pub Mutex<Option<tauri_plugin_shell::process::CommandChild>>);

impl Default for ServerHandle {
    fn default() -> Self {
        ServerHandle(Mutex::new(None))
    }
}

const SERVER_PORT: u16 = 5000;

/// Spawns `resources/server/node.exe resources/server/server.js` and
/// polls http://127.0.0.1:5000 until it responds, then shows the window.
pub fn spawn_server(app: &AppHandle) -> tauri::Result<()> {
    let resource_dir = app.path().resource_dir()?;
    let node_exe = resource_dir.join("server").join("node.exe");
    let server_js = resource_dir.join("server").join("server.cjs");

    crate::logging::write_line(app, "sidecar", &format!(
        "Spawning node: {} {}",
        node_exe.display(),
        server_js.display()
    ));

    let (mut rx, child) = app
        .shell()
        .command(node_exe.to_string_lossy().as_ref())
        .args([server_js.to_string_lossy().as_ref()])
        .env("PORT", SERVER_PORT.to_string())
        .spawn()
        .map_err(|e| tauri::Error::Anyhow(e.into()))?;

    // Stash handle for kill-on-exit
    app.state::<ServerHandle>()
        .0
        .lock()
        .unwrap()
        .replace(child);

    // Pipe stdout/stderr to the debug log
    let app_log = app.clone();
    tauri::async_runtime::spawn(async move {
        use tauri_plugin_shell::process::CommandEvent;
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    crate::logging::write_line(&app_log, "server", &String::from_utf8_lossy(&line));
                }
                CommandEvent::Stderr(line) => {
                    crate::logging::write_line(&app_log, "server-err", &String::from_utf8_lossy(&line));
                }
                CommandEvent::Error(err) => {
                    crate::logging::write_line(&app_log, "server-fatal", &err);
                }
                CommandEvent::Terminated(payload) => {
                    crate::logging::write_line(
                        &app_log,
                        "server",
                        &format!("terminated: {:?}", payload),
                    );
                }
                _ => {}
            }
        }
    });

    // Poll for readiness then reveal window
    let app_poll = app.clone();
    tauri::async_runtime::spawn(async move {
        let url = format!("http://127.0.0.1:{}", SERVER_PORT);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(20);

        loop {
            if tokio::time::Instant::now() > deadline {
                crate::logging::write_line(
                    &app_poll,
                    "sidecar",
                    "Timed out waiting for backend to become ready",
                );
                break;
            }
            match client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() || resp.status().as_u16() < 500 => {
                    crate::logging::write_line(&app_poll, "sidecar", "Backend ready — showing window");
                    if let Some(win) = app_poll.get_webview_window("main") {
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                    break;
                }
                _ => {
                    tokio::time::sleep(Duration::from_millis(150)).await;
                }
            }
        }
    });

    Ok(())
}

/// Kills the backend process. Called on CloseRequested.
pub fn kill_server(app: &AppHandle) {
    if let Some(child) = app
        .state::<ServerHandle>()
        .0
        .lock()
        .unwrap()
        .take()
    {
        let _ = child.kill();
        crate::logging::write_line(app, "sidecar", "Server process killed");
    }
}
