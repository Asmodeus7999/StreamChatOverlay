// Dev-mode server launcher.
// Sets BASE_DIR to the project root (one level above src-tauri-app/)
// so the Express server finds public/ correctly during development.
const path = require('path');
process.env.BASE_DIR = path.resolve(__dirname, '..');
require('./src-tauri/resources/server/server.cjs');
