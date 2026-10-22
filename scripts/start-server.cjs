// scripts/start-server.cjs
// Dev-mode server launcher for Tauri beforeDevCommand.
// Sets BASE_DIR to the project root so Express serves public/ correctly.
const path = require('path');
process.env.BASE_DIR = path.resolve(__dirname, '..');
process.env.DEV_MODE = '1';
require('../src-tauri/resources/server/server.cjs');
