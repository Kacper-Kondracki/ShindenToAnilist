import { rmSync } from 'node:fs';
import { dirname, resolve, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const targetArgument = process.argv[2];

if (targetArgument === undefined) {
  throw new Error('Usage: node scripts/clean-dir.mjs <repo-relative-dir>');
}

const target = resolve(root, targetArgument);
const relativeTarget = relative(root, target);

if (
  relativeTarget === '' ||
  relativeTarget.startsWith('..') ||
  relativeTarget.includes('..')
) {
  throw new Error(`Refusing to clean unsafe path: ${targetArgument}`);
}

rmSync(target, { force: true, recursive: true });
