import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { deflateSync } from 'node:zlib';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const outputDir = resolve(root, 'electron', 'build');
const appId = 'io.github.kacperkondracki.shindentoanilist';

mkdirSync(outputDir, { recursive: true });

const width = 256;
const height = 256;
const pixels = Buffer.alloc(width * height * 4);

function setPixel(x, y, r, g, b, a = 255) {
  const offset = (y * width + x) * 4;
  pixels[offset] = r;
  pixels[offset + 1] = g;
  pixels[offset + 2] = b;
  pixels[offset + 3] = a;
}

function lerp(a, b, t) {
  return Math.round(a + (b - a) * t);
}

function insideRoundedRect(x, y, left, top, right, bottom, radius) {
  const cx = Math.max(left + radius, Math.min(x, right - radius));
  const cy = Math.max(top + radius, Math.min(y, bottom - radius));
  const dx = x - cx;
  const dy = y - cy;

  return dx * dx + dy * dy <= radius * radius;
}

function drawDiagonalStroke(x, y, offset) {
  const center = y - x + offset;

  return center > -16 && center < 16;
}

for (let y = 0; y < height; y += 1) {
  for (let x = 0; x < width; x += 1) {
    const t = (x + y) / (width + height - 2);
    let r = lerp(24, 17, t);
    let g = lerp(29, 42, t);
    let b = lerp(48, 61, t);

    if (insideRoundedRect(x, y, 22, 22, 234, 234, 42)) {
      r = lerp(36, 40, t);
      g = lerp(210, 130, t);
      b = lerp(186, 236, t);
    }

    if (drawDiagonalStroke(x, y, 118) || drawDiagonalStroke(x, y, 176)) {
      r = 255;
      g = 255;
      b = 255;
    }

    if (x > 74 && x < 102 && y > 70 && y < 188) {
      r = 20;
      g = 27;
      b = 48;
    }

    if (x > 148 && x < 176 && y > 70 && y < 188) {
      r = 20;
      g = 27;
      b = 48;
    }

    setPixel(x, y, r, g, b);
  }
}

function crc32(buffer) {
  let crc = 0xffffffff;

  for (const byte of buffer) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit += 1) {
      crc = crc & 1 ? 0xedb88320 ^ (crc >>> 1) : crc >>> 1;
    }
  }

  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const typeBuffer = Buffer.from(type, 'ascii');
  const length = Buffer.alloc(4);
  const crc = Buffer.alloc(4);

  length.writeUInt32BE(data.length, 0);
  crc.writeUInt32BE(crc32(Buffer.concat([typeBuffer, data])), 0);

  return Buffer.concat([length, typeBuffer, data, crc]);
}

function pngFromPixels() {
  const rawRows = Buffer.alloc((width * 4 + 1) * height);

  for (let y = 0; y < height; y += 1) {
    const rowStart = y * (width * 4 + 1);
    rawRows[rowStart] = 0;
    pixels.copy(rawRows, rowStart + 1, y * width * 4, (y + 1) * width * 4);
  }

  const header = Buffer.alloc(13);
  header.writeUInt32BE(width, 0);
  header.writeUInt32BE(height, 4);
  header[8] = 8;
  header[9] = 6;

  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', header),
    chunk('IDAT', deflateSync(rawRows)),
    chunk('IEND', Buffer.alloc(0))
  ]);
}

function icoFromPng(png) {
  const header = Buffer.alloc(22);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(1, 4);
  header[6] = 0;
  header[7] = 0;
  header[8] = 0;
  header[9] = 0;
  header.writeUInt16LE(1, 10);
  header.writeUInt16LE(32, 12);
  header.writeUInt32LE(png.length, 14);
  header.writeUInt32LE(22, 18);

  return Buffer.concat([header, png]);
}

const png = pngFromPixels();
const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" role="img" aria-label="ShindenToAnilist">
  <title>ShindenToAnilist</title>
  <rect width="256" height="256" rx="48" fill="#141b30"/>
  <rect x="22" y="22" width="212" height="212" rx="42" fill="#24d2ba"/>
  <path d="M64 176 176 64M98 192 192 98" stroke="#fff" stroke-width="28" stroke-linecap="round"/>
  <path d="M88 72v112M162 72v112" stroke="#141b30" stroke-width="28" stroke-linecap="round"/>
</svg>
`;

writeFileSync(resolve(outputDir, 'icon.png'), png);
writeFileSync(resolve(outputDir, 'icon.ico'), icoFromPng(png));
writeFileSync(resolve(outputDir, 'icon.svg'), svg);
writeFileSync(resolve(outputDir, `${appId}.png`), png);
writeFileSync(resolve(outputDir, `${appId}.svg`), svg);
