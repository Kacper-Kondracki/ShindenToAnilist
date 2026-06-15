import { existsSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceIcon = resolve(root, 'assets', 'sta_icon.png');
const outputDir = resolve(root, 'tauri', 'icons');

const pngIcons = [
  ['32x32.png', 32],
  ['128x128.png', 128],
  ['128x128@2x.png', 256],
  ['icon.png', 512]
];
const icoSizes = [16, 24, 32, 48, 64, 128, 256];

if (!existsSync(sourceIcon)) {
  throw new Error(`Missing source icon: ${sourceIcon}`);
}

mkdirSync(outputDir, { recursive: true });

function runFfmpeg(args) {
  const result = spawnSync('ffmpeg', ['-y', '-hide_banner', ...args], {
    stdio: 'inherit'
  });

  if (result.error?.code === 'ENOENT') {
    throw new Error(
      'ffmpeg is required to generate Tauri icons, but it was not found.'
    );
  }

  if (result.status !== 0) {
    throw new Error('ffmpeg failed while generating Tauri icons.');
  }
}

function generatePng(filename, size) {
  runFfmpeg([
    '-loglevel',
    'error',
    '-i',
    sourceIcon,
    '-vf',
    `scale=${size}:${size}:flags=lanczos,format=rgba`,
    '-frames:v',
    '1',
    resolve(outputDir, filename)
  ]);
}

function generateIco() {
  const splitLabels = icoSizes.map((size) => `[s${size}]`).join('');
  const scaleFilters = icoSizes
    .map(
      (size) =>
        `[s${size}]scale=${size}:${size}:flags=lanczos,format=bgra[i${size}]`
    )
    .join(';');
  const filterComplex = `[0:v]split=${icoSizes.length}${splitLabels};${scaleFilters}`;
  const maps = icoSizes.flatMap((size) => ['-map', `[i${size}]`]);

  runFfmpeg([
    '-loglevel',
    'error',
    '-i',
    sourceIcon,
    '-filter_complex',
    filterComplex,
    ...maps,
    resolve(outputDir, 'icon.ico')
  ]);
}

for (const [filename, size] of pngIcons) {
  generatePng(filename, size);
}

generateIco();
