// scripts/prepare-tauri-resources.mjs
// Assembles the resources/server/ directory consumed by Tauri's bundle.resources.
// Run via: npm run build:tauri-resources  (called automatically by npm run build)
//
// Output layout (fully self-contained — no node_modules needed):
//   src-tauri/resources/server/
//   ├── node.exe                      ← portable Node LTS (win-x64)
//   └── server.cjs                    ← esbuild bundle (ALL deps included)

import { mkdirSync, existsSync, writeFileSync } from 'fs';
import { execSync } from 'child_process';
import { patchYouTubeChat } from './patch-youtube-chat.mjs';

// Dynamically fetch the latest Node.js version from nodejs.org
async function resolveLatestNodeVersion() {
  try {
    const res = await fetch('https://nodejs.org/dist/index.json', {
      signal: AbortSignal.timeout(4000),
      headers: { 'User-Agent': 'Node-Fetch' }
    });
    if (res.ok) {
      const releases = await res.json();
      if (Array.isArray(releases) && releases.length > 0) {
        const latest = releases[0].version.replace(/^v/, '');
        return latest;
      }
    }
  } catch (err) {
    console.warn(`Note: Could not fetch latest Node from nodejs.org (${err.message}). Using local Node version.`);
  }
  return process.version.replace(/^v/, '') || '26.12.1';
}

const NODE_VERSION = await resolveLatestNodeVersion();
const NODE_URL = `https://nodejs.org/dist/v${NODE_VERSION}/win-x64/node.exe`;
const OUT = 'src-tauri/resources/server';

console.log(`=== Preparing Tauri resources (Node.js v${NODE_VERSION}) ===\n`);

// 1. Ensure output dir exists
mkdirSync(OUT, { recursive: true });

// 2. Verify server.cjs was already built by build:server
const serverBundle = `${OUT}/server.cjs`;
if (!existsSync(serverBundle)) {
  console.error(`ERROR: ${serverBundle} not found. Run 'npm run build:server' first.`);
  process.exit(1);
}
console.log(`✓ ${serverBundle} present`);

// 3. Download portable node.exe if not cached or version mismatch
const nodeExeDest = `${OUT}/node.exe`;
let needsDownload = true;

if (existsSync(nodeExeDest)) {
  try {
    const currentVer = execSync(`"${nodeExeDest}" -v`, { encoding: 'utf8' }).trim();
    if (currentVer === `v${NODE_VERSION}`) {
      console.log(`✓ node.exe (${currentVer}) already up to date, skipping download.`);
      needsDownload = false;
    } else {
      console.log(`Updating node.exe: found ${currentVer}, expected v${NODE_VERSION}.`);
    }
  } catch {
    needsDownload = true;
  }
}

if (needsDownload) {
  console.log(`Downloading portable Node ${NODE_VERSION} from ${NODE_URL}...`);
  try {
    execSync(`curl -L --progress-bar "${NODE_URL}" -o "${nodeExeDest}"`, { stdio: 'inherit' });
    console.log(`Downloaded node.exe → ${nodeExeDest}`);
  } catch (e) {
    console.error('ERROR: Failed to download node.exe:', e.message);
    process.exit(1);
  }
}

// 4. Install production dependencies that were excluded from the bundle
console.log('Installing external dependencies into resources/server...');
const serverPackageJson = {
  name: "server-deps",
  version: "1.0.0",
  private: true,
  type: "commonjs",
  dependencies: {
    "tiktok-live-connector": "^2.4.4",
    "youtube-chat": "^2.2.0"
  }
};
writeFileSync(`${OUT}/package.json`, JSON.stringify(serverPackageJson, null, 2));
execSync('npm install --omit=dev', { cwd: OUT, stdio: 'inherit' });
patchYouTubeChat(OUT);

// 5. Clean up package files so they don't get bundled in the installer
import { rmSync } from 'fs';
try {
  rmSync(`${OUT}/package.json`, { force: true });
  rmSync(`${OUT}/package-lock.json`, { force: true });
} catch (e) {
  console.log('Note: Could not remove package files:', e.message);
}

console.log(`\n✓ Tauri resources ready at ${OUT}`);
console.log('  Contents: node.exe + server.cjs + node_modules (with tiktok and youtube packages)');
console.log('  Frontend is embedded directly in the Tauri exe via frontendDist.');
