import { app, BrowserWindow, dialog, ipcMain } from 'electron';
import envPaths from 'env-paths';
import { mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const appPaths = envPaths('ShindenToAnilist', { suffix: '' });

mkdirSync(appPaths.data, { recursive: true });

const rendererPaths = {
  base: appPaths.data,
  database: join(appPaths.data, 'database.jsonl'),
  export: join(appPaths.data, 'export.xml')
};
const selectExportPathChannel = 'shinden-to-anilist:select-export-path';

type SelectExportPathOptions = {
  defaultPath?: string;
};

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

function createWindow(): void {
  const win = new BrowserWindow({
    width: 1200,
    height: 800,
    webPreferences: {
      additionalArguments: [
        `--shinden-to-anilist-paths=${JSON.stringify(rendererPaths)}`
      ],
      preload: join(__dirname, 'preload.cjs')
    }
  });

  const url = 'http://127.0.0.1:4173';

  win.loadURL(url);
}

app.whenReady().then(createWindow);
