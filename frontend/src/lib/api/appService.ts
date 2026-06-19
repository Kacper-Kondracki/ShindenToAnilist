import { create } from '@bufbuild/protobuf';
import {
  ConnectError,
  createClient,
  decodeBinaryHeader
} from '@connectrpc/connect';
import { createGrpcWebTransport } from '@connectrpc/connect-web';
import { Channel, invoke, isTauri as isTauriApi } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import {
  AnimeListSortedBy,
  SourceProvider,
  type Date as ProtoDate
} from '../gen/shinden_to_anilist/v1/common_pb';
import { AppErrorSchema } from '../gen/shinden_to_anilist/v1/error_pb';
import { SourceIdPairSchema } from '../gen/shinden_to_anilist/v1/export_pb';
import type {
  DatabaseEntry as ProtoDatabaseEntry,
  DatabaseMetadata,
  DatabaseReleaseInfo,
  DatabaseUpdateCheck
} from '../gen/shinden_to_anilist/v1/database_pb';
import type {
  MatchResult as ProtoMatchResult,
  ShindenMatchResult as ProtoShindenMatchResult,
  SourceMatchResult as ProtoSourceMatchResult
} from '../gen/shinden_to_anilist/v1/matching_pb';
import { SearchOptionsSchema } from '../gen/shinden_to_anilist/v1/search_pb';
import {
  CheckDatabaseUpdateRequestSchema,
  DownloadDatabaseRequestSchema,
  ExportXmlRequestSchema,
  FetchSourceListRequestSchema,
  FetchShindenListRequestSchema,
  FuzzyMatchRequestSchema,
  FuzzySearchRequestSchema,
  GetDatabaseFullRequestSchema,
  GetDatabaseMetadataRequestSchema,
  GetSourceFullRequestSchema,
  GetSourceIdsRequestSchema,
  GetShindenFullRequestSchema,
  GetShindenIdsRequestSchema,
  LoadDatabaseRequestSchema,
  MatchSourceListRequestSchema,
  MatchShindenListRequestSchema,
  ShindenToAnilistService
} from '../gen/shinden_to_anilist/v1/service_pb';
import type { ShindenEntry as ProtoShindenEntry } from '../gen/shinden_to_anilist/v1/shinden_pb';
import type {
  SourceEntry as ProtoSourceEntry,
  SourceFetchProgress as ProtoSourceFetchProgress
} from '../gen/shinden_to_anilist/v1/source_pb';
import type {
  DatabaseEntry,
  DatabaseInfo,
  ExportResult,
  MatchListResult,
  MatchResult,
  MatchSelection,
  SearchOptions,
  SearchResult,
  SourceEntry,
  ShindenListIndex,
  WireNumber
} from '../domain/anime';

type WireNumberInput = WireNumber | number | string;

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
  compressedSize?: WireNumberInput;
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
  id: WireNumberInput;
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

type TauriSourceEntry = {
  id: WireNumberInput;
  provider: number;
  title: string;
  animeStatus: number;
  animeType: number;
  premiereDate?: WireDate;
  year?: number | null;
  episodes?: number | null;
  watchStatus: number;
  watchedEpisodes: number;
  score?: number | null;
  sourceUrl: string;
  malId?: WireNumberInput | null;
  coverId?: number | null;
  isFavourite?: boolean | null;
};

type TauriDatabaseEntry = {
  id: WireNumberInput;
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
  id: WireNumberInput;
  finalScore: number;
};

type TauriShindenMatchResult = {
  shindenId: WireNumberInput;
  candidates: TauriMatchResult[];
  topCandidates: TauriMatchResult[];
  winner?: TauriMatchResult | null;
};

type WireShindenMatchResult = ProtoShindenMatchResult | TauriShindenMatchResult;

type TauriSourceMatchResult = {
  sourceId: WireNumberInput;
  candidates: TauriMatchResult[];
  topCandidates: TauriMatchResult[];
  winner?: TauriMatchResult | null;
};

type WireSourceMatchResult = ProtoSourceMatchResult | TauriSourceMatchResult;

type TauriSearchOptions = {
  limit: number;
  threshold?: number;
};

type TauriFetchShindenListResponse = {
  shindenVersion: WireNumberInput;
};

export type SourceFetchProgress = {
  provider: SourceProvider;
  phase: number;
  current: number;
  total: number;
  latestTitle: string;
};

type TauriSourceFetchProgress = {
  provider: number;
  phase: number;
  current: WireNumberInput;
  total: WireNumberInput;
  latestTitle: string;
};

