import {
  accessSync,
  copyFileSync,
  mkdirSync,
  renameSync,
  rmSync
} from 'node:fs';
import { dirname, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const options = process.argv.slice(2);
const stageDir = resolve(
  root,
  'electron',
  'build',
  ...(options.includes('--dev') ? ['dev', 'native'] : ['native'])
);
const destination = resolve(stageDir, 'shinden_to_anilist.node');

if (options.includes('--check')) {
  accessSync(destination);
  process.exit(0);
}

const args = [
  'build',
  '--locked',
  '--package',
  'shinden-to-anilist-node',
  '--message-format=json-render-diagnostics'
];

if (options.includes('--release')) args.push('--release');

const result = spawnSync('cargo', args, {
  cwd: root,
  encoding: 'utf8',
  maxBuffer: 16 * 1024 * 1024,
  stdio: ['inherit', 'pipe', 'inherit']
});

if (result.error) throw result.error;

let addon;
for (const line of result.stdout.trim().split('\n')) {
  if (line === '') continue;
  const message = JSON.parse(line);
  if (message.reason === 'compiler-message') {
    process.stderr.write(message.message.rendered ?? '');
  }
  if (
    message.reason === 'compiler-artifact' &&
    message.target.name === 'shinden_to_anilist_node' &&
    message.target.crate_types.includes('cdylib')
  ) {
    addon = message.filenames.find((name) => /\.(so|dylib|dll)$/.test(name));
  }
}

if (result.status !== 0) process.exit(result.status ?? 1);
if (addon === undefined)
  throw new Error('Cargo did not emit the native addon.');

mkdirSync(stageDir, { recursive: true });
const temporary = `${destination}.${process.pid}.tmp`;
try {
  copyFileSync(addon, temporary);
  renameSync(temporary, destination);
} finally {
  rmSync(temporary, { force: true });
}
