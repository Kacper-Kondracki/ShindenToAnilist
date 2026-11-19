import {
  appendFileSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
  statSync
} from 'node:fs';
import { homedir } from 'node:os';
import { isAbsolute, join } from 'node:path';

const maxLogBytes = 1024 * 1024;
let logPath: string | undefined;
let reportedWriteFailure = false;
const loggedErrors = new WeakSet<Error>();

function logDirectory() {
  if (process.platform === 'win32') {
    return join(
      process.env.LOCALAPPDATA || join(homedir(), 'AppData', 'Local'),
      'ShindenToAnilist',
      'logs'
    );
  }
  if (process.platform === 'darwin') {
    return join(homedir(), 'Library', 'Logs', 'ShindenToAnilist');
  }

  const state = process.env.XDG_STATE_HOME;
  return join(
    state && isAbsolute(state) ? state : join(homedir(), '.local', 'state'),
    'ShindenToAnilist',
    'logs'
  );
}

export function startDiagnostics(version: string) {
  let revision = 'unknown';
  try {
    revision = JSON.parse(
      readFileSync(new URL('./build-info.json', import.meta.url), 'utf8')
    ).revision;
  } catch {
    // A development build may lack build metadata.
  }

  logEvent('INFO', 'session started', {
    revision,
    version,
    os: process.platform,
    architecture: process.arch,
    electron: process.versions.electron,
    chromium: process.versions.chrome
  });
}

export function logEvent(
  level: 'INFO' | 'WARN' | 'ERROR',
  event: string,
  fields: Record<string, string | number | undefined> = {}
) {
  const safeFields = Object.fromEntries(
    Object.entries(fields).map(([name, value]) => [
      name,
      typeof value === 'string' ? safeDiagnostic(value) : value
    ])
  );
  const entry =
    JSON.stringify({
      time: new Date().toISOString(),
      level,
      event,
      ...safeFields
    }) + '\n';
  try {
    if (logPath === undefined) {
      const dir = logDirectory();
      mkdirSync(dir, { recursive: true, mode: 0o700 });
      logPath = join(dir, 'electron.log');
    }
    if (
      (statSync(logPath, { throwIfNoEntry: false })?.size ?? 0) +
        Buffer.byteLength(entry) >
      maxLogBytes
    ) {
      const previous = logPath + '.1';
      if (process.platform === 'win32') rmSync(previous, { force: true });
      renameSync(logPath, previous);
    }
    appendFileSync(logPath, entry, { mode: 0o600 });
  } catch (error) {
    if (!reportedWriteFailure) {
      reportedWriteFailure = true;
      console.error('Could not write application log', error);
    }
    process.stderr.write(entry);
  }
}

export function logFailure(event: string, error: unknown) {
  if (error instanceof Error) {
    if (loggedErrors.has(error)) return;
    loggedErrors.add(error);
  }
  const causes: string[] = [];
  let cause = error;
  for (let depth = 0; cause !== undefined && depth < 4; depth += 1) {
    if (!(cause instanceof Error)) {
      if (typeof cause === 'string') causes.push(cause);
      break;
    }
    causes.push(cause.stack ?? cause.message);
    cause = cause.cause;
  }
  logEvent('ERROR', event, { detail: causes.join('\nCaused by: ') });
}

function safeDiagnostic(value: string) {
  return value
    .replace(/(https?:\/\/[^\s?#"'<>]+)[?#][^\s"'<>]*/gi, '$1')
    .replace(
      /\b(cf_clearance|sess_shinden|authorization|set-cookie)\s*[:=]\s*[^\r\n;]+/gi,
      '$1=[redacted]'
    )
    .slice(0, 5000);
}
