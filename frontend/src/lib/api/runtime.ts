import { Code, ConnectError, createClient } from '@connectrpc/connect';
import { createGrpcWebTransport } from '@connectrpc/connect-web';
import { invoke, isTauri as isTauriApi } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import {
  AppErrorSchema,
  ErrorKind
} from '../gen/shinden_to_anilist/v1/error_pb';
import { ShindenToAnilistService } from '../gen/shinden_to_anilist/v1/service_pb';
import { AppFailure } from './errors';

type AppPaths = {
  base: string;
  database: string;
  export: string;
};

export type ShindenCloudflareClearance = {
  userAgent: string;
  cfClearance: string;
  domain: string;
  path: string;
  expiresUnixSeconds?: number;
  capturedAtMs: number;
};

export type ShindenCloudflareVerificationMode = 'clearance' | 'autocloseTest';

export type ShindenCloudflareVerificationOptions = {
  mode?: ShindenCloudflareVerificationMode;
};

const fallbackPaths = {
  base: '/tmp',
  database: '/tmp/shinden-to-anilist-database.jsonl',
  export: '/tmp/shinden-to-anilist-export.xml'
};

let rpcClient: AppClient | undefined;

type AppClient = Awaited<ReturnType<typeof createAppClient>>;

async function createAppClient() {
  const transport = createGrpcWebTransport({
    baseUrl: await resolveGrpcBaseUrl(),
    fetch: async (input, init) => {
      try {
        return await globalThis.fetch(input, init);
      } catch (cause) {
        if (init?.signal?.aborted) throw cause;
        throw new ConnectError(
          'gRPC network request failed',
          Code.Unavailable,
          undefined,
          undefined,
          cause
        );
      }
    }
  });

  return createClient(ShindenToAnilistService, transport);
}

async function resolveGrpcBaseUrl() {
  const getGrpcBaseUrl = globalThis.shindenToAnilist?.getGrpcBaseUrl;

  if (getGrpcBaseUrl !== undefined) {
    return await callDesktop(getGrpcBaseUrl, ErrorKind.SERVICE_UNAVAILABLE);
  }

  return (
    import.meta.env.VITE_SHINDEN_TO_ANILIST_GRPC_BASE_URL ??
    'http://127.0.0.1:45187'
  );
}

export async function resolveAppPaths(): Promise<AppPaths> {
  const getAppPaths = globalThis.shindenToAnilist?.getAppPaths;
  if (getAppPaths !== undefined) {
    return callDesktop(getAppPaths, ErrorKind.APP_PATHS_FAILED);
  }

  if (isTauriRuntime()) {
    return await callTauri<AppPaths>('app_paths');
  }

  return fallbackPaths;
}

export function isTauriRuntime() {
  return isTauriApi() || globalThis.__TAURI_INTERNALS__ !== undefined;
}

export async function openDesktopLink(url: string) {
  const openExternalUrl = globalThis.shindenToAnilist?.openExternalUrl;
  if (openExternalUrl !== undefined) {
    await callDesktop(() => openExternalUrl(url), ErrorKind.UNKNOWN);
  } else if (isTauriRuntime()) {
    const { openUrl } = await import('@tauri-apps/plugin-opener');
    await openUrl(url);
  } else {
    window.open(url, '_blank', 'noopener,noreferrer');
  }
}

export async function selectExportPath(defaultPath: string) {
  const selectPath = globalThis.shindenToAnilist?.selectExportPath;

  if (selectPath !== undefined) {
    return callDesktop(
      () => selectPath({ defaultPath }),
      ErrorKind.EXPORT_DIALOG_FAILED
    );
  }

  if (isTauriRuntime()) {
    return await callNative(
      () =>
        save({
          title: 'Wybierz plik eksportu',
          defaultPath,
          filters: [
            { name: 'XML', extensions: ['xml'] },
            { name: 'Wszystkie pliki', extensions: ['*'] }
          ]
        }),
      ErrorKind.EXPORT_DIALOG_FAILED
    );
  }

  return defaultPath;
}

export async function openShindenCloudflareVerification(
  options: ShindenCloudflareVerificationOptions = {}
) {
  const openVerification =
    globalThis.shindenToAnilist?.openShindenCloudflareVerification;

  if (openVerification !== undefined) {
    return callDesktop(
      () => openVerification(options),
      ErrorKind.VERIFICATION_WINDOW_FAILED
    );
  }

  if (isTauriRuntime()) {
    return callTauri<ShindenCloudflareClearance>(
      'open_shinden_cloudflare_verification',
      { options }
    );
  }

  throw new AppFailure(ErrorKind.VERIFICATION_WINDOW_UNAVAILABLE);
}

export async function callRpc<T>(run: (client: AppClient) => Promise<T>) {
  if (isTauriRuntime()) {
    throw new Error('gRPC client called in Tauri');
  }

  try {
    rpcClient ??= await createAppClient();
    return await run(rpcClient);
  } catch (error) {
    if (!(error instanceof ConnectError)) {
      throw error;
    }

    const [detail] = error.findDetails(AppErrorSchema);
    const transportKind = {
      [Code.Canceled]: ErrorKind.CANCELLED,
      [Code.DeadlineExceeded]: ErrorKind.DEADLINE_EXCEEDED,
      [Code.Unavailable]: ErrorKind.SERVICE_UNAVAILABLE
    } as Partial<Record<Code, ErrorKind>>;
    throw new AppFailure(
      detail?.kind || transportKind[error.code] || ErrorKind.UNKNOWN,
      { cause: error },
      detail?.httpStatus,
      detail !== undefined
    );
  }
}

export async function callTauri<T>(
  command: string,
  args?: Record<string, unknown>
) {
  try {
    return await invoke<T>(command, args ?? {});
  } catch (error) {
    if (
      typeof error === 'object' &&
      error !== null &&
      'kind' in error &&
      typeof error.kind === 'number' &&
      Object.hasOwn(ErrorKind, error.kind)
    ) {
      const httpStatus =
        'httpStatus' in error && typeof error.httpStatus === 'number'
          ? error.httpStatus
          : undefined;
      throw new AppFailure(error.kind, { cause: error }, httpStatus, true);
    }

    throw new AppFailure(ErrorKind.UNKNOWN, { cause: error });
  }
}

const nativeErrorKinds = {
  cancelled: ErrorKind.CANCELLED,
  verificationAlreadyOpen: ErrorKind.VERIFICATION_WINDOW_ALREADY_OPEN,
  verificationFailed: ErrorKind.VERIFICATION_WINDOW_FAILED,
  sidecarUnavailable: ErrorKind.SERVICE_UNAVAILABLE,
  exportDialogFailed: ErrorKind.EXPORT_DIALOG_FAILED,
  appPathsFailed: ErrorKind.APP_PATHS_FAILED,
  externalLinkFailed: ErrorKind.UNKNOWN
};

async function callDesktop<T>(
  run: () => Promise<DesktopIpcResult<T>>,
  fallback: ErrorKind
): Promise<T> {
  const response = await callNative(run, fallback);
  if (!response.ok) {
    throw new AppFailure(
      nativeErrorKinds[response.error] ?? fallback,
      undefined,
      undefined,
      true
    );
  }
  return response.value;
}

async function callNative<T>(
  run: () => Promise<T>,
  fallback: ErrorKind
): Promise<T> {
  try {
    return await run();
  } catch (cause) {
    throw new AppFailure(fallback, { cause });
  }
}
