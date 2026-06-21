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

  try {
    if (openVerification !== undefined) {
      return await openVerification();
    }

    if (isTauriRuntime()) {
      return await callTauri<ShindenCloudflareClearance>(
        'open_shinden_cloudflare_verification'
      );
    }
  } catch (error) {
    throw normalizeFrontendError(
      error,
      'Nie udało się otworzyć okna weryfikacji Cloudflare.'
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

  return (
    userMessageForErrorKind(detail.kind, detail.http?.status ?? 0) ??
    userFacingErrorMessage(detail.message)
  );
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

export function toUserFacingErrorMessage(
  error: unknown,
  fallback = 'Nie udało się wykonać operacji.'
) {
  if (error instanceof BackendError) {
    return error.appError === null
      ? userFacingErrorMessage(error.message)
      : appErrorMessageFromRuntime(error.appError);
  }

  if (error instanceof Error) {
    return userFacingErrorMessage(error.message);
  }

  if (typeof error === 'string') {
    return userFacingErrorMessage(error);
  }

  return fallback;
}

function normalizeFrontendError(error: unknown, fallback: string) {
  if (error instanceof BackendError) {
    return error;
  }

  return new BackendError(toUserFacingErrorMessage(error, fallback));
}

function userFacingErrorMessage(message: string) {
  const trimmed = message.trim();
  const normalized = trimmed.toLowerCase();
  const shindenHttpMatch = message.match(
    /shinden api returned http ([^;]+?)(?: for [^;]+)?;/i
  );
  if (shindenHttpMatch !== null) {
    return `Shinden zwrócił nieoczekiwaną odpowiedź. Kod odpowiedzi: ${shindenHttpMatch[1]}.`;
  }

  if (normalized.includes('cf-mitigated: challenge')) {
    return 'Shinden wymaga weryfikacji Cloudflare.';
  }

  if (normalized.includes('private') || normalized.includes('prywat')) {
    return 'Lista użytkownika Shinden jest prywatna albo niedostępna.';
  }

  if (
    normalized.includes('not found') ||
    normalized.includes('nie znaleziono')
  ) {
    return 'Nie znaleziono listy użytkownika Shinden.';
  }

  const clearanceMessage = cloudflareClearanceMessage(normalized);
  if (clearanceMessage !== null) {
    return clearanceMessage;
  }

  if (normalized.includes('deadline exceeded')) {
    return 'Operacja trwała zbyt długo. Spróbuj ponownie za chwilę.';
  }

  if (normalized.includes('aborted') || normalized.includes('cancelled')) {
    return 'Operacja została anulowana.';
  }

  if (normalized.includes('permission denied')) {
    return 'Brak uprawnień do wykonania operacji.';
  }

  if (
    normalized.includes('failed to fetch') ||
    normalized.includes('fetch failed') ||
    normalized.includes('networkerror') ||
    normalized.includes('load failed') ||
    normalized.includes('connection refused') ||
    normalized.includes('connection reset') ||
    normalized.includes('connection closed')
  ) {
    return 'Nie udało się połączyć z usługą aplikacji. Sprawdź, czy proces pomocniczy działa.';
  }

  if (
    normalized.includes('invalid argument') ||
    normalized.includes('invalid_argument')
  ) {
    return 'Aplikacja otrzymała nieprawidłowe dane.';
  }

  if (
    normalized.includes('internal') ||
    normalized.includes('unknown') ||
    normalized.includes('failed')
  ) {
    return 'Operacja nie powiodła się.';
  }

  if (looksLikeTechnicalEnglishMessage(trimmed)) {
    return 'Operacja nie powiodła się.';
  }

  if (trimmed === '') {
    return 'Nie udało się wykonać operacji.';
  }

  return trimmed;
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

  const clearanceMessage = cloudflareClearanceMessage(normalized);
  if (clearanceMessage !== null) {
    return clearanceMessage;
  }

  if (message.trim() === '') {
    return 'Nie udało się wykonać operacji.';
  }

  return userFacingErrorMessage(message);
}

function cloudflareClearanceMessage(normalizedMessage: string) {
  if (normalizedMessage.includes('missing cloudflare browser user agent')) {
    return 'Nie udało się odczytać danych przeglądarki z okna weryfikacji Cloudflare.';
  }

  if (normalizedMessage.includes('cloudflare browser user agent is too long')) {
    return 'Dane przeglądarki z okna weryfikacji Cloudflare są zbyt długie.';
  }

  if (
    normalizedMessage.includes('missing cf_clearance cookie') ||
    normalizedMessage.includes('did not produce cf_clearance')
  ) {
    return 'Nie udało się odczytać ciasteczka Cloudflare z okna weryfikacji.';
  }

  if (normalizedMessage.includes('cf_clearance cookie is too long')) {
    return 'Ciasteczko Cloudflare z okna weryfikacji jest zbyt długie.';
  }

  if (
    normalizedMessage.includes(
      'cf_clearance cookie contains invalid characters'
    )
  ) {
    return 'Ciasteczko Cloudflare z okna weryfikacji ma nieprawidłowy format.';
  }

  if (
    normalizedMessage.includes(
      'cf_clearance cookie domain is not a shinden domain'
    )
  ) {
    return 'Ciasteczko Cloudflare pochodzi z nieobsługiwanej domeny.';
  }

  if (normalizedMessage.includes('cf_clearance cookie path is invalid')) {
    return 'Ciasteczko Cloudflare ma nieprawidłową ścieżkę.';
  }

  if (normalizedMessage.includes('cf_clearance cookie is expired')) {
    return 'Weryfikacja Cloudflare wygasła. Otwórz okno weryfikacji ponownie.';
  }

  if (normalizedMessage.includes('internal shinden cookie url is invalid')) {
    return 'Nie udało się przygotować ciasteczka Cloudflare dla Shindena.';
  }

  if (normalizedMessage.includes('failed to build shinden http client')) {
    return 'Nie udało się zastosować weryfikacji Cloudflare dla Shindena.';
  }

  return null;
}

function looksLikeTechnicalEnglishMessage(message: string) {
  return (
    /\b(error|failed|failure|invalid|missing|unable|cannot|timeout|network|socket|connection|refused|reset|closed|denied|unavailable)\b/i.test(
      message
    ) || /\bERR_[A-Z0-9_]+\b/.test(message)
  );
}

function userMessageForErrorKind(kind: ErrorKind, status: number) {
  const statusMessage = status === 0 ? '' : ` Kod odpowiedzi: ${status}.`;

  switch (kind) {
    case ErrorKind.SHINDEN_LIST_NOT_LOADED:
      return 'Lista Shinden nie została jeszcze wczytana.';
    case ErrorKind.DATABASE_NOT_LOADED:
      return 'Baza danych nie została jeszcze wczytana.';
    case ErrorKind.SHINDEN_IO:
      return 'Nie udało się odczytać danych Shindena.';
    case ErrorKind.SHINDEN_JSON:
      return 'Shinden zwrócił nieprawidłowe dane JSON.';
    case ErrorKind.SHINDEN_HTTP:
      return `Nie udało się połączyć z Shindenem.${statusMessage}`;
    case ErrorKind.DATABASE_IO:
    case ErrorKind.DATABASE_SIDECAR_IO:
      return 'Nie udało się odczytać lub zapisać bazy danych.';
    case ErrorKind.DATABASE_JSON:
    case ErrorKind.DATABASE_SIDECAR_JSON:
      return 'Plik bazy danych ma nieprawidłowy format JSON.';
    case ErrorKind.DATABASE_HTTP:
      return `Nie udało się pobrać informacji o bazie danych.${statusMessage}`;
    case ErrorKind.DATABASE_EMPTY:
      return 'Baza danych jest pusta.';
    case ErrorKind.DATABASE_MISSING_RELEASE_ASSET:
      return 'Nie znaleziono wymaganego pliku bazy danych w wydaniu.';
    case ErrorKind.DATABASE_DIGEST_MISMATCH:
      return 'Pobrana baza danych nie przeszła sprawdzania integralności.';
    case ErrorKind.EXPORT_XML:
      return 'Nie udało się przygotować pliku XML.';
    case ErrorKind.EXPORT_OUT_OF_INDEX:
      return 'Nie udało się wyeksportować jednej z wybranych pozycji.';
    case ErrorKind.EXPORT_IO:
      return 'Nie udało się zapisać pliku eksportu.';
    case ErrorKind.ANIME_ZONE_IO:
      return 'Nie udało się odczytać danych AnimeZone.';
    case ErrorKind.ANIME_ZONE_HTTP:
      return `Nie udało się połączyć z AnimeZone.${statusMessage}`;
    case ErrorKind.ANIME_ZONE_PARSE:
      return 'Nie udało się odczytać listy AnimeZone. Strona mogła zmienić format danych.';
    case ErrorKind.ANIME_ZONE_RETRY_EXHAUSTED:
      return 'Nie udało się pobrać listy AnimeZone po kilku próbach.';
    case ErrorKind.OGLADAJ_ANIME_IO:
      return 'Nie udało się odczytać danych Oglądaj Anime.';
    case ErrorKind.OGLADAJ_ANIME_HTTP:
      return `Nie udało się połączyć z Oglądaj Anime.${statusMessage}`;
    case ErrorKind.OGLADAJ_ANIME_PARSE:
      return 'Nie udało się odczytać listy Oglądaj Anime. Strona mogła zmienić format danych.';
    case ErrorKind.OGLADAJ_ANIME_RETRY_EXHAUSTED:
      return 'Nie udało się pobrać listy Oglądaj Anime po kilku próbach.';
    case ErrorKind.UNKNOWN:
    case ErrorKind.UNSPECIFIED:
      return 'Operacja nie powiodła się.';
    case ErrorKind.SHINDEN_API:
      return null;
  }

  return null;
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
