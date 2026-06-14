import { copyFileSync, mkdirSync, rmSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const platform = process.argv[2];
const stageDir = resolve(root, 'electron', 'build', 'sidecar');

if (platform !== 'linux' && platform !== 'windows') {
  throw new Error('Usage: node scripts/stage-sidecar.mjs <linux|windows>');
}

const source =
  platform === 'windows'
    ? resolve(
        root,
        'target',
        'x86_64-pc-windows-msvc',
        'release',
        'shinden-to-anilist-grpc.exe'
      )
    : resolve(root, 'target', 'release', 'shinden-to-anilist-grpc');
const destination = resolve(
  stageDir,
  platform === 'windows'
    ? 'shinden-to-anilist-grpc.exe'
    : 'shinden-to-anilist-grpc'
);

rmSync(stageDir, { force: true, recursive: true });
mkdirSync(stageDir, { recursive: true });
copyFileSync(source, destination);
