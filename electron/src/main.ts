import { app, BrowserWindow } from 'electron';
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
