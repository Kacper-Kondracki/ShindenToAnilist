import { app, BrowserWindow } from 'electron';

function createWindow(): void {
  const win = new BrowserWindow({
    width: 1200,
    height: 800
  });

  const url = 'http://127.0.0.1:4173';

  win.loadURL(url);
}

app.whenReady().then(createWindow);
