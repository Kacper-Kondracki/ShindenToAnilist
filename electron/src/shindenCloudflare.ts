import { BrowserWindow, ipcMain, session } from 'electron';

export const openShindenCloudflareVerificationChannel =
  'shinden-to-anilist:open-shinden-cloudflare-verification';

const shindenOrigin = 'https://lista.shinden.pl/';
const shindenParentOrigin = 'https://shinden.pl/';
const shindenSessionPartition = 'persist:shinden-cloudflare';
const acceptLanguages = 'pl-PL,pl,en-US,en';
const cfClearanceCookie = 'cf_clearance';
const shindenRequestFilter = {
  urls: [
    'https://lista.shinden.pl/*',
    'https://*.shinden.pl/*',
    'https://challenges.cloudflare.com/*'
  ]
};

type ShindenCloudflareClearance = {
  userAgent: string;
  cfClearance: string;
  domain: string;
  path: string;
  expiresUnixSeconds?: number;
  capturedAtMs: number;
};

type CapturedClearanceCookie = {
  value: string;
  domain: string;
  path: string;
  expiresUnixSeconds?: number;
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
  configureShindenRequestHeaders(browserSession, userAgent);
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

    win
      .loadURL(shindenOrigin, {
        userAgent,
        extraHeaders: `Accept-Language: ${acceptLanguages}`
      })
      .catch((error: unknown) => {
        settled = true;
        reject(error instanceof Error ? error : new Error(String(error)));
        win.destroy();
      });
  });
}

async function captureElectronClearance(
  win: BrowserWindow
): Promise<ShindenCloudflareClearance> {
  const userAgent = await browserWindowUserAgent(win);
  await win.webContents.session.cookies.flushStore();
  const cookies = await collectElectronCookies(win.webContents.session);
  const cookie =
    bestElectronClearanceCookie(cookies) ??
    (await browserWindowClearanceCookie(win));

  if (cookie === undefined) {
    console.warn(
      'Shinden Cloudflare verification did not produce cf_clearance',
      {
        cookieCount: cookies.length,
        currentUrl: win.webContents.getURL(),
        cookies: summarizeCookies(cookies)
      }
    );
    throw new Error(
      'Nie udało się odczytać ciasteczka Cloudflare z okna weryfikacji.'
    );
  }

  console.info('Captured Shinden Cloudflare clearance from Electron window', {
    userAgentLength: userAgent.length,
    cookieLength: cookie.value.length,
    domain: cookie.domain,
    path: cookie.path
  });

  return {
    userAgent,
    cfClearance: cookie.value,
    domain: cookie.domain,
    path: cookie.path,
    ...(cookie.expiresUnixSeconds === undefined
      ? {}
      : { expiresUnixSeconds: cookie.expiresUnixSeconds }),
    capturedAtMs: Date.now()
  };
}

async function collectElectronCookies(browserSession: Electron.Session) {
  const cookies = await Promise.all([
    browserSession.cookies.get({ url: shindenOrigin }),
    browserSession.cookies.get({ url: shindenParentOrigin }),
    browserSession.cookies.get({ name: cfClearanceCookie }),
    browserSession.cookies.get({})
  ]);

  return uniqueCookies(cookies.flat());
}

function bestElectronClearanceCookie(
  cookies: Electron.Cookie[]
): CapturedClearanceCookie | undefined {
  const cookie = cookies
    .filter((candidate) => candidate.name.toLowerCase() === cfClearanceCookie)
    .filter((candidate) =>
      isShindenCookieDomain(candidate.domain ?? 'lista.shinden.pl')
    )
    .sort(compareClearanceCookies)
    .at(-1);

  if (cookie === undefined) {
    return undefined;
  }

  return {
    value: cookie.value,
    domain: cookie.domain ?? 'lista.shinden.pl',
    path: cookie.path ?? '/',
    ...(cookie.expirationDate === undefined
      ? {}
      : { expiresUnixSeconds: cookie.expirationDate })
  };
}

async function browserWindowClearanceCookie(
  win: BrowserWindow
): Promise<CapturedClearanceCookie | undefined> {
  try {
    const value = (await win.webContents.executeJavaScript(
      'document.cookie',
      true
    )) as unknown;

    if (typeof value !== 'string') {
      return undefined;
    }

    const cfClearance = parseDocumentCookie(value, cfClearanceCookie);
    if (cfClearance === undefined) {
      return undefined;
    }

    return {
      value: cfClearance,
      domain: 'lista.shinden.pl',
      path: '/'
    };
  } catch (error: unknown) {
    console.warn('Unable to read Shinden verification document.cookie', error);
    return undefined;
  }
}

