# StreamChat Overlay (Tauri)

This folder contains the native Tauri shell for StreamChat Overlay. The desktop app loads the same local web UI as the Node/Express backend at `http://127.0.0.1:5000`, while using a bundled server sidecar and native window behavior.

## Development

From the repo root:

```bash
npm install
npm run dev
```

## Build

```bash
npm run dist
```

This runs the backend bundle step and the Tauri production build for the desktop app.

## Relevant folders

- `src-tauri/` — Rust app, window configuration, shortcuts, tray, sidecar bootstrap
- `resources/` — bundled server/runtime assets for the packaged app
- `start-server.cjs` — local dev bootstrap helper
