// scripts/dist.mjs
// Cross-shell dist script: runs build:all then tauri build
// Usage: npm run dist
import { execSync } from 'child_process';

function run(cmd, cwd = process.cwd()) {
    console.log(`\n> ${cmd}`);
    execSync(cmd, { stdio: 'inherit', shell: true, cwd });
}

run('npm run build:all');
run('npm run tauri build', new URL('../src-tauri-app', import.meta.url).pathname.replace(/^\//, ''));
