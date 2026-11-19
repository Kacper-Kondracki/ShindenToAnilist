import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';

const root = new URL('../', import.meta.url);
const args = process.argv.slice(2);
const checking = args[0] === '--check';
const releaseManifest = readManifest('package.json');
const electronManifest = readManifest('electron/package.json');
const version = checking ? releaseManifest.data.version : args[0];
const semver =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([\da-zA-Z-]+(?:\.[\da-zA-Z-]+)*))?(?:\+[\da-zA-Z-]+(?:\.[\da-zA-Z-]+)*)?$/;
const match = typeof version === 'string' ? semver.exec(version) : null;

if (
  args.length !== 1 ||
  match === null ||
  match[0] !== version ||
  match[4]?.split('.').some((part) => /^0\d+$/.test(part))
) {
  throw new Error('Expected a SemVer version. Usage: pnpm version:bump 1.0.3');
}

if (readManifest('tauri/tauri.conf.json').data.version !== '../package.json') {
  throw new Error('Tauri must read its version from ../package.json.');
}

if (checking) {
  if (electronManifest.data.version !== version) {
    throw new Error(
      `electron/package.json has version ${electronManifest.data.version}. Run pnpm version:bump ${version}.`
    );
  }
  checkRustVersions();
  updateLockfile(['--locked']);
  console.log(`Release version ${version} is in sync.`);
} else {
  const changes = [];
  try {
    for (const manifest of [releaseManifest, electronManifest]) {
      if (manifest.data.version === version) continue;
      const newline = manifest.source.includes('\r\n') ? '\r\n' : '\n';
      const updated = JSON.stringify({ ...manifest.data, version }, null, 2);
      writeChanged(
        manifest.path,
        manifest.source,
        updated.replaceAll('\n', newline) + newline
      );
    }

    const path = new URL('Cargo.toml', root);
    const source = readFileSync(path, 'utf8');
    const workspaceVersion =
      /^(\[workspace\.package\](?:\r?\n(?![ \t]*\[).*)*?\r?\n[ \t]*version[ \t]*=[ \t]*)(?:"[^"\r\n]+"|'[^'\r\n]+')/m;
    if (!workspaceVersion.test(source)) {
      throw new Error('Cargo.toml is missing a workspace.package version.');
    }
    const updated = source.replace(
      workspaceVersion,
      (_, prefix) => `${prefix}"${version}"`
    );
    writeChanged(path, source, updated);
    checkRustVersions();
    updateLockfile();
  } catch (error) {
    for (const { path, source } of changes.reverse()) {
      writeFileSync(path, source);
    }
    throw error;
  }
  console.log(`Release version set to ${version}.`);

  function writeChanged(path, source, updated) {
    if (source === updated) return;
    changes.push({ path, source });
    writeFileSync(path, updated);
  }
}

function readManifest(file) {
  const path = new URL(file, root);
  const source = readFileSync(path, 'utf8');
  return { path, source, data: JSON.parse(source) };
}

function checkRustVersions() {
  const { packages } = JSON.parse(
    execFileSync('cargo', ['metadata', '--no-deps', '--format-version', '1'], {
      cwd: root,
      encoding: 'utf8'
    })
  );
  for (const pkg of packages) {
    if (pkg.version !== version) {
      throw new Error(
        `${pkg.name} has version ${pkg.version}, expected ${version}. Check workspace version inheritance and run pnpm version:bump ${version}.`
      );
    }
  }
}

function updateLockfile(args = []) {
  execFileSync('cargo', ['update', '--workspace', ...args], {
    cwd: root,
    stdio: 'inherit'
  });
}