type TauriFetchSourceListResponse = {
  sourceVersion: WireNumberInput;
};

type TauriGetShindenIdsResponse = {
  shindenVersion: WireNumberInput;
  ids: WireNumberInput[];
};

type TauriGetSourceIdsResponse = {
  sourceVersion: WireNumberInput;
  ids: WireNumberInput[];
};

type TauriGetShindenFullResponse = {
  shindenVersion: WireNumberInput;
  entries: TauriShindenEntry[];
};

type TauriGetSourceFullResponse = {
  sourceVersion: WireNumberInput;
  entries: TauriSourceEntry[];
};

type TauriCheckDatabaseUpdateResponse = {
  status?: AppDatabaseUpdateCheck | null;
};

type TauriDownloadDatabaseResponse = {
  status?: AppDatabaseReleaseInfo | null;
};

type TauriLoadDatabaseResponse = {
  databaseVersion: WireNumberInput;
};

type TauriGetDatabaseMetadataResponse = {
  metadata?: AppDatabaseMetadata | null;
};

type TauriGetDatabaseFullResponse = {
  databaseVersion: WireNumberInput;
  entries: TauriDatabaseEntry[];
};

type TauriFuzzySearchResponse = {
  databaseVersion: WireNumberInput;
  results: Array<{ id: WireNumberInput; score: number }>;
};

type TauriFuzzyMatchResponse = {
  databaseVersion: WireNumberInput;
  results: TauriMatchResult[];
};

type TauriMatchShindenListResponse = {
  shindenVersion: WireNumberInput;
  databaseVersion: WireNumberInput;
  results: TauriShindenMatchResult[];
};

type TauriMatchSourceListResponse = {
  sourceVersion: WireNumberInput;
  databaseVersion: WireNumberInput;
  results: TauriSourceMatchResult[];
};

