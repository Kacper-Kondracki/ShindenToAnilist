import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  rmSync
} from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import packageJson from '../package.json' with { type: 'json' };

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const targetTriple = parseTargetTriple();
const source = resolve(
  root,
  'target',
  ...(targetTriple === null ? [] : [targetTriple]),
  'release',
  'shinden-to-anilist-tauri'
);
const outputDir = resolve(root, 'dist', 'tauri-linux');
const output = resolve(
  outputDir,
  `ShindenToAnilist-${packageJson.version}-linux-x64-tauri`
);

if (!existsSync(source)) {
  throw new Error(`Missing Tauri Linux executable: ${source}`);
}

rmSync(outputDir, { force: true, recursive: true });
mkdirSync(outputDir, { recursive: true });
copyFileSync(source, output);
chmodSync(output, 0o755);

console.log(`Staged Tauri Linux executable: ${output}`);

function parseTargetTriple() {
  const targetArg = process.argv.find((arg) => arg.startsWith('--target='));

  if (targetArg !== undefined) {
    return targetArg.slice('--target='.length);
  }

  const targetIndex = process.argv.indexOf('--target');

  return targetIndex >= 0 ? (process.argv[targetIndex + 1] ?? null) : null;
}
