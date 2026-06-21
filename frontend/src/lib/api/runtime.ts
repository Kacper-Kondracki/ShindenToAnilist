import {
  ConnectError,
  createClient,
  decodeBinaryHeader
} from '@connectrpc/connect';
import { createGrpcWebTransport } from '@connectrpc/connect-web';
import { invoke, isTauri as isTauriApi } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import {
  AppErrorSchema,
  ErrorKind,
  type AppError
} from '../gen/shinden_to_anilist/v1/error_pb';
import { ShindenToAnilistService } from '../gen/shinden_to_anilist/v1/service_pb';

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

type RuntimeHttpError = {
  message: string;
  url: string;
  status: number;
};

type RuntimeAppError = {
  kind: ErrorKind;
  message: string;
  http: RuntimeHttpError | null;
};

export class BackendError extends Error {
  constructor(
    message: string,
    readonly appError: RuntimeAppError | null = null
  ) {
    super(message);
    this.name = 'BackendError';
  }
}

const fallbackPaths = {
  base: '/tmp',
  database: '/tmp/shinden-to-anilist-database.jsonl',
  export: '/tmp/shinden-to-anilist-export.xml'
};

export const databasePath =
  globalThis.shindenToAnilist?.paths.database ?? fallbackPaths.database;
export const exportPath =
  globalThis.shindenToAnilist?.paths.export ?? fallbackPaths.export;

export const appPathsPromise = resolveAppPaths();
const clientPromise = isTauriRuntime() ? null : createAppClient();

type AppClient = Awaited<ReturnType<typeof createAppClient>>;

async function createAppClient() {
  const transport = createGrpcWebTransport({
    baseUrl: await resolveGrpcBaseUrl()
  });

  return createClient(ShindenToAnilistService, transport);
}

async function resolveGrpcBaseUrl() {
  const getGrpcBaseUrl = globalThis.shindenToAnilist?.getGrpcBaseUrl;

  if (getGrpcBaseUrl !== undefined) {
    return await getGrpcBaseUrl();
  }

  return (
    import.meta.env.VITE_SHINDEN_TO_ANILIST_GRPC_BASE_URL ??
    'http://127.0.0.1:45187'
  );
}

async function resolveAppPaths(): Promise<AppPaths> {
  if (globalThis.shindenToAnilist?.paths !== undefined) {
    return globalThis.shindenToAnilist.paths;
  }

  if (isTauriRuntime()) {
    return await callTauri<AppPaths>('app_paths');
  }

  return fallbackPaths;
}

export function isTauriRuntime() {
  return isTauriApi() || globalThis.__TAURI_INTERNALS__ !== undefined;
}

export async function selectExportPath(defaultPath: string) {
  const selectPath = globalThis.shindenToAnilist?.selectExportPath;

  if (selectPath !== undefined) {
    return selectPath({ defaultPath });
  }

  if (isTauriRuntime()) {
    return await save({
      title: 'Wybierz plik eksportu',
      defaultPath,
      filters: [
        { name: 'XML', extensions: ['xml'] },
        { name: 'Wszystkie pliki', extensions: ['*'] }
      ]
    });
  }

  return defaultPath;
}

export async function openShindenCloudflareVerification() {
  const openVerification =
    globalThis.shindenToAnilist?.openShindenCloudflareVerification;

  if (openVerification !== undefined) {
    return await openVerification();
  }

  if (isTauriRuntime()) {
    return await callTauri<ShindenCloudflareClearance>(
      'open_shinden_cloudflare_verification'
    );
  }

  throw new Error(
    'Okno weryfikacji Cloudflare nie jest dostępne w tym trybie.'
  );
}

export async function callRpc<T>(run: (client: AppClient) => Promise<T>) {
  if (clientPromise === null) {
    throw new Error('Klient gRPC nie jest dostępny w trybie Tauri');
  }

  try {
    return await run(await clientPromise);
  } catch (error) {
    throw normalizeRpcError(error);
  }
}

export async function callTauri<T>(
  command: string,
  args?: Record<string, unknown>
) {
  try {
    return await invoke<T>(command, args ?? {});
  } catch (error) {
    throw normalizeTauriError(error);
  }
}

function normalizeRpcError(error: unknown) {
  const connectError = ConnectError.from(error);
  const detail = appErrorFromConnectError(connectError);
  const appError = detail === null ? null : runtimeAppErrorFromProto(detail);
  return new BackendError(
    detail !== null
      ? appErrorMessage(detail)
      : transportErrorMessage(connectError.rawMessage),
    appError
  );
}

