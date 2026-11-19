import { logFailure } from './diagnostics.js';

import type { DesktopIpcError, DesktopIpcResult } from './ipcTypes.js';

export class DesktopIpcFailure extends Error {
  readonly code: DesktopIpcError;

  constructor(code: DesktopIpcError, message: string) {
    super(message);
    this.code = code;
    this.name = 'DesktopIpcFailure';
  }
}

export async function ipcResult<T>(
  fallback: DesktopIpcError,
  run: () => Promise<T>
): Promise<DesktopIpcResult<T>> {
  try {
    return { ok: true, value: await run() };
  } catch (error) {
    const code = error instanceof DesktopIpcFailure ? error.code : fallback;
    if (code !== 'cancelled') {
      logFailure(`Desktop IPC failed (${code})`, error);
    }
    return { ok: false, error: code };
  }
}
