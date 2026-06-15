import { create } from '@bufbuild/protobuf';
import {
  ConnectError,
  createClient,
  decodeBinaryHeader
} from '@connectrpc/connect';
import { createGrpcWebTransport } from '@connectrpc/connect-web';
import { invoke, isTauri as isTauriApi } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import {
  AnimeListSortedBy,
  type Date as ProtoDate
} from '../gen/shinden_to_anilist/v1/common_pb';
import { AppErrorSchema } from '../gen/shinden_to_anilist/v1/error_pb';
import { AnimeIdPairSchema } from '../gen/shinden_to_anilist/v1/export_pb';
import type {
  DatabaseEntry as ProtoDatabaseEntry,
  DatabaseMetadata,
  DatabaseReleaseInfo,
  DatabaseUpdateCheck
} from '../gen/shinden_to_anilist/v1/database_pb';
import type {
  MatchResult as ProtoMatchResult,
  ShindenMatchResult as ProtoShindenMatchResult
} from '../gen/shinden_to_anilist/v1/matching_pb';
import { SearchOptionsSchema } from '../gen/shinden_to_anilist/v1/search_pb';
import {
  CheckDatabaseUpdateRequestSchema,
  DownloadDatabaseRequestSchema,
  ExportXmlRequestSchema,
  FetchShindenListRequestSchema,
  FuzzyMatchRequestSchema,
  FuzzySearchRequestSchema,
  GetDatabaseFullRequestSchema,
  GetDatabaseMetadataRequestSchema,
  GetShindenFullRequestSchema,
  GetShindenIdsRequestSchema,
  LoadDatabaseRequestSchema,
  MatchShindenListRequestSchema,
  ShindenToAnilistService
} from '../gen/shinden_to_anilist/v1/service_pb';
import type { ShindenEntry as ProtoShindenEntry } from '../gen/shinden_to_anilist/v1/shinden_pb';
import type {
  DatabaseEntry,
  DatabaseInfo,
  ExportResult,
  MatchListResult,
  MatchResult,
  MatchSelection,
  SearchOptions,
  SearchResult,
  ShindenEntry,
  ShindenListIndex
} from '../domain/anime';

type WireNumber = bigint | number;

type AppPaths = {
  base: string;
  database: string;
  export: string;
};

type TauriDate = {
  year: number;
  month: number;
  day: number;
};

type WireDate = ProtoDate | TauriDate | null | undefined;

type AppDatabaseReleaseInfo = Pick<
  DatabaseReleaseInfo,
  'release' | 'sha256'
> & {
  compressedSize?: WireNumber;
};

type AppDatabaseMetadata = {
  lastUpdate?: WireDate;
};

type AppDatabaseUpdateCheck = {
  local?: AppDatabaseReleaseInfo | null;
  remote?: AppDatabaseReleaseInfo | null;
  needsUpdate: boolean;
};

type TauriShindenEntry = {
  id: WireNumber;
  coverId?: number | null;
  title: string;
  animeStatus: number;
  animeType: number;
  premiereDate?: WireDate;
  finishDate?: WireDate;
  episodes?: number | null;
  isFavourite: boolean;
  watchStatus: number;
  watchedEpisodes: number;
  score?: number | null;
};

type TauriDatabaseEntry = {
  id: WireNumber;
  sources: string[];
  title: string;
  animeType: number;
  episodes: number;
  status: number;
  season: number;
  year?: number | null;
  picture: string;
  thumbnail: string;
  duration?: number | null;
  synonyms: string[];
};

type WireMatchResult = ProtoMatchResult | TauriMatchResult;

type TauriMatchResult = {
  id: WireNumber;
  finalScore: number;
};

type TauriShindenMatchResult = {
  shindenId: WireNumber;
  candidates: TauriMatchResult[];
  topCandidates: TauriMatchResult[];
  winner?: TauriMatchResult | null;
};

type WireShindenMatchResult = ProtoShindenMatchResult | TauriShindenMatchResult;

type TauriSearchOptions = {
  limit: number;
  threshold?: number;
};

type TauriFetchShindenListResponse = {
  shindenVersion: WireNumber;
};

type TauriGetShindenIdsResponse = {
  shindenVersion: WireNumber;
  ids: WireNumber[];
};

type TauriGetShindenFullResponse = {
  shindenVersion: WireNumber;
  entries: TauriShindenEntry[];
};

type TauriCheckDatabaseUpdateResponse = {
  status?: AppDatabaseUpdateCheck | null;
};

