import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  rmSync
} from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import packageJson from '../package.json' with { type: 'json' };

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceDir = resolve(root, 'target', 'release', 'bundle', 'appimage');
const outputDir = resolve(root, 'dist', 'tauri-appimage');
const output = resolve(
  outputDir,
  `ShindenToAnilist-${packageJson.version}-linux-x64-tauri.AppImage`
);

if (!existsSync(sourceDir)) {
  throw new Error(`Missing Tauri AppImage output directory: ${sourceDir}`);
}

const appImages = readdirSync(sourceDir).filter((entry) =>
  entry.endsWith('.AppImage')
);

if (appImages.length !== 1) {
  throw new Error(
    `Expected one Tauri AppImage in ${sourceDir}, found ${appImages.length}`
  );
}

rmSync(outputDir, { force: true, recursive: true });
mkdirSync(outputDir, { recursive: true });
copyFileSync(resolve(sourceDir, appImages[0]), output);

console.log(`Staged Tauri AppImage: ${output}`);