function parseDocumentCookie(cookieHeader: string, name: string) {
  const needle = `${name}=`;
  for (const part of cookieHeader.split(';')) {
    const cookie = part.trim();
    if (cookie.startsWith(needle)) {
      return cookie.slice(needle.length);
    }
  }

  return undefined;
}

function configureShindenRequestHeaders(
  browserSession: Electron.Session,
  userAgent: string
) {
  browserSession.webRequest.onBeforeSendHeaders(
    shindenRequestFilter,
    (details, callback) => {
      const requestHeaders = { ...details.requestHeaders };
      setHeader(requestHeaders, 'User-Agent', userAgent);
      setHeader(requestHeaders, 'Accept-Language', acceptLanguages);
      setHeader(requestHeaders, 'Sec-CH-UA', secChUaHeader());
      setHeader(requestHeaders, 'Sec-CH-UA-Mobile', '?0');
      setHeader(requestHeaders, 'Sec-CH-UA-Platform', secChUaPlatformHeader());
      callback({ requestHeaders });
    }
  );
}

function setHeader(
  headers: Record<string, string | string[]>,
  name: string,
  value: string
) {
  const lowerName = name.toLowerCase();
  for (const key of Object.keys(headers)) {
    if (key.toLowerCase() === lowerName) {
      delete headers[key];
    }
  }

  headers[name] = value;
}

async function browserWindowUserAgent(win: BrowserWindow) {
  try {
    const value = (await win.webContents.executeJavaScript(
      'navigator.userAgent',
      true
    )) as unknown;

    if (typeof value === 'string' && value.trim() !== '') {
      return value;
    }
  } catch (error: unknown) {
    console.warn(
      'Unable to read Shinden verification navigator.userAgent',
      error
    );
  }

  return win.webContents.getUserAgent();
}

function uniqueCookies(cookies: Electron.Cookie[]) {
  const byKey = new Map<string, Electron.Cookie>();
  for (const cookie of cookies) {
    const partitionKey = cookiePartitionKey(cookie);
    byKey.set(
      `${partitionKey}\n${cookie.domain ?? ''}\n${cookie.path ?? ''}\n${cookie.name}`,
      cookie
    );
  }

  return [...byKey.values()];
}

function cookiePartitionKey(cookie: Electron.Cookie) {
  return (
    (cookie as Electron.Cookie & { partitionKey?: string }).partitionKey ?? ''
  );
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

function isShindenCookieDomain(domain: string) {
  const normalized = domain.trim().replace(/^\./, '').toLowerCase();
  return (
    normalized === 'lista.shinden.pl' ||
    normalized === 'shinden.pl' ||
    normalized.endsWith('.shinden.pl')
  );
}

function summarizeCookies(cookies: Electron.Cookie[]) {
  return cookies
    .filter((cookie) => isShindenCookieDomain(cookie.domain ?? ''))
    .map((cookie) => ({
      name: cookie.name,
      domain: cookie.domain,
      path: cookie.path,
      secure: cookie.secure,
      httpOnly: cookie.httpOnly,
      expirationDate: cookie.expirationDate,
      partitionKey: cookiePartitionKey(cookie) || undefined,
      valueLength: cookie.value.length
    }));
}

function chromiumUserAgent() {
  const defaultUserAgent = session.defaultSession.getUserAgent();
  const chromeUserAgent = defaultUserAgent
    .replace(/\sElectron\/\S+/g, '')
    .trim();

  if (/\bChrome\/\S+\s+Safari\/\S+/.test(chromeUserAgent)) {
    return chromeUserAgent;
  }

  const chromeVersion = process.versions.chrome;
  const platform =
    process.platform === 'win32'
      ? 'Windows NT 10.0; Win64; x64'
      : process.platform === 'darwin'
        ? 'Macintosh; Intel Mac OS X 10_15_7'
        : 'X11; Linux x86_64';

  return `Mozilla/5.0 (${platform}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/${chromeVersion} Safari/537.36`;
}

function chromeMajorVersion() {
  return process.versions.chrome.split('.', 1)[0] ?? '120';
}

function secChUaHeader() {
  const version = chromeMajorVersion();
  return `"Chromium";v="${version}", "Not A(Brand";v="99", "Google Chrome";v="${version}"`;
}

function secChUaPlatformHeader() {
  const platform =
    process.platform === 'win32'
      ? 'Windows'
      : process.platform === 'darwin'
        ? 'macOS'
        : 'Linux';

  return `"${platform}"`;
}
