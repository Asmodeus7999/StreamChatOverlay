// scripts/portable-zip.mjs
// Zips the portable (non-installer) build from target/release into dist/.
// Usage: npm run dist:portable   (builds fresh) or npm run zip:portable (zips existing release/ output only)
import { existsSync, mkdirSync, readFileSync } from 'fs';
import { execSync } from 'child_process';
import path from 'path';

const RELEASE_DIR = 'src-tauri-app/src-tauri/target/release';
const OUT_DIR = 'dist-portable';

const tauriConf = JSON.parse(
  readFileSync('src-tauri-app/src-tauri/tauri.conf.json', 'utf8')
);
const version = tauriConf.version;
const productName = tauriConf.productName;

const EXE_NAME = 'streamchat.exe'; // matches Cargo.toml [package].name

const requiredPaths = [
  path.join(RELEASE_DIR, EXE_NAME),
  path.join(RELEASE_DIR, 'server'),
  path.join(RELEASE_DIR, 'public'),
];

console.log('=== Building portable zip ===\n');

for (const p of requiredPaths) {
  if (!existsSync(p)) {
    console.error(`ERROR: ${p} not found. Run 'npm run dist' first to produce a release build.`);
    process.exit(1);
  }
}

mkdirSync(OUT_DIR, { recursive: true });

const zipName = `${productName}-${version}-portable-win64.zip`;
const zipPath = path.join(OUT_DIR, zipName);

console.log(`Zipping ${EXE_NAME}, server/, public/ → ${zipPath} ...`);

// PowerShell's Compress-Archive ships on every Windows install, no extra deps needed.
const psCommand = [
  `Compress-Archive`,
  `-Path '${path.join(RELEASE_DIR, EXE_NAME)}','${path.join(RELEASE_DIR, 'server')}','${path.join(RELEASE_DIR, 'public')}'`,
  `-DestinationPath '${zipPath}'`,
  `-Force`,
].join(' ');

execSync(`powershell -NoProfile -Command "${psCommand}"`, { stdio: 'inherit' });

console.log(`\n✓ Portable build ready: ${zipPath}`);
