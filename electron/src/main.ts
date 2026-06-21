import {
  app,
  BrowserWindow,
  Menu,
  dialog,
  ipcMain,
  session,
  shell
} from 'electron';
import type { ChildProcess } from 'node:child_process';
import { spawn } from 'node:child_process';
import { existsSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { registerShindenCloudflareIpc } from './shindenCloudflare.js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const productName = 'ShindenToAnilist';
const selectExportPathChannel = 'shinden-to-anilist:select-export-path';
const getGrpcBaseUrlChannel = 'shinden-to-anilist:get-grpc-base-url';
const openExternalUrlChannel = 'shinden-to-anilist:open-external-url';
const appConfigArgumentPrefix = '--shinden-to-anilist-config=';
const sidecarReadyTimeoutMs = 15_000;
const sidecarShutdownTimeoutMs = 3_000;
const defaultDevGrpcBaseUrl = 'http://127.0.0.1:45187';
const rendererContentSecurityPolicy = [
  "default-src 'self'",
  "base-uri 'self'",
  "object-src 'none'",
  "script-src 'self'",
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data: https:",
  "font-src 'self' data:",
  "connect-src 'self' http://127.0.0.1:* ws://127.0.0.1:*"
].join('; ');

type SelectExportPathOptions = {
  defaultPath?: string;
};

type RendererPaths = {
  base: string;
  database: string;
  export: string;
};

type AppConfig = {
  paths: RendererPaths;
};

type SidecarReadyEvent = {
  event: string;
  addr?: string;
};

let rendererPaths: RendererPaths;
let sidecarProcess: ChildProcess | undefined;
let sidecarShutdownTimer: NodeJS.Timeout | undefined;
let grpcBaseUrlPromise: Promise<string> | undefined;

ipcMain.handle(
  selectExportPathChannel,
  async (event, options?: SelectExportPathOptions) => {
    const window = BrowserWindow.fromWebContents(event.sender);
    const dialogOptions = {
      title: 'Wybierz plik eksportu',
      defaultPath: options?.defaultPath ?? rendererPaths.export,
      buttonLabel: 'Eksportuj',
      filters: [
        { name: 'XML', extensions: ['xml'] },
        { name: 'Wszystkie pliki', extensions: ['*'] }
      ]
    };
    const result =
      window === null
        ? await dialog.showSaveDialog(dialogOptions)
        : await dialog.showSaveDialog(window, dialogOptions);

    return result.canceled ? null : (result.filePath ?? null);
  }
);

ipcMain.handle(getGrpcBaseUrlChannel, async () => {
  if (grpcBaseUrlPromise === undefined) {
    throw new Error('ShindenToAnilist gRPC sidecar is not initializing');
  }

  return await grpcBaseUrlPromise;
});

ipcMain.handle(openExternalUrlChannel, async (_event, url: string) => {
  const parsedUrl = new URL(url);

  if (parsedUrl.protocol !== 'https:' && parsedUrl.protocol !== 'http:') {
    throw new Error(`Unsupported external URL protocol: ${parsedUrl.protocol}`);
  }

  await shell.openExternal(parsedUrl.toString());
});

function createRendererPaths(): RendererPaths {
  const base = appDataBasePath();

  mkdirSync(base, { recursive: true });

  return {
    base,
    database: join(base, 'database.jsonl'),
    export: join(app.getPath('documents'), 'export.xml')
  };
}

function appDataBasePath(): string {
  return join(appDataHome(), productName);
}

function appDataHome(): string {
  if (process.platform !== 'linux') {
    return app.getPath('appData');
  }

  return (
    process.env.XDG_DATA_HOME ?? join(app.getPath('home'), '.local', 'share')
  );
}

function sidecarExecutableName(): string {
  return process.platform === 'win32'
    ? 'shinden-to-anilist-grpc.exe'
    : 'shinden-to-anilist-grpc';
}

function resourceDir(): string {
  return process.env.SHINDEN_TO_ANILIST_RESOURCE_DIR ?? process.resourcesPath;
}

function appIconPath(): string | undefined {
  const candidates = [
    process.env.SHINDEN_TO_ANILIST_ICON,
    join(resourceDir(), 'icon.png'),
    resolve(__dirname, '..', 'build', 'icon.png')
  ];

  return candidates.find(
    (candidate): candidate is string =>
      candidate !== undefined && existsSync(candidate)
  );
}

function sidecarPath(): string {
  return (
    process.env.SHINDEN_TO_ANILIST_GRPC_BINARY ??
    join(resourceDir(), 'sidecar', sidecarExecutableName())
  );
}

function parseSidecarReadyLine(line: string): string | undefined {
  try {
    const event = JSON.parse(line) as SidecarReadyEvent;

    if (event.event === 'ready' && event.addr !== undefined) {
      return event.addr;
    }
  } catch {
    return undefined;
  }

  return undefined;
}

function startSidecar(): Promise<string> {
  if (process.env.SHINDEN_TO_ANILIST_GRPC_BASE_URL !== undefined) {
    return Promise.resolve(process.env.SHINDEN_TO_ANILIST_GRPC_BASE_URL);
  }

  return new Promise((resolve, reject) => {
    const child = spawn(
      sidecarPath(),
      ['--listen-addr', '127.0.0.1:0', '--exit-on-stdin-close'],
      {
        stdio: ['pipe', 'pipe', 'pipe']
      }
    );

    sidecarProcess = child;
    child.stdout.setEncoding('utf8');
    child.stderr.setEncoding('utf8');

    let settled = false;
    let stdoutBuffer = '';
    let stderrBuffer = '';

    const timer = setTimeout(() => {
      if (settled) {
        return;
      }

      settled = true;
      reject(
        new Error(
          `Timed out waiting for ShindenToAnilist gRPC sidecar readiness from ${sidecarPath()}`
        )
      );
      child.kill();
    }, sidecarReadyTimeoutMs);

    child.stdout.on('data', (chunk: string) => {
      stdoutBuffer += chunk;

      for (;;) {
        const newline = stdoutBuffer.indexOf('\n');
        if (newline < 0) {
          break;
        }

        const line = stdoutBuffer.slice(0, newline).trim();
        stdoutBuffer = stdoutBuffer.slice(newline + 1);
        const addr = parseSidecarReadyLine(line);

        if (addr !== undefined && !settled) {
          settled = true;
          clearTimeout(timer);
          resolve(`http://${addr}`);
        }
      }
    });

    child.stderr.on('data', (chunk: string) => {
      stderrBuffer += chunk;
      process.stderr.write(chunk);
    });

    child.on('error', (error) => {
      if (!settled) {
        settled = true;
        clearTimeout(timer);
        reject(error);
      }
    });

    child.on('exit', (code, signal) => {
      if (sidecarProcess === child) {
        sidecarProcess = undefined;
      }

      if (sidecarShutdownTimer !== undefined) {
        clearTimeout(sidecarShutdownTimer);
        sidecarShutdownTimer = undefined;
      }

      if (!settled) {
        settled = true;
        clearTimeout(timer);
        reject(
          new Error(
            `ShindenToAnilist gRPC sidecar exited before readiness: code=${code ?? 'null'} signal=${
              signal ?? 'null'
            } stderr=${stderrBuffer.trim()}`
          )
        );
      }
    });
  });
}

function rendererDevUrl(): string | undefined {
  return process.env.SHINDEN_TO_ANILIST_RENDERER_URL;
}

function shouldHideApplicationMenu(): boolean {
  return (
    process.platform !== 'darwin' &&
    process.env.SHINDEN_TO_ANILIST_ELECTRON_DEV !== '1'
  );
}

function registerRendererContentSecurityPolicy(): void {
  session.defaultSession.webRequest.onHeadersReceived((details, callback) => {
    const responseHeaders = details.responseHeaders ?? {};

    if (!isRendererUrl(details.url)) {
      callback({ responseHeaders });
      return;
    }

    callback({
      responseHeaders: {
        ...responseHeaders,
        'Content-Security-Policy': [rendererContentSecurityPolicy]
      }
    });
  });
}

function isRendererUrl(url: string): boolean {
  const devUrl = rendererDevUrl();

  if (devUrl !== undefined) {
    return url.startsWith(devUrl);
  }

  return url.startsWith('file://');
}

async function loadRenderer(win: BrowserWindow): Promise<void> {
  const devUrl = rendererDevUrl();

  if (devUrl !== undefined) {
    await win.loadURL(devUrl);
    return;
  }

  const indexPath =
    process.env.SHINDEN_TO_ANILIST_RENDERER_DIST !== undefined
      ? join(process.env.SHINDEN_TO_ANILIST_RENDERER_DIST, 'index.html')
      : join(app.getAppPath(), 'renderer', 'index.html');

  await win.loadFile(indexPath);
}

async function createWindow(config: AppConfig): Promise<void> {
  const icon = appIconPath();
  const win = new BrowserWindow({
    width: 1200,
    height: 800,
    minWidth: 720,
    minHeight: 800,
    ...(icon === undefined ? {} : { icon }),
    webPreferences: {
      additionalArguments: [
        `${appConfigArgumentPrefix}${JSON.stringify(config)}`
      ],
      preload: join(__dirname, 'preload.cjs')
    }
  });

  await loadRenderer(win);
}

function stopSidecar(): void {
  if (sidecarProcess === undefined || sidecarProcess.killed) {
    return;
  }

  sidecarProcess.stdin?.end();
  sidecarShutdownTimer ??= setTimeout(() => {
    sidecarProcess?.kill();
  }, sidecarShutdownTimeoutMs);
  sidecarShutdownTimer.unref();
}

app.on('before-quit', stopSidecar);
process.once('exit', () => {
  sidecarProcess?.stdin?.destroy();
});
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.setName(productName);

app.whenReady().then(async () => {
  if (shouldHideApplicationMenu()) {
    Menu.setApplicationMenu(null);
  }

  registerRendererContentSecurityPolicy();
  registerShindenCloudflareIpc({ getIconPath: appIconPath });
  rendererPaths = createRendererPaths();
  grpcBaseUrlPromise =
    process.env.SHINDEN_TO_ANILIST_ELECTRON_DEV === '1'
      ? Promise.resolve(
          process.env.SHINDEN_TO_ANILIST_GRPC_BASE_URL ?? defaultDevGrpcBaseUrl
        )
      : startSidecar();
  void grpcBaseUrlPromise.catch((error: unknown) => {
    console.error('Unable to start ShindenToAnilist gRPC sidecar', error);
  });

  await createWindow({
    paths: rendererPaths
  });
});
