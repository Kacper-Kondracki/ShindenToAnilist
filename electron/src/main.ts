import { logEvent, logFailure, startDiagnostics } from './diagnostics.js';
import { ipcResult } from './ipcResult.js';
import type { RendererPaths } from './ipcTypes.js';
import {
  app,
  BrowserWindow,
  Menu,
  dialog,
  ipcMain,
  session,
  shell
} from 'electron';
import { existsSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import {
  loadNativeService,
  registerNativeServiceIpc
} from './nativeService.js';
import { registerShindenCloudflareIpc } from './shindenCloudflare.js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const productName = 'ShindenToAnilist';
const selectExportPathChannel = 'shinden-to-anilist:select-export-path';
const openExternalUrlChannel = 'shinden-to-anilist:open-external-url';
const getAppPathsChannel = 'shinden-to-anilist:get-app-paths';
const githubReleaseApi =
  'https://api.github.com/repos/Kacper-Kondracki/ShindenToAnilist/releases/latest';
const rendererContentSecurityPolicy = [
  "default-src 'self'",
  "base-uri 'self'",
  "object-src 'none'",
  "script-src 'self'",
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data: https:",
  "font-src 'self' data:",
  rendererDevUrl() === undefined
    ? `connect-src 'self' ${githubReleaseApi}`
    : `connect-src 'self' ${githubReleaseApi} ws://127.0.0.1:4173`
].join('; ');

type SelectExportPathOptions = {
  defaultPath?: string;
};

ipcMain.handle(
  selectExportPathChannel,
  (event, options?: SelectExportPathOptions) =>
    ipcResult('exportDialogFailed', async () => {
      const window = BrowserWindow.fromWebContents(event.sender);
      const dialogOptions = {
        title: 'Wybierz plik eksportu',
        defaultPath:
          options?.defaultPath ?? join(app.getPath('documents'), 'export.xml'),
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
    })
);

ipcMain.on(
  'shinden-to-anilist:renderer-failure',
  (_event, operation: unknown, detail: unknown) => {
    if (
      typeof operation === 'string' &&
      /^[a-z][a-z0-9_]{0,63}$/.test(operation) &&
      typeof detail === 'string'
    ) {
      logEvent('ERROR', 'renderer operation failed', { operation, detail });
    }
  }
);

ipcMain.handle(getAppPathsChannel, () =>
  ipcResult('appPathsFailed', async () => createRendererPaths())
);

ipcMain.handle(openExternalUrlChannel, (_event, url: string) =>
  ipcResult('externalLinkFailed', async () => {
    const parsedUrl = new URL(url);

    if (parsedUrl.protocol !== 'https:' && parsedUrl.protocol !== 'http:') {
      throw new Error(
        `Unsupported external URL protocol: ${parsedUrl.protocol}`
      );
    }

    await shell.openExternal(parsedUrl.toString());
  })
);

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

function resourceDir(): string {
  const resources = process.env.SHINDEN_TO_ANILIST_RESOURCE_DIR;
  if (resources !== undefined) return resolve(resources);
  return process.env.SHINDEN_TO_ANILIST_ELECTRON_DEV === '1'
    ? resolve(__dirname, '..', 'build', 'dev')
    : process.resourcesPath;
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
    return new URL(url).origin === new URL(devUrl).origin;
  }

  return url.startsWith('file://');
}

async function createWindow(): Promise<void> {
  const icon = appIconPath();
  const win = new BrowserWindow({
    width: 1200,
    height: 800,
    minWidth: 720,
    minHeight: 800,
    ...(icon === undefined ? {} : { icon }),
    webPreferences: {
      preload: join(__dirname, 'preload.cjs')
    }
  });

  win.webContents.on('render-process-gone', (_event, details) => {
    if (details.reason !== 'clean-exit') {
      logEvent('ERROR', 'renderer process exited', {
        reason: details.reason,
        exitCode: details.exitCode
      });
    }
  });
  const rendererUrl =
    rendererDevUrl() ??
    pathToFileURL(
      process.env.SHINDEN_TO_ANILIST_RENDERER_DIST !== undefined
        ? resolve(process.env.SHINDEN_TO_ANILIST_RENDERER_DIST, 'index.html')
        : join(app.getAppPath(), 'renderer', 'index.html')
    ).href;
  registerNativeServiceIpc(
    win.webContents,
    rendererUrl,
    loadNativeService(resourceDir())
  );
  await win.loadURL(rendererUrl);
}

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.setName(productName);
startDiagnostics(app.getVersion());
process.on('uncaughtExceptionMonitor', (error) =>
  logFailure('uncaught main process exception', error)
);

app
  .whenReady()
  .then(async () => {
    if (shouldHideApplicationMenu()) {
      Menu.setApplicationMenu(null);
    }

    registerRendererContentSecurityPolicy();
    registerShindenCloudflareIpc({ getIconPath: appIconPath });
    await createWindow();
  })
  .catch((error) => {
    logFailure('desktop startup failed', error);
    app.quit();
  });