type TauriExportXmlResponse = {
  sourceVersion: WireNumberInput;
  shindenVersion: WireNumberInput;
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

const U64_MAX = (1n << 64n) - 1n;

let shindenVersion: WireNumber = 0n;
let sourceVersion: WireNumber = 0n;
let databaseVersion: WireNumber = 0n;

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
    source: sourceVersion,
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

export async function fetchSourceList(
  provider: SourceProvider,
  user: string,
  onProgress?: (progress: SourceFetchProgress) => void,
  options?: { requestId?: number; signal?: AbortSignal }
) {
  if (isTauriRuntime()) {
    const progressChannel = new Channel<TauriSourceFetchProgress>();
    progressChannel.onmessage = (progress) => {
      onProgress?.(toSourceFetchProgress(progress));
    };

    const response = await callTauri<TauriFetchSourceListResponse>(
      'fetch_source_list',
      {
        requestId: options?.requestId ?? 0,
        provider,
        user,
        onProgress: progressChannel
      }
    );
    observeSourceVersion(response.sourceVersion);
    return { sourceVersion: toWireNumber(response.sourceVersion) };
  }

  return callRpc(async (client) => {
    let nextSourceVersion = 0n;
    const callOptions =
      options?.signal === undefined ? undefined : { signal: options.signal };

    for await (const chunk of client.fetchSourceList(
      create(FetchSourceListRequestSchema, { provider, user }),
      callOptions
    )) {
      if (chunk.progress !== undefined) {
        onProgress?.(toSourceFetchProgress(chunk.progress));
      }

      if (chunk.done) {
        nextSourceVersion = chunk.sourceVersion;
      }
    }

    observeSourceVersion(nextSourceVersion);
    return { sourceVersion: toWireNumber(nextSourceVersion) };
  });
}

export async function cancelSourceListFetch(requestId: number) {
  if (!isTauriRuntime()) {
    return;
  }

  await callTauri<void>('cancel_source_list_fetch', { requestId });
}

export async function fetchShindenList(userId: number) {
  if (isTauriRuntime()) {
    const response = await callTauri<TauriFetchShindenListResponse>(
      'fetch_shinden_list',
      { id: toTauriWireNumber(userId) }
    );
    observeShindenVersion(response.shindenVersion);
    return { shindenVersion: toWireNumber(response.shindenVersion) };
  }

  return callRpc(async (client) => {
    const response = await client.fetchShindenList(
      create(FetchShindenListRequestSchema, { id: BigInt(userId) })
    );
    observeShindenVersion(response.shindenVersion);
    return { shindenVersion: toWireNumber(response.shindenVersion) };
  });
}

export async function getSourceIds(sortedBy = AnimeListSortedBy.URGENCY) {
  if (isTauriRuntime()) {
    const response = await callTauri<TauriGetSourceIdsResponse>(
      'get_source_ids',
      { sortedBy }
    );
    observeSourceVersion(response.sourceVersion);

    return {
      sourceVersion: toWireNumber(response.sourceVersion),
      shindenVersion: toWireNumber(response.sourceVersion),
      entryIds: response.ids.map(toWireNumber)
    };
  }

  return callRpc(async (client): Promise<ShindenListIndex> => {
    const response = await client.getSourceIds(
      create(GetSourceIdsRequestSchema, { sortedBy })
    );
    observeSourceVersion(response.sourceVersion);

    return {
      sourceVersion: toWireNumber(response.sourceVersion),
      shindenVersion: toWireNumber(response.sourceVersion),
      entryIds: response.ids.map(toWireNumber)
    };
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
      shindenVersion: toWireNumber(response.shindenVersion),
      entryIds: response.ids.map(toWireNumber)
    };
  }

  return callRpc(async (client): Promise<ShindenListIndex> => {
    const response = await client.getShindenIds(
      create(GetShindenIdsRequestSchema, { sortedBy })
    );
    observeShindenVersion(response.shindenVersion);

    return {
      shindenVersion: toWireNumber(response.shindenVersion),
      entryIds: response.ids.map(toWireNumber)
    };
  });
}

export async function getSourceFull() {
  if (isTauriRuntime()) {
    const response =
      await callTauri<TauriGetSourceFullResponse>('get_source_full');
    observeSourceVersion(response.sourceVersion);
    return {
      sourceVersion: toWireNumber(response.sourceVersion),
      shindenVersion: toWireNumber(response.sourceVersion),
      entries: response.entries.map(toSourceEntry)
    };
  }

  return callRpc(async (client) => {
    const entries: SourceEntry[] = [];
    let version = 0n;

    for await (const chunk of client.getSourceFull(
      create(GetSourceFullRequestSchema)
    )) {
      version = chunk.sourceVersion;
      entries.push(...chunk.entries.map(toSourceEntry));
    }

    observeSourceVersion(version);
    return {
      sourceVersion: toWireNumber(version),
      shindenVersion: toWireNumber(version),
      entries
    };
  });
}

export async function getShindenFull() {
  if (isTauriRuntime()) {
    const response =
      await callTauri<TauriGetShindenFullResponse>('get_shinden_full');
    observeShindenVersion(response.shindenVersion);
    return {
      shindenVersion: toWireNumber(response.shindenVersion),
      entries: response.entries.map(toShindenEntry)
    };
  }

  return callRpc(async (client) => {
    const entries: SourceEntry[] = [];
    let version = 0n;

    for await (const chunk of client.getShindenFull(
      create(GetShindenFullRequestSchema)
    )) {
      version = chunk.shindenVersion;
      entries.push(...chunk.entries.map(toShindenEntry));
    }

    observeShindenVersion(version);
    return {
      shindenVersion: toWireNumber(version),
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
    return { databaseVersion: toWireNumber(response.databaseVersion) };
  }

  return callRpc(async (client) => {
    const response = await client.loadDatabase(
      create(LoadDatabaseRequestSchema, { path: resolvedPath })
    );
    observeDatabaseVersion(response.databaseVersion);
    return { databaseVersion: toWireNumber(response.databaseVersion) };
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
      databaseVersion: toWireNumber(response.databaseVersion),
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
      databaseVersion: toWireNumber(version),
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
      databaseVersion: toWireNumber(response.databaseVersion),
      items: response.results.map((item) => ({
        id: toWireNumber(item.id),
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
      databaseVersion: toWireNumber(response.databaseVersion),
      items: response.results.map((item) => ({
        id: toWireNumber(item.id),
        score: item.score
      }))
    };
  });
}

export async function fuzzyMatch(
  query: string,
  options: SearchOptions = {},
  shindenId?: WireNumber
) {
  if (isTauriRuntime()) {
    const response = await callTauri<TauriFuzzyMatchResponse>('fuzzy_match', {
      query,
      options: createTauriSearchOptions(options),
      shindenId:
        shindenId === undefined ? undefined : toTauriWireNumber(shindenId),
      sourceId:
        shindenId === undefined ? undefined : toTauriWireNumber(shindenId)
    });
    observeDatabaseVersion(response.databaseVersion);

    return {
      databaseVersion: toWireNumber(response.databaseVersion),
      result: toMatchResult(response.results)
    };
  }

  return callRpc(async (client) => {
    const response = await client.fuzzyMatch(
      create(FuzzyMatchRequestSchema, {
        query,
        options: createSearchOptions(options),
        shindenId: shindenId === undefined ? undefined : BigInt(shindenId),
        sourceId: shindenId === undefined ? undefined : BigInt(shindenId)
      })
    );
    observeDatabaseVersion(response.databaseVersion);

    return {
      databaseVersion: toWireNumber(response.databaseVersion),
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

export async function matchSourceList(options: SearchOptions = {}) {
  if (isTauriRuntime()) {
    const response = await callTauri<TauriMatchSourceListResponse>(
      'match_source_list',
      { options: createTauriSearchOptions(options) }
    );
    observeSourceVersion(response.sourceVersion);
    observeDatabaseVersion(response.databaseVersion);
    return toSourceMatchListResult(
      response.results,
      response.sourceVersion,
      response.databaseVersion
    );
  }

  return callRpc(async (client): Promise<MatchListResult> => {
    const entries: ProtoSourceMatchResult[] = [];
    let nextSourceVersion = 0n;
    let nextDatabaseVersion = 0n;

    for await (const chunk of client.matchSourceList(
      create(MatchSourceListRequestSchema, {
        options: createSearchOptions(options)
      })
    )) {
      nextSourceVersion = chunk.sourceVersion;
      nextDatabaseVersion = chunk.databaseVersion;
      entries.push(...chunk.results);
    }

    observeSourceVersion(nextSourceVersion);
    observeDatabaseVersion(nextDatabaseVersion);
    return toSourceMatchListResult(
      entries,
      nextSourceVersion,
      nextDatabaseVersion
    );
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
        shindenVersion: sourceVersion
      };
    }

    const response = await callTauri<TauriExportXmlResponse>('export_xml', {
      path: selectedPath,
      matches: matches.map((match) => ({
        sourceId: toTauriWireNumber(match.sourceId),
        databaseId: toTauriWireNumber(match.databaseId)
      }))
    });
    observeSourceVersion(response.sourceVersion);

    return {
      path: response.path,
      exportedCount: matches.length,
      cancelled: false,
      shindenVersion: toWireNumber(response.sourceVersion)
    };
  }

  return callRpc(async (client): Promise<ExportResult> => {
    const selectedPath = await selectExportPath(resolvedPath);

    if (selectedPath === null) {
      return {
        path: resolvedPath,
        exportedCount: 0,
        cancelled: true,
        shindenVersion: sourceVersion
      };
    }

    const response = await client.exportXml(
      create(ExportXmlRequestSchema, {
        path: selectedPath,
        matches: matches.map((match) =>
          create(SourceIdPairSchema, {
            sourceId: BigInt(match.sourceId),
            databaseId: BigInt(match.databaseId)
          })
        )
      })
    );
    observeSourceVersion(response.sourceVersion);

    return {
      path: response.path,
      exportedCount: matches.length,
      cancelled: false,
      shindenVersion: toWireNumber(response.sourceVersion)
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

function observeShindenVersion(version: WireNumberInput) {
  const nextVersion = toWireNumber(version);
  if (isGreaterWireNumber(nextVersion, shindenVersion)) {
    shindenVersion = nextVersion;
  }
}

function observeSourceVersion(version: WireNumberInput) {
  const nextVersion = toWireNumber(version);
  if (isGreaterWireNumber(nextVersion, sourceVersion)) {
    sourceVersion = nextVersion;
  }
  observeShindenVersion(version);
}

function observeDatabaseVersion(version: WireNumberInput) {
  const nextVersion = toWireNumber(version);
  if (isGreaterWireNumber(nextVersion, databaseVersion)) {
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
  databaseVersion: WireNumberInput;
}): DatabaseInfo {
  return {
    path: input.path,
    release: input.release?.release ?? '',
    sha256: input.release?.sha256 ?? '',
    lastUpdate: formatProtoDate(input.metadata.lastUpdate) ?? '',
    needsUpdate: input.needsUpdate,
    databaseVersion: toWireNumber(input.databaseVersion)
  };
}

function toShindenEntry(
  entry: ProtoShindenEntry | TauriShindenEntry
): SourceEntry {
  return {
    id: toWireNumber(entry.id),
    provider: SourceProvider.SHINDEN,
    coverId: entry.coverId ?? null,
    title: entry.title,
    animeStatus: entry.animeStatus,
    animeType: entry.animeType,
    premiereDate: formatProtoDate(entry.premiereDate),
    year: entry.premiereDate?.year ?? null,
    finishDate: formatProtoDate(entry.finishDate),
    episodes: entry.episodes ?? null,
    isFavourite: entry.isFavourite,
    watchStatus: entry.watchStatus,
    watchedEpisodes: entry.watchedEpisodes,
    score: entry.score ?? null,
    sourceUrl: `https://shinden.pl/series/${entry.id.toString()}`,
    malId: null
  };
}

function toSourceEntry(
  entry: ProtoSourceEntry | TauriSourceEntry
): SourceEntry {
  return {
    id: toWireNumber(entry.id),
    provider: entry.provider,
    title: entry.title,
    animeStatus: entry.animeStatus,
    animeType: entry.animeType,
    premiereDate: formatProtoDate(entry.premiereDate),
    year: entry.year ?? entry.premiereDate?.year ?? null,
    finishDate: null,
    episodes: entry.episodes ?? null,
    watchStatus: entry.watchStatus,
    watchedEpisodes: entry.watchedEpisodes,
    score: entry.score ?? null,
    sourceUrl: entry.sourceUrl,
    malId: entry.malId == null ? null : toWireNumber(entry.malId),
    coverId: 'coverId' in entry ? (entry.coverId ?? null) : null,
    isFavourite: 'isFavourite' in entry ? (entry.isFavourite ?? false) : false
  };
}

function toDatabaseEntry(
  entry: ProtoDatabaseEntry | TauriDatabaseEntry
): DatabaseEntry {
  return {
    id: toWireNumber(entry.id),
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
  nextShindenVersion: WireNumberInput,
  nextDatabaseVersion: WireNumberInput
): MatchListResult {
  const mappedEntries = entries.map((entry) => ({
    shindenId: toWireNumber(entry.shindenId),
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
    shindenVersion: toWireNumber(nextShindenVersion),
    sourceVersion: toWireNumber(nextShindenVersion),
    databaseVersion: toWireNumber(nextDatabaseVersion)
  };
}

function toSourceMatchListResult(
  entries: WireSourceMatchResult[],
  nextSourceVersion: WireNumberInput,
  nextDatabaseVersion: WireNumberInput
): MatchListResult {
  const mappedEntries = entries.map((entry) => ({
    shindenId: toWireNumber(entry.sourceId),
    sourceId: toWireNumber(entry.sourceId),
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
    shindenVersion: toWireNumber(nextSourceVersion),
    sourceVersion: toWireNumber(nextSourceVersion),
    databaseVersion: toWireNumber(nextDatabaseVersion)
  };
}

function toSourceFetchProgress(
  progress: ProtoSourceFetchProgress | TauriSourceFetchProgress
): SourceFetchProgress {
  return {
    provider: progress.provider,
    phase: progress.phase,
    current: toSafeNumber(progress.current),
    total: toSafeNumber(progress.total),
    latestTitle: progress.latestTitle
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
    id: toWireNumber(candidate.id),
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

function toWireNumber(value: WireNumberInput): WireNumber {
  if (typeof value === 'number') {
    if (!Number.isInteger(value)) {
      throw new Error(`Id lub wersja nie jest liczbą całkowitą: ${value}`);
    }

    if (!Number.isSafeInteger(value)) {
      throw new Error(
        `Id lub wersja poza bezpiecznym zakresem number: ${value}`
      );
    }

    if (value < 0) {
      throw new Error(`Id lub wersja poza zakresem u64: ${value}`);
    }

    return BigInt(value);
  }

  if (typeof value === 'string') {
    if (!/^\d+$/.test(value)) {
      throw new Error(`Id lub wersja nie jest liczbą całkowitą: ${value}`);
    }

    value = BigInt(value);
  }

  if (value < 0n || value > U64_MAX) {
    throw new Error(`Id lub wersja poza zakresem u64: ${value}`);
  }

  return value;
}

function toTauriWireNumber(value: WireNumberInput): string {
  return toWireNumber(value).toString();
}

function toSafeNumber(value: WireNumberInput): number {
  const normalized = toWireNumber(value);

  const numberValue = Number(normalized);
  if (!Number.isSafeInteger(numberValue)) {
    throw new Error(`Id lub wersja poza bezpiecznym zakresem number: ${value}`);
  }

  return numberValue;
}

function isGreaterWireNumber(left: WireNumber, right: WireNumber) {
  return left > right;
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
