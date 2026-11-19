import { invoke } from '@tauri-apps/api/core';
import { AppFailure, ErrorKind } from './errors';
import { isTauriRuntime } from './runtime';

export function reportFailure(operation: string, error: unknown) {
  if (
    error instanceof AppFailure &&
    (error.reported || error.kind === ErrorKind.CANCELLED)
  ) {
    return;
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
  const detail = (causes.join('\nCaused by: ') || 'No error details')
    .replace(/(https?:\/\/[^\s?#"'<>]+)[?#][^\s"'<>]*/gi, '$1')
    .replace(
      /\b(cf_clearance|sess_shinden|authorization|set-cookie)\s*[:=]\s*[^\r\n;]+/gi,
      '$1=[redacted]'
    )
    .slice(0, 5000);

  const send = globalThis.shindenToAnilist?.reportFailure;
  if (send !== undefined) {
    send(operation, detail);
  } else if (isTauriRuntime()) {
    void invoke('report_renderer_failure', { operation, detail }).catch(() => {
      console.error(operation, detail);
    });
  } else {
    console.error(operation, detail);
  }
}

export function installRendererDiagnostics() {
  const onError = (event: ErrorEvent) =>
    reportFailure('uncaught_exception', event.error ?? event.message);
  const onRejection = (event: PromiseRejectionEvent) =>
    reportFailure('unhandled_rejection', event.reason);

  window.addEventListener('error', onError);
  window.addEventListener('unhandledrejection', onRejection);
  return () => {
    window.removeEventListener('error', onError);
    window.removeEventListener('unhandledrejection', onRejection);
  };
}
