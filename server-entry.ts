import path from 'path';
import { startServer } from './server';

// In production (Tauri sidecar): __dirname = resources/server/, so baseDir = resources/
//   public/ is bundled at resources/public/
// In dev (beforeDevCommand): BASE_DIR env var points to the project root
//   where public/ actually lives during development.
const baseDir = process.env.BASE_DIR
    ? path.resolve(process.env.BASE_DIR)
    : path.join(__dirname, '..');

startServer(baseDir);
