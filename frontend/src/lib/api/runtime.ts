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
  return new Error(
    detail !== null
      ? appErrorMessage(detail)
      : transportErrorMessage(connectError.rawMessage)
  );
}

function normalizeTauriError(error: unknown) {
  if (error instanceof Error) {
    return new Error(userFacingErrorMessage(error.message));
  }

  if (typeof error === 'string') {
    return new Error(userFacingErrorMessage(error));
  }

  return new Error('Nie udało się wykonać polecenia Tauri');
}

function appErrorMessage(detail: AppError) {
  if (
    detail.kind === ErrorKind.SHINDEN_HTTP &&
    detail.details.case === 'http'
  ) {
    const status = detail.details.value.status;
    const statusMessage = status === 0 ? '' : ` Kod odpowiedzi: ${status}.`;

    return `Shinden zwrócił nieoczekiwaną odpowiedź.${statusMessage}`;
  }

  return userFacingErrorMessage(detail.message);
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
