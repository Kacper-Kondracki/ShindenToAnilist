import { BrowserWindow, ipcMain, session } from 'electron';

export const openShindenCloudflareVerificationChannel =
  'shinden-to-anilist:open-shinden-cloudflare-verification';

const shindenOrigin = 'https://lista.shinden.pl/';
const shindenSessionPartition = 'persist:shinden-cloudflare';
const acceptLanguages = 'pl-PL,pl,en-US,en';
const cfClearanceCookie = 'cf_clearance';

type ShindenCloudflareClearance = {
  userAgent: string;
  cfClearance: string;
  domain: string;
  path: string;
  expiresUnixSeconds?: number;
  capturedAtMs: number;
};

type RegisterOptions = {
  getIconPath: () => string | undefined;
};

let verificationWindow: BrowserWindow | undefined;

export function registerShindenCloudflareIpc(options: RegisterOptions): void {
  ipcMain.handle(openShindenCloudflareVerificationChannel, async (event) => {
    const parent = BrowserWindow.fromWebContents(event.sender) ?? undefined;
    return await openShindenCloudflareVerification(parent, options);
  });
}

async function openShindenCloudflareVerification(
  parent: BrowserWindow | undefined,
  options: RegisterOptions
): Promise<ShindenCloudflareClearance> {
  if (verificationWindow !== undefined && !verificationWindow.isDestroyed()) {
    verificationWindow.focus();
    throw new Error('Okno weryfikacji Shinden jest już otwarte.');
  }

  const userAgent = chromiumUserAgent();
  const browserSession = session.fromPartition(shindenSessionPartition, {
    cache: true
  });
  browserSession.setUserAgent(userAgent, acceptLanguages);
  console.info('Opening Shinden Cloudflare verification window', {
    partition: shindenSessionPartition,
    userAgentLength: userAgent.length
  });

  return await new Promise((resolve, reject) => {
    const icon = options.getIconPath();
    const win = new BrowserWindow({
      width: 960,
      height: 760,
      minWidth: 720,
      minHeight: 600,
      title: 'Shinden - weryfikacja Cloudflare',
      ...(parent === undefined ? {} : { parent }),
      ...(icon === undefined ? {} : { icon }),
      webPreferences: {
        backgroundThrottling: false,
        contextIsolation: true,
        nodeIntegration: false,
        partition: shindenSessionPartition,
        sandbox: true
      }
    });

    verificationWindow = win;
    win.webContents.setUserAgent(userAgent);
    let captureStarted = false;
    let settled = false;

    win.on('close', (event) => {
      if (captureStarted) {
        return;
      }

      event.preventDefault();
      captureStarted = true;
      console.info(
        'Shinden Cloudflare verification window closed by user; capturing cookies'
      );
      void captureElectronClearance(win)
        .then((clearance) => {
          settled = true;
          resolve(clearance);
        })
        .catch((error: unknown) => {
          settled = true;
          reject(error instanceof Error ? error : new Error(String(error)));
        })
        .finally(() => {
          if (verificationWindow === win) {
            verificationWindow = undefined;
          }
          win.destroy();
        });
    });

    win.on('closed', () => {
      if (verificationWindow === win) {
        verificationWindow = undefined;
      }
      if (!settled && !captureStarted) {
        reject(
          new Error('Okno weryfikacji Shinden zostało zamknięte bez wyniku.')
        );
      }
    });

    win.loadURL(shindenOrigin).catch((error: unknown) => {
      settled = true;
      reject(error instanceof Error ? error : new Error(String(error)));
      win.destroy();
    });
  });
}

async function captureElectronClearance(
  win: BrowserWindow
): Promise<ShindenCloudflareClearance> {
  await win.webContents.session.cookies.flushStore();
  const cookies = uniqueCookies(
    await win.webContents.session.cookies.get({ url: shindenOrigin })
  );
  const cookie = cookies
    .filter((candidate) => candidate.name.toLowerCase() === cfClearanceCookie)
    .sort(compareClearanceCookies)
    .at(-1);

  if (cookie === undefined) {
    console.warn(
      'Shinden Cloudflare verification did not produce cf_clearance'
    );
    throw new Error(
      'Nie udało się odczytać ciasteczka Cloudflare z okna weryfikacji.'
    );
  }

  console.info('Captured Shinden Cloudflare clearance from Electron window', {
    userAgentLength: win.webContents.getUserAgent().length,
    cookieLength: cookie.value.length,
    domain: cookie.domain,
    path: cookie.path
  });

  return {
    userAgent: win.webContents.getUserAgent(),
    cfClearance: cookie.value,
    domain: cookie.domain ?? 'lista.shinden.pl',
    path: cookie.path ?? '/',
    ...(cookie.expirationDate === undefined
      ? {}
      : { expiresUnixSeconds: cookie.expirationDate }),
    capturedAtMs: Date.now()
  };
}

function uniqueCookies(cookies: Electron.Cookie[]) {
  const byKey = new Map<string, Electron.Cookie>();
  for (const cookie of cookies) {
    byKey.set(
      `${cookie.domain ?? ''}\n${cookie.path ?? ''}\n${cookie.name}`,
      cookie
    );
  }

  return [...byKey.values()];
}

function compareClearanceCookies(
  left: Electron.Cookie,
  right: Electron.Cookie
) {
  return cookieScore(left) - cookieScore(right);
}

function cookieScore(cookie: Electron.Cookie) {
  const domain = (cookie.domain ?? 'lista.shinden.pl')
    .trim()
    .replace(/^\./, '')
    .toLowerCase();
  const domainScore =
    domain === 'lista.shinden.pl'
      ? 300
      : domain === 'shinden.pl'
        ? 200
        : domain.endsWith('.shinden.pl')
          ? 100
          : 0;
  const pathScore = (cookie.path ?? '/') === '/' ? 10 : 0;

  return domainScore + pathScore + cookie.value.length;
}

function chromiumUserAgent() {
  const chromeVersion = process.versions.chrome;
  const platform =
    process.platform === 'win32'
      ? 'Windows NT 10.0; Win64; x64'
      : process.platform === 'darwin'
        ? 'Macintosh; Intel Mac OS X 10_15_7'
        : 'X11; Linux x86_64';

  return `Mozilla/5.0 (${platform}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/${chromeVersion} Safari/537.36`;
}
