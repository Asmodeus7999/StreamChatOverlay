// scripts/patch-youtube-chat.mjs
// Lightweight, dependency-free patch for youtube-chat.
// Fixes runtime crash when runs is undefined/null in parseMessages().
import { readFileSync, writeFileSync, existsSync } from 'fs';
import path from 'path';

export function patchYouTubeChat(targetDir = process.cwd()) {
  const parserPath = path.resolve(targetDir, 'node_modules/youtube-chat/dist/parser.js');
  if (!existsSync(parserPath)) {
    return false;
  }

  let code = readFileSync(parserPath, 'utf8');
  if (code.includes('if (!runs) return [];')) {
    return true; // Already patched
  }

  const target = 'function parseMessages(runs) {';
  if (!code.includes(target)) {
    console.warn('⚠️ patch-youtube-chat: target signature function parseMessages(runs) not found.');
    return false;
  }

  code = code.replace(target, 'function parseMessages(runs) {\n    if (!runs) return [];');
  writeFileSync(parserPath, code, 'utf8');
  console.log(`✓ Applied null-safety patch to ${parserPath}`);
  return true;
}

// Run immediately when invoked directly via node
patchYouTubeChat();
