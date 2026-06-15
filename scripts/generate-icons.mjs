import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourceIcon = resolve(root, 'assets', 'sta_icon.png');
const outputDir = resolve(root, 'electron', 'build');
const appId = 'io.github.kacperkondracki.shindentoanilist';

const pngIcons = [
  ['icon.png', 256],
  [`${appId}.png`, 256]
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
      'ffmpeg is required to generate icons, but it was not found.'
    );
  }

  if (result.status !== 0) {
    throw new Error('ffmpeg failed while generating application icons.');
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

function generateSvgWrapper(filename) {
  const source = readFileSync(sourceIcon);
  const encoded = source.toString('base64');
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" role="img" aria-label="ShindenToAnilist">
  <title>ShindenToAnilist</title>
  <image width="1024" height="1024" href="data:image/png;base64,${encoded}"/>
</svg>
`;

  writeFileSync(resolve(outputDir, filename), svg);
}

for (const [filename, size] of pngIcons) {
  generatePng(filename, size);
}

generateIco();
generateSvgWrapper('icon.svg');
generateSvgWrapper(`${appId}.svg`);