type TauriDownloadDatabaseResponse = {
  status?: AppDatabaseReleaseInfo | null;
};

type TauriLoadDatabaseResponse = {
  databaseVersion: WireNumber;
};

type TauriGetDatabaseMetadataResponse = {
  metadata?: AppDatabaseMetadata | null;
};

type TauriGetDatabaseFullResponse = {
  databaseVersion: WireNumber;
  entries: TauriDatabaseEntry[];
};

type TauriFuzzySearchResponse = {
  databaseVersion: WireNumber;
  results: Array<{ id: WireNumber; score: number }>;
};

type TauriFuzzyMatchResponse = {
  databaseVersion: WireNumber;
  results: TauriMatchResult[];
};

type TauriMatchShindenListResponse = {
  shindenVersion: WireNumber;
  databaseVersion: WireNumber;
  results: TauriShindenMatchResult[];
};

type TauriExportXmlResponse = {
  shindenVersion: WireNumber;
  path: string;
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

const appPathsPromise = resolveAppPaths();
const clientPromise = isTauriRuntime() ? null : createAppClient();
type AppClient = Awaited<ReturnType<typeof createAppClient>>;

let shindenVersion = 0;
let databaseVersion = 0;

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

function isTauriRuntime() {
  return isTauriApi() || globalThis.__TAURI_INTERNALS__ !== undefined;
}

export function currentVersions() {
  return {
    shinden: shindenVersion,
    database: databaseVersion
  };
}

export async function ensureDatabase() {
  const paths = await appPathsPromise;
  const update = await checkDatabaseUpdate(paths.database);

  let release = update.local ?? update.remote ?? null;
  if (update.needsUpdate || release === null) {
    release = await downloadDatabase(paths.database);
  }

  const [{ databaseVersion: loadedVersion }, metadata] = await Promise.all([
    loadDatabase(paths.database),
    getDatabaseMetadata(paths.database)
  ]);

  return toDatabaseInfo({
    path: paths.database,
    release,
    metadata,
    needsUpdate: update.needsUpdate,
    databaseVersion: loadedVersion
  });
}

export async function fetchShindenList(userId: number) {
  if (isTauriRuntime()) {
    const response = await callTauri<TauriFetchShindenListResponse>(
      'fetch_shinden_list',
      { id: userId }
    );
    observeShindenVersion(response.shindenVersion);
    return { shindenVersion: toNumber(response.shindenVersion) };
  }

  return callRpc(async (client) => {
    const response = await client.fetchShindenList(
      create(FetchShindenListRequestSchema, { id: BigInt(userId) })
    );
    observeShindenVersion(response.shindenVersion);
    return { shindenVersion: toNumber(response.shindenVersion) };
  });
}

export async function getShindenIds(sortedBy = AnimeListSortedBy.URGENCY) {
  if (isTauriRuntime()) {
    const response = await callTauri<TauriGetShindenIdsResponse>(
      'get_shinden_ids',
      { sortedBy }
    );
    observeShindenVersion(response.shindenVersion);

    return {
      shindenVersion: toNumber(response.shindenVersion),
      entryIds: response.ids.map(toNumber)
    };
  }

  return callRpc(async (client): Promise<ShindenListIndex> => {
    const response = await client.getShindenIds(
      create(GetShindenIdsRequestSchema, { sortedBy })
    );
    observeShindenVersion(response.shindenVersion);

    return {
      shindenVersion: toNumber(response.shindenVersion),
      entryIds: response.ids.map(toNumber)
    };
  });
}

export async function getShindenFull() {
  if (isTauriRuntime()) {
    const response =
      await callTauri<TauriGetShindenFullResponse>('get_shinden_full');
    observeShindenVersion(response.shindenVersion);
    return {
      shindenVersion: toNumber(response.shindenVersion),
      entries: response.entries.map(toShindenEntry)
    };
  }

  return callRpc(async (client) => {
    const entries: ShindenEntry[] = [];
    let version = 0n;

    for await (const chunk of client.getShindenFull(
      create(GetShindenFullRequestSchema)
    )) {
      version = chunk.shindenVersion;
      entries.push(...chunk.entries.map(toShindenEntry));
    }

    observeShindenVersion(version);
    return {
      shindenVersion: toNumber(version),
      entries
    };
  });
}

export async function checkDatabaseUpdate(path?: string) {
  const resolvedPath = path ?? (await appPathsPromise).database;

  if (isTauriRuntime()) {
    const response = await callTauri<TauriCheckDatabaseUpdateResponse>(
      'check_database_update',
      { path: resolvedPath }
    );

    if (response.status == null) {
      throw new Error('Brak statusu aktualizacji bazy danych');
    }

    return response.status;
  }

  return callRpc(async (client): Promise<DatabaseUpdateCheck> => {
    const response = await client.checkDatabaseUpdate(
      create(CheckDatabaseUpdateRequestSchema, { path: resolvedPath })
    );

    if (response.status === undefined) {
      throw new Error('Brak statusu aktualizacji bazy danych');
    }

    return response.status;
  });
}

export async function downloadDatabase(path?: string) {
  const resolvedPath = path ?? (await appPathsPromise).database;

  if (isTauriRuntime()) {
    const response = await callTauri<TauriDownloadDatabaseResponse>(
      'download_database',
      { path: resolvedPath }
    );

    if (response.status == null) {
      throw new Error('Brak informacji o pobranej bazie danych');
    }

    return response.status;
  }

  return callRpc(async (client): Promise<DatabaseReleaseInfo> => {
    const response = await client.downloadDatabase(
      create(DownloadDatabaseRequestSchema, { path: resolvedPath })
    );

    if (response.status === undefined) {
      throw new Error('Brak informacji o pobranej bazie danych');
    }

    return response.status;
  });
}

export async function loadDatabase(path?: string) {
  const resolvedPath = path ?? (await appPathsPromise).database;

  if (isTauriRuntime()) {
    const response = await callTauri<TauriLoadDatabaseResponse>(
      'load_database',
      { path: resolvedPath }
    );
    observeDatabaseVersion(response.databaseVersion);
    return { databaseVersion: toNumber(response.databaseVersion) };
  }

  return callRpc(async (client) => {
    const response = await client.loadDatabase(
      create(LoadDatabaseRequestSchema, { path: resolvedPath })
    );
    observeDatabaseVersion(response.databaseVersion);
    return { databaseVersion: toNumber(response.databaseVersion) };
  });
}

export async function getDatabaseMetadata(path?: string) {
  const resolvedPath = path ?? (await appPathsPromise).database;

  if (isTauriRuntime()) {
    const response = await callTauri<TauriGetDatabaseMetadataResponse>(
      'get_database_metadata',
      { path: resolvedPath }
    );

    if (response.metadata == null) {
      throw new Error('Brak metadanych bazy danych');
    }

    return response.metadata;
  }

  return callRpc(async (client): Promise<DatabaseMetadata> => {
    const response = await client.getDatabaseMetadata(
      create(GetDatabaseMetadataRequestSchema, { path: resolvedPath })
    );

    if (response.metadata === undefined) {
      throw new Error('Brak metadanych bazy danych');
    }

    return response.metadata;
  });
}

export async function getDatabaseFull() {
  if (isTauriRuntime()) {
    const response =
      await callTauri<TauriGetDatabaseFullResponse>('get_database_full');
    observeDatabaseVersion(response.databaseVersion);
    return {
      databaseVersion: toNumber(response.databaseVersion),
      entries: response.entries.map(toDatabaseEntry)
    };
  }

  return callRpc(async (client) => {
    const entries: DatabaseEntry[] = [];
    let version = 0n;

    for await (const chunk of client.getDatabaseFull(
      create(GetDatabaseFullRequestSchema)
    )) {
      version = chunk.databaseVersion;
      entries.push(...chunk.entries.map(toDatabaseEntry));
    }

    observeDatabaseVersion(version);
    return {
      databaseVersion: toNumber(version),
      entries
    };
  });
}

export async function fuzzySearch(query: string, options: SearchOptions = {}) {
  if (isTauriRuntime()) {
    const response = await callTauri<TauriFuzzySearchResponse>('fuzzy_search', {
      query,
      options: createTauriSearchOptions(options)
    });
    observeDatabaseVersion(response.databaseVersion);

    return {
      databaseVersion: toNumber(response.databaseVersion),
      items: response.results.map((item) => ({
        id: toNumber(item.id),
        score: item.score
      }))
    };
  }

  return callRpc(async (client): Promise<SearchResult> => {
    const response = await client.fuzzySearch(
      create(FuzzySearchRequestSchema, {
        query,
        options: createSearchOptions(options)
      })
    );
    observeDatabaseVersion(response.databaseVersion);

    return {
      databaseVersion: toNumber(response.databaseVersion),
      items: response.results.map((item) => ({
        id: toNumber(item.id),
        score: item.score
      }))
    };
  });
}

export async function fuzzyMatch(
  query: string,
  options: SearchOptions = {},
  shindenId?: number
) {
  if (isTauriRuntime()) {
    const response = await callTauri<TauriFuzzyMatchResponse>('fuzzy_match', {
      query,
      options: createTauriSearchOptions(options),
      shindenId
    });
    observeDatabaseVersion(response.databaseVersion);

    return {
      databaseVersion: toNumber(response.databaseVersion),
      result: toMatchResult(response.results)
    };
  }

  return callRpc(async (client) => {
    const response = await client.fuzzyMatch(
      create(FuzzyMatchRequestSchema, {
        query,
        options: createSearchOptions(options),
        shindenId: shindenId === undefined ? undefined : BigInt(shindenId)
      })
    );
    observeDatabaseVersion(response.databaseVersion);

    return {
      databaseVersion: toNumber(response.databaseVersion),
      result: toMatchResult(response.results)
    };
  });
}

export async function matchShindenList(options: SearchOptions = {}) {
  if (isTauriRuntime()) {
    const response = await callTauri<TauriMatchShindenListResponse>(
      'match_shinden_list',
      { options: createTauriSearchOptions(options) }
    );
    observeShindenVersion(response.shindenVersion);
    observeDatabaseVersion(response.databaseVersion);
    return toMatchListResult(
      response.results,
      response.shindenVersion,
      response.databaseVersion
    );
  }

  return callRpc(async (client): Promise<MatchListResult> => {
    const entries: ProtoShindenMatchResult[] = [];
    let nextShindenVersion = 0n;
    let nextDatabaseVersion = 0n;

    for await (const chunk of client.matchShindenList(
      create(MatchShindenListRequestSchema, {
        options: createSearchOptions(options)
      })
    )) {
      nextShindenVersion = chunk.shindenVersion;
      nextDatabaseVersion = chunk.databaseVersion;
      entries.push(...chunk.results);
    }

    observeShindenVersion(nextShindenVersion);
    observeDatabaseVersion(nextDatabaseVersion);
    return toMatchListResult(entries, nextShindenVersion, nextDatabaseVersion);
  });
}

export async function exportXml(matches: MatchSelection[], path?: string) {
  const resolvedPath = path ?? (await appPathsPromise).export;

  if (isTauriRuntime()) {
    const selectedPath = await selectExportPath(resolvedPath);

    if (selectedPath === null) {
      return {
        path: resolvedPath,
        exportedCount: 0,
        cancelled: true,
        shindenVersion
      };
    }

    const response = await callTauri<TauriExportXmlResponse>('export_xml', {
      path: selectedPath,
      matches
    });
    observeShindenVersion(response.shindenVersion);

    return {
      path: response.path,
      exportedCount: matches.length,
      cancelled: false,
      shindenVersion: toNumber(response.shindenVersion)
    };
  }

  return callRpc(async (client): Promise<ExportResult> => {
    const selectedPath = await selectExportPath(resolvedPath);

    if (selectedPath === null) {
      return {
        path: resolvedPath,
        exportedCount: 0,
        cancelled: true,
        shindenVersion
      };
    }

    const response = await client.exportXml(
      create(ExportXmlRequestSchema, {
        path: selectedPath,
        matches: matches.map((match) =>
          create(AnimeIdPairSchema, {
            shindenId: BigInt(match.shindenId),
            databaseId: BigInt(match.databaseId)
          })
        )
      })
    );
    observeShindenVersion(response.shindenVersion);

    return {
      path: response.path,
      exportedCount: matches.length,
      cancelled: false,
      shindenVersion: toNumber(response.shindenVersion)
    };
  });
}

async function selectExportPath(defaultPath: string) {
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

async function callRpc<T>(run: (client: AppClient) => Promise<T>) {
  if (clientPromise === null) {
    throw new Error('gRPC client is not available in Tauri runtime');
  }

  try {
    return await run(await clientPromise);
  } catch (error) {
    throw normalizeRpcError(error);
  }
}

async function callTauri<T>(command: string, args?: Record<string, unknown>) {
  try {
    return await invoke<T>(command, args ?? {});
  } catch (error) {
    throw normalizeTauriError(error);
  }
}

function observeShindenVersion(version: WireNumber) {
  const nextVersion = toNumber(version);
  if (nextVersion > shindenVersion) {
    shindenVersion = nextVersion;
  }
}

function observeDatabaseVersion(version: WireNumber) {
  const nextVersion = toNumber(version);
  if (nextVersion > databaseVersion) {
    databaseVersion = nextVersion;
  }
}

function createSearchOptions(options: SearchOptions) {
  return create(SearchOptionsSchema, {
    limit: options.limit ?? 0,
    threshold: options.threshold
  });
}

function createTauriSearchOptions(options: SearchOptions): TauriSearchOptions {
  const tauriOptions: TauriSearchOptions = {
    limit: options.limit ?? 0
  };

  if (options.threshold !== undefined) {
    tauriOptions.threshold = options.threshold;
  }

  return tauriOptions;
}

function toDatabaseInfo(input: {
  path: string;
  release: AppDatabaseReleaseInfo | null;
  metadata: AppDatabaseMetadata;
  needsUpdate: boolean;
  databaseVersion: number;
}): DatabaseInfo {
  return {
    path: input.path,
    release: input.release?.release ?? '',
    sha256: input.release?.sha256 ?? '',
    lastUpdate: formatProtoDate(input.metadata.lastUpdate) ?? '',
    needsUpdate: input.needsUpdate,
    databaseVersion: input.databaseVersion
  };
}

function toShindenEntry(
  entry: ProtoShindenEntry | TauriShindenEntry
): ShindenEntry {
  return {
    id: toNumber(entry.id),
    coverId: entry.coverId ?? null,
    title: entry.title,
    animeStatus: entry.animeStatus,
    animeType: entry.animeType,
    premiereDate: formatProtoDate(entry.premiereDate),
    finishDate: formatProtoDate(entry.finishDate),
    episodes: entry.episodes ?? null,
    isFavourite: entry.isFavourite,
    watchStatus: entry.watchStatus,
    watchedEpisodes: entry.watchedEpisodes,
    score: entry.score ?? null
  };
}

function toDatabaseEntry(
  entry: ProtoDatabaseEntry | TauriDatabaseEntry
): DatabaseEntry {
  return {
    id: toNumber(entry.id),
    sources: entry.sources,
    title: entry.title,
    animeType: entry.animeType,
    episodes: entry.episodes,
    status: entry.status,
    season: entry.season,
    year: entry.year ?? null,
    picture: entry.picture,
    thumbnail: entry.thumbnail,
    duration: entry.duration ?? null,
    synonyms: entry.synonyms
  };
}

function toMatchListResult(
  entries: WireShindenMatchResult[],
  nextShindenVersion: WireNumber,
  nextDatabaseVersion: WireNumber
): MatchListResult {
  const mappedEntries = entries.map((entry) => ({
    shindenId: toNumber(entry.shindenId),
    result: toMatchResult(entry.candidates, entry.topCandidates, entry.winner)
  }));

  return {
    entries: mappedEntries,
    total: mappedEntries.length,
    winners: mappedEntries.filter((entry) => entry.result.winner !== null)
      .length,
    hasTop: mappedEntries.filter((entry) => entry.result.top.length > 0).length,
    unmatched: mappedEntries.filter(
      (entry) => entry.result.winner === null && entry.result.top.length === 0
    ).length,
    shindenVersion: toNumber(nextShindenVersion),
    databaseVersion: toNumber(nextDatabaseVersion)
  };
}

function toMatchResult(
  candidates: WireMatchResult[],
  topCandidates: WireMatchResult[] = [],
  winner?: WireMatchResult | null
): MatchResult {
  return {
    items: candidates.map(toScoredCandidate),
    top: topCandidates.map(toScoredCandidate),
    winner: winner == null ? null : toScoredCandidate(winner)
  };
}

function toScoredCandidate(candidate: WireMatchResult) {
  return {
    id: toNumber(candidate.id),
    score: candidate.finalScore
  };
}

function formatProtoDate(date: WireDate) {
  if (date == null) {
    return null;
  }

  const month = String(date.month).padStart(2, '0');
  const day = String(date.day).padStart(2, '0');
  return `${date.year}-${month}-${day}`;
}

function toNumber(value: WireNumber) {
  const numberValue = Number(value);
  if (!Number.isSafeInteger(numberValue)) {
    throw new Error(`Id lub wersja poza bezpiecznym zakresem: ${value}`);
  }

  return numberValue;
}

function normalizeRpcError(error: unknown) {
  const connectError = ConnectError.from(error);
  const detail = appErrorFromConnectError(connectError);
  return new Error(detail?.message || connectError.rawMessage);
}

function normalizeTauriError(error: unknown) {
  if (error instanceof Error) {
    return error;
  }

  if (typeof error === 'string') {
    return new Error(error);
  }

  return new Error('Nie udało się wykonać polecenia Tauri');
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
