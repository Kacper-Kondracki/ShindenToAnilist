import {
  chmodSync,
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  rmSync,
  writeFileSync
} from 'node:fs';
import { dirname, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import packageJson from '../package.json' with { type: 'json' };

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const appId = 'io.github.kacperkondracki.shindentoanilist';
const productName = 'ShindenToAnilist';
const executableName = 'shindentoanilist';
const version = packageJson.version;
const format = parseFormat();
const outputRoot = resolve(root, 'dist', 'system-electron');
const bundleName = `${productName}-${version}-linux-x64-system-electron`;
const bundleRoot = resolve(outputRoot, bundleName);
const sidecarSource = resolve(
  root,
  'target',
  'release',
  'shinden-to-anilist-grpc'
);

function parseFormat() {
  const formatArg = process.argv.find((arg) => arg.startsWith('--format='));
  const nextFormat =
    formatArg === undefined ? 'tar' : formatArg.slice('--format='.length);

  if (nextFormat !== 'tar' && nextFormat !== 'appimage') {
    throw new Error('--format must be tar or appimage');
  }

  return nextFormat;
}

function ensureBuiltInputs() {
  const inputs = [
    resolve(root, 'electron', 'dist', 'main.js'),
    resolve(root, 'electron', 'dist', 'preload.cjs'),
    resolve(root, 'frontend', 'dist', 'index.html'),
    sidecarSource,
    resolve(root, 'LICENSE'),
    resolve(root, 'electron', 'build', 'icon.png'),
    resolve(root, 'electron', 'build', 'icon.svg')
  ];

  for (const input of inputs) {
    if (!existsSync(input)) {
      throw new Error(`Missing required build input: ${input}`);
    }
  }
}

function copyApp(appRoot, resourceRoot) {
  mkdirSync(appRoot, { recursive: true });
  mkdirSync(resolve(resourceRoot, 'sidecar'), { recursive: true });

  cpSync(resolve(root, 'electron', 'dist'), resolve(appRoot, 'dist'), {
    recursive: true
  });
  cpSync(resolve(root, 'frontend', 'dist'), resolve(appRoot, 'renderer'), {
    recursive: true
  });
  writeFileSync(
    resolve(appRoot, 'package.json'),
    `${JSON.stringify(
      {
        name: 'shinden-to-anilist',
        version,
        productName,
        main: 'dist/main.js',
        type: 'module',
        author: 'Kacper Kondracki',
        license: 'MPL-2.0'
      },
      null,
      2
    )}\n`
  );
  copyFileSync(
    sidecarSource,
    resolve(resourceRoot, 'sidecar', 'shinden-to-anilist-grpc')
  );
  chmodSync(resolve(resourceRoot, 'sidecar', 'shinden-to-anilist-grpc'), 0o755);
}

function desktopFile(exec) {
  return `[Desktop Entry]
Type=Application
Name=${productName}
Comment=Export Shinden lists for AniList-compatible workflows
Exec=${exec}
Icon=${appId}
Terminal=false
Categories=Utility;
`;
}

function hostElectronLauncher(appPath, resourcePath) {
  return `#!/usr/bin/env sh
set -eu

ELECTRON_BIN="\${ELECTRON_EXECUTABLE:-electron}"
if ! command -v "$ELECTRON_BIN" >/dev/null 2>&1 && [ ! -x "$ELECTRON_BIN" ]; then
  if [ -x /usr/bin/electron ]; then
    ELECTRON_BIN=/usr/bin/electron
  else
    echo "ShindenToAnilist requires a host Electron executable. Set ELECTRON_EXECUTABLE or install /usr/bin/electron." >&2
    exit 1
  fi
fi

export SHINDEN_TO_ANILIST_RESOURCE_DIR="${resourcePath}"
exec "$ELECTRON_BIN" "${appPath}" "$@"
`;
}

function createTarLayout() {
  rmSync(bundleRoot, { force: true, recursive: true });
  mkdirSync(resolve(bundleRoot, 'bin'), { recursive: true });
  mkdirSync(resolve(bundleRoot, 'share', 'applications'), { recursive: true });
  mkdirSync(
    resolve(bundleRoot, 'share', 'icons', 'hicolor', '256x256', 'apps'),
    { recursive: true }
  );

  copyApp(resolve(bundleRoot, 'app'), resolve(bundleRoot, 'resources'));
  copyFileSync(resolve(root, 'LICENSE'), resolve(bundleRoot, 'LICENSE'));
  copyFileSync(
    resolve(root, 'electron', 'build', 'icon.png'),
    resolve(
      bundleRoot,
      'share',
      'icons',
      'hicolor',
      '256x256',
      'apps',
      `${appId}.png`
    )
  );
  copyFileSync(
    resolve(root, 'electron', 'build', 'icon.svg'),
    resolve(bundleRoot, `${appId}.svg`)
  );

  const launcher = resolve(bundleRoot, 'bin', executableName);
  writeFileSync(
    launcher,
    `#!/usr/bin/env sh
set -eu
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
APP_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
${hostElectronLauncher('$APP_ROOT/app', '$APP_ROOT/resources')
  .split('\n')
  .slice(3)
  .join('\n')}`
  );
  chmodSync(launcher, 0o755);

  writeFileSync(
    resolve(bundleRoot, 'share', 'applications', `${appId}.desktop`),
    desktopFile(executableName)
  );
}

function createTarball() {
  const artifact = resolve(
    root,
    'dist',
    `${productName}-${version}-linux-x64-system-electron.tar.gz`
  );
  const result = spawnSync(
    'tar',
    ['-C', outputRoot, '-czf', artifact, bundleName],
    {
      stdio: 'inherit'
    }
  );

  if (result.status !== 0) {
    throw new Error('tar failed while creating system Electron artifact');
  }
}

function createAppImage() {
  const appDir = resolve(outputRoot, `${bundleName}.AppDir`);

  rmSync(appDir, { force: true, recursive: true });
  mkdirSync(resolve(appDir, 'usr', 'bin'), { recursive: true });
  mkdirSync(resolve(appDir, 'usr', 'lib', executableName), { recursive: true });

  const appRoot = resolve(appDir, 'usr', 'lib', executableName, 'app');
  const resourceRoot = resolve(
    appDir,
    'usr',
    'lib',
    executableName,
    'resources'
  );
  copyApp(appRoot, resourceRoot);
  copyFileSync(resolve(root, 'LICENSE'), resolve(appDir, 'LICENSE'));
  copyFileSync(
    resolve(root, 'electron', 'build', 'icon.png'),
    resolve(appDir, `${appId}.png`)
  );
  copyFileSync(
    resolve(root, 'electron', 'build', 'icon.png'),
    resolve(appDir, '.DirIcon')
  );
  writeFileSync(resolve(appDir, `${appId}.desktop`), desktopFile('AppRun'));

  const appRun = resolve(appDir, 'AppRun');
  writeFileSync(
    appRun,
    hostElectronLauncher(
      '$APPDIR/usr/lib/shindentoanilist/app',
      '$APPDIR/usr/lib/shindentoanilist/resources'
    )
  );
  chmodSync(appRun, 0o755);

  const appImageTool = process.env.APPIMAGETOOL ?? 'appimagetool';
  const artifact = resolve(
    root,
    'dist',
    `${productName}-${version}-linux-x64-system-electron.AppImage`
  );
  const result = spawnSync(appImageTool, [appDir, artifact], {
    env: {
      ...process.env,
      ARCH: process.env.ARCH ?? 'x86_64'
    },
    stdio: 'inherit'
  });

  if (result.status !== 0) {
    throw new Error(
      `AppImage build failed. Install appimagetool or set APPIMAGETOOL to its path.`
    );
  }
}

ensureBuiltInputs();
mkdirSync(resolve(root, 'dist'), { recursive: true });

if (format === 'tar') {
  createTarLayout();
  createTarball();
} else {
  createAppImage();
}
