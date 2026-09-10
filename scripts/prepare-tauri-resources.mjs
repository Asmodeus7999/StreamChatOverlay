// scripts/prepare-tauri-resources.mjs
// Assembles the resources/server/ directory consumed by Tauri's bundle.resources.
// Run via: npm run build:tauri-resources  (called automatically by npm run build:all)
//
// Output layout:
//   src-tauri-app/src-tauri/resources/server/
//   ├── node.exe                      ← portable Node LTS (win-x64), >=20 required by tiktok-live-connector
//   ├── server.cjs                    ← esbuild bundle of server-entry.ts
//   └── node_modules/
//       ├── tiktok-live-connector/    ← full package + its FULL transitive dependency tree
//       └── youtube-chat/             ← full package (patch already applied)

import { cpSync, mkdirSync, existsSync, rmSync, mkdtempSync, readFileSync, writeFileSync } from 'fs';
import { execSync } from 'child_process';
import path from 'path';
import os from 'os';

const NODE_VERSION = '22.11.0'; // tiktok-live-connector requires Node >=20.0.0 — 18 will NOT work
const NODE_URL = `https://nodejs.org/dist/v${NODE_VERSION}/win-x64/node.exe`;
const OUT = 'src-tauri-app/src-tauri/resources/server';
const NATIVE_PACKAGES = ['tiktok-live-connector', 'youtube-chat'];

console.log('=== Preparing Tauri resources ===\n');

// 1. Ensure output dirs exist (don't clean — server.cjs is already written by build:server)
mkdirSync(`${OUT}/node_modules`, { recursive: true });

// 2. Verify server.cjs was already built by build:server
const serverBundle = `${OUT}/server.cjs`;
if (!existsSync(serverBundle)) {
  console.error(`ERROR: ${serverBundle} not found. Run 'npm run build:server' first.`);
  process.exit(1);
}
console.log(`✓ ${serverBundle} present`);

// 3. Build a COMPLETE, self-contained node_modules for the native packages.
//    Copying only node_modules/<pkg> from the root project misses anything npm
//    hoisted to the root node_modules (tiktok-live-proto, @bufbuild/protobuf,
//    got, ws, etc.) — which is invisible to Node's resolver once this folder
//    is moved outside the project tree (e.g. installed via NSIS).
//    Fix: do a clean, isolated `npm install` in a scratch folder so the
//    resulting node_modules is fully self-contained, then copy that whole tree.
if (existsSync(`${OUT}/node_modules`)) {
  rmSync(`${OUT}/node_modules`, { recursive: true, force: true });
}
mkdirSync(`${OUT}/node_modules`, { recursive: true });

const rootPkg = JSON.parse(readFileSync('package.json', 'utf8'));
const tmpDir = mkdtempSync(path.join(os.tmpdir(), 'tauri-server-deps-'));
const scratchPkg = {
  name: 'tauri-server-deps',
  version: '0.0.0',
  private: true,
  dependencies: Object.fromEntries(
    NATIVE_PACKAGES.map((p) => [p, rootPkg.dependencies[p]])
  ),
};
writeFileSync(path.join(tmpDir, 'package.json'), JSON.stringify(scratchPkg, null, 2));

console.log(`Installing self-contained dependency tree in ${tmpDir}...`);
execSync('npm install --omit=dev --no-audit --no-fund', { cwd: tmpDir, stdio: 'inherit' });

console.log(`Copying full resolved node_modules → ${OUT}/node_modules...`);
cpSync(path.join(tmpDir, 'node_modules'), `${OUT}/node_modules`, { recursive: true });

// Re-apply the youtube-chat patch by overwriting with the already-patched copy
// from the root project (avoids re-running patch-package against the scratch install).
console.log('Re-applying patched youtube-chat...');
cpSync('node_modules/youtube-chat', `${OUT}/node_modules/youtube-chat`, { recursive: true });

rmSync(tmpDir, { recursive: true, force: true });

// 4. Download portable node.exe if not already cached
const nodeExeDest = `${OUT}/node.exe`;
if (existsSync(nodeExeDest)) {
  console.log(`node.exe already present at ${nodeExeDest}, skipping download.`);
  console.log('  (If you previously had an old Node 18 build cached here, delete it manually so it re-downloads.)');
} else {
  console.log(`Downloading portable Node ${NODE_VERSION} from ${NODE_URL}...`);
  try {
    execSync(`curl -L --progress-bar "${NODE_URL}" -o "${nodeExeDest}"`, { stdio: 'inherit' });
    console.log(`Downloaded node.exe → ${nodeExeDest}`);
  } catch (e) {
    console.error('ERROR: Failed to download node.exe:', e.message);
    process.exit(1);
  }
}

// 5. Copy frontend (public/) into resources so it's bundled alongside server
const publicSrc = 'public';
const publicDest = `src-tauri-app/src-tauri/resources/public`;
if (existsSync(publicSrc)) {
  console.log(`Syncing ${publicSrc} → ${publicDest}...`);
  if (existsSync(publicDest)) rmSync(publicDest, { recursive: true });
  cpSync(publicSrc, publicDest, { recursive: true });
}

console.log(`\n✓ Tauri resources ready at ${OUT}`);
console.log('  Next: cd src-tauri-app && npm run tauri build');
