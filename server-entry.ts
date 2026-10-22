import path from 'path';
import { startServer } from './server';

// In production (Tauri sidecar): __dirname = resources/server/
//   dist/ is bundled at resources/server/dist/
// In dev (beforeDevCommand): BASE_DIR env var points to the project root
//   where the vite dev server serves files directly.
const baseDir = process.env.BASE_DIR
    ? path.resolve(process.env.BASE_DIR)
    : __dirname;

startServer(baseDir);