function normalizeTauriError(error: unknown) {
  if (error instanceof Error) {
    return new BackendError(userFacingErrorMessage(error.message));
  }

  if (typeof error === 'string') {
    return new BackendError(userFacingErrorMessage(error));
  }

  const appError = runtimeAppErrorFromTauri(error);
  if (appError !== null) {
    return new BackendError(appErrorMessageFromRuntime(appError), appError);
  }

  if (isTauriCommandError(error)) {
    return new BackendError(userFacingErrorMessage(error.message));
  }

  return new BackendError('Nie udało się wykonać polecenia Tauri');
}

function appErrorMessage(detail: AppError) {
  return appErrorMessageFromRuntime(runtimeAppErrorFromProto(detail));
}

function appErrorMessageFromRuntime(detail: RuntimeAppError) {
  if (detail.kind === ErrorKind.SHINDEN_HTTP && detail.http !== null) {
    const status = detail.http.status;
    const statusMessage = status === 0 ? '' : ` Kod odpowiedzi: ${status}.`;

    return `Shinden zwrócił nieoczekiwaną odpowiedź.${statusMessage}`;
  }

  return userFacingErrorMessage(detail.message);
}

export function isShindenCloudflareChallengeError(error: unknown) {
  if (error instanceof BackendError && error.appError !== null) {
    return (
      error.appError.kind === ErrorKind.SHINDEN_HTTP &&
      error.appError.http?.message
        .toLowerCase()
        .includes('cf-mitigated: challenge') === true
    );
  }

  const message =
    error instanceof Error
      ? error.message
      : typeof error === 'string'
        ? error
        : '';

  return message.toLowerCase().includes('cf-mitigated: challenge');
}

function userFacingErrorMessage(message: string) {
  const shindenHttpMatch = message.match(
    /shinden api returned http ([^;]+?)(?: for [^;]+)?;/i
  );
  if (shindenHttpMatch !== null) {
    return `Shinden zwrócił nieoczekiwaną odpowiedź. Kod odpowiedzi: ${shindenHttpMatch[1]}.`;
  }

  return message;
}

function transportErrorMessage(message: string) {
  const normalized = message.toLowerCase();
  if (
    normalized.includes('failed to fetch') ||
    normalized.includes('fetch failed') ||
    normalized.includes('networkerror') ||
    normalized.includes('load failed')
  ) {
    return 'Nie udało się połączyć z usługą aplikacji. Sprawdź, czy proces pomocniczy działa.';
  }

  if (normalized.includes('aborted') || normalized.includes('cancelled')) {
    return 'Operacja została anulowana.';
  }

  if (message.trim() === '') {
    return 'Nie udało się wykonać operacji.';
  }

  return userFacingErrorMessage(message);
}

function appErrorFromConnectError(error: ConnectError) {
  const [detail] = error.findDetails(AppErrorSchema);
  if (detail !== undefined) {
    return detail;
  }

  const rawDetails = error.metadata.get('grpc-status-details-bin');
  if (rawDetails === null) {
    return null;
  }

  try {
    return decodeBinaryHeader(rawDetails, AppErrorSchema);
  } catch {
    return null;
  }
}

function runtimeAppErrorFromProto(error: AppError): RuntimeAppError {
  return {
    kind: error.kind,
    message: error.message,
    http:
      error.details.case === 'http'
        ? {
            message: error.details.value.message,
            url: error.details.value.url,
            status: error.details.value.status
          }
        : null
  };
}

function runtimeAppErrorFromTauri(error: unknown): RuntimeAppError | null {
  if (!isTauriCommandError(error) || !isTauriAppError(error.appError)) {
    return null;
  }

  return {
    kind: error.appError.kind,
    message: error.appError.message,
    http: isTauriHttpError(error.appError.http) ? error.appError.http : null
  };
}

function isTauriCommandError(
  error: unknown
): error is { message: string; appError?: unknown } {
  return (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    typeof error.message === 'string'
  );
}

function isTauriAppError(
  error: unknown
): error is { kind: ErrorKind; message: string; http?: unknown } {
  return (
    typeof error === 'object' &&
    error !== null &&
    'kind' in error &&
    typeof error.kind === 'number' &&
    'message' in error &&
    typeof error.message === 'string'
  );
}

function isTauriHttpError(error: unknown): error is RuntimeHttpError {
  return (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    typeof error.message === 'string' &&
    'url' in error &&
    typeof error.url === 'string' &&
    'status' in error &&
    typeof error.status === 'number'
  );
}
