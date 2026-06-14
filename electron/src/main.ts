import { app, BrowserWindow, dialog, ipcMain } from 'electron';
import type { ChildProcess } from 'node:child_process';
import { spawn } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const selectExportPathChannel = 'shinden-to-anilist:select-export-path';
const appConfigArgumentPrefix = '--shinden-to-anilist-config=';
const sidecarReadyTimeoutMs = 15_000;
const defaultDevGrpcBaseUrl = 'http://127.0.0.1:45187';

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
  grpcBaseUrl: string;
};

type SidecarReadyEvent = {
  event: string;
  addr?: string;
};

let rendererPaths: RendererPaths;
let sidecarProcess: ChildProcess | undefined;

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

function createRendererPaths(): RendererPaths {
  const base = app.getPath('userData');

  mkdirSync(base, { recursive: true });

  return {
    base,
    database: join(base, 'database.jsonl'),
    export: join(base, 'export.xml')
  };
}

function sidecarExecutableName(): string {
  return process.platform === 'win32'
    ? 'shinden-to-anilist-grpc.exe'
    : 'shinden-to-anilist-grpc';
}

function resourceDir(): string {
  return process.env.SHINDEN_TO_ANILIST_RESOURCE_DIR ?? process.resourcesPath;
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
    const child = spawn(sidecarPath(), ['--listen-addr', '127.0.0.1:0'], {
      stdio: ['ignore', 'pipe', 'pipe']
    });

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
    });

    child.on('error', (error) => {
      if (!settled) {
        settled = true;
        clearTimeout(timer);
        reject(error);
      }
    });

    child.on('exit', (code, signal) => {
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
  const win = new BrowserWindow({
    width: 1200,
    height: 800,
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

  sidecarProcess.kill();
}

app.on('before-quit', stopSidecar);
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.whenReady().then(async () => {
  rendererPaths = createRendererPaths();
  const grpcBaseUrl =
    process.env.SHINDEN_TO_ANILIST_ELECTRON_DEV === '1'
      ? (process.env.SHINDEN_TO_ANILIST_GRPC_BASE_URL ?? defaultDevGrpcBaseUrl)
      : await startSidecar();

  await createWindow({
    paths: rendererPaths,
    grpcBaseUrl
  });
});
