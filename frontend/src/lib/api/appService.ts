import { create } from '@bufbuild/protobuf';
import {
  ConnectError,
  createClient,
  decodeBinaryHeader
} from '@connectrpc/connect';
import { createGrpcWebTransport } from '@connectrpc/connect-web';
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
  GetDatabaseEntriesRequestSchema,
  GetDatabaseFullRequestSchema,
  GetDatabaseMetadataRequestSchema,
  GetShindenEntriesRequestSchema,
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
import {
  clearDatabaseQueries,
  clearMatchQueries,
  clearShindenQueries
} from './queryClient';

const grpcBaseUrl = 'http://127.0.0.1:50051';
export const databasePath = '/tmp/shinden-to-anilist-database.jsonl';
export const exportPath = '/tmp/shinden-to-anilist-export.xml';

const transport = createGrpcWebTransport({
  baseUrl: grpcBaseUrl
});

const client = createClient(ShindenToAnilistService, transport);

let shindenVersion = 0;
let databaseVersion = 0;

export function currentVersions() {
  return {
    shinden: shindenVersion,
    database: databaseVersion
  };
}

export async function ensureDatabase() {
  const update = await checkDatabaseUpdate(databasePath);

  let release = update.local ?? update.remote ?? null;
  if (update.needsUpdate || release === null) {
    release = await downloadDatabase(databasePath);
  }

  const [{ databaseVersion: loadedVersion }, metadata] = await Promise.all([
    loadDatabase(databasePath),
    getDatabaseMetadata(databasePath)
  ]);

  return toDatabaseInfo({
    path: databasePath,
    release,
    metadata,
    needsUpdate: update.needsUpdate,
    databaseVersion: loadedVersion
  });
}

export async function fetchShindenList(userId: number) {
  return callRpc(async () => {
    const response = await client.fetchShindenList(
      create(FetchShindenListRequestSchema, { id: BigInt(userId) })
    );
    observeShindenVersion(response.shindenVersion);
    return { shindenVersion: toNumber(response.shindenVersion) };
  });
}

export async function getShindenIds(sortedBy = AnimeListSortedBy.URGENCY) {
  return callRpc(async (): Promise<ShindenListIndex> => {
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

export async function getShindenEntries(entryIds: number[]) {
  return callRpc(async () => {
    const response = await client.getShindenEntries(
      create(GetShindenEntriesRequestSchema, {
        ids: entryIds.map(BigInt)
      })
    );
    observeShindenVersion(response.shindenVersion);
    return response.entries.map(toShindenEntry);
  });
}

export async function getShindenFull() {
  return callRpc(async () => {
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

export async function checkDatabaseUpdate(path = databasePath) {
  return callRpc(async () => {
    const response = await client.checkDatabaseUpdate(
      create(CheckDatabaseUpdateRequestSchema, { path })
    );

    if (response.status === undefined) {
      throw new Error('Brak statusu aktualizacji bazy danych');
    }

    return response.status;
  });
}

export async function downloadDatabase(path = databasePath) {
  return callRpc(async () => {
    const response = await client.downloadDatabase(
      create(DownloadDatabaseRequestSchema, { path })
    );

    if (response.status === undefined) {
      throw new Error('Brak informacji o pobranej bazie danych');
    }

    return response.status;
  });
}

export async function loadDatabase(path = databasePath) {
  return callRpc(async () => {
    const response = await client.loadDatabase(
      create(LoadDatabaseRequestSchema, { path })
    );
    observeDatabaseVersion(response.databaseVersion);
    return { databaseVersion: toNumber(response.databaseVersion) };
  });
}

export async function getDatabaseMetadata(path = databasePath) {
  return callRpc(async () => {
    const response = await client.getDatabaseMetadata(
      create(GetDatabaseMetadataRequestSchema, { path })
    );

    if (response.metadata === undefined) {
      throw new Error('Brak metadanych bazy danych');
    }

    return response.metadata;
  });
}

export async function getDatabaseEntries(entryIds: number[]) {
  return callRpc(async () => {
    const response = await client.getDatabaseEntries(
      create(GetDatabaseEntriesRequestSchema, {
        ids: entryIds.map(BigInt)
      })
    );
    observeDatabaseVersion(response.databaseVersion);
    return response.entries.map(toDatabaseEntry);
  });
}

export async function getDatabaseFull() {
  return callRpc(async () => {
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
  return callRpc(async (): Promise<SearchResult> => {
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

export async function fuzzyMatch(query: string, options: SearchOptions = {}) {
  return callRpc(async () => {
    const response = await client.fuzzyMatch(
      create(FuzzyMatchRequestSchema, {
        query,
        options: createSearchOptions(options)
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
  return callRpc(async (): Promise<MatchListResult> => {
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

export async function exportXml(matches: MatchSelection[], path = exportPath) {
  return callRpc(async (): Promise<ExportResult> => {
    const response = await client.exportXml(
      create(ExportXmlRequestSchema, {
        path,
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

async function callRpc<T>(run: () => Promise<T>) {
  try {
    return await run();
  } catch (error) {
    throw normalizeRpcError(error);
  }
}

function observeShindenVersion(version: bigint) {
  const nextVersion = toNumber(version);
  if (nextVersion > shindenVersion) {
    shindenVersion = nextVersion;
    clearShindenQueries();
    clearMatchQueries();
  }
}

function observeDatabaseVersion(version: bigint) {
  const nextVersion = toNumber(version);
  if (nextVersion > databaseVersion) {
    databaseVersion = nextVersion;
    clearDatabaseQueries();
    clearMatchQueries();
  }
}

function createSearchOptions(options: SearchOptions) {
  return create(SearchOptionsSchema, {
    limit: options.limit ?? 0,
    threshold: options.threshold
  });
}

function toDatabaseInfo(input: {
  path: string;
  release: DatabaseReleaseInfo | null;
  metadata: DatabaseMetadata;
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

function toShindenEntry(entry: ProtoShindenEntry): ShindenEntry {
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

function toDatabaseEntry(entry: ProtoDatabaseEntry): DatabaseEntry {
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
  entries: ProtoShindenMatchResult[],
  nextShindenVersion: bigint,
  nextDatabaseVersion: bigint
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
    hasTop: mappedEntries.filter((entry) => entry.result.top.length > 0)
      .length,
    unmatched: mappedEntries.filter(
      (entry) =>
        entry.result.winner === null && entry.result.top.length === 0
    ).length,
    shindenVersion: toNumber(nextShindenVersion),
    databaseVersion: toNumber(nextDatabaseVersion)
  };
}

function toMatchResult(
  candidates: ProtoMatchResult[],
  topCandidates: ProtoMatchResult[] = [],
  winner?: ProtoMatchResult
): MatchResult {
  return {
    items: candidates.map(toScoredCandidate),
    top: topCandidates.map(toScoredCandidate),
    winner: winner === undefined ? null : toScoredCandidate(winner)
  };
}

function toScoredCandidate(candidate: ProtoMatchResult) {
  return {
    id: toNumber(candidate.id),
    score: candidate.finalScore
  };
}

function formatProtoDate(date: ProtoDate | undefined) {
  if (date === undefined) {
    return null;
  }

  const month = String(date.month).padStart(2, '0');
  const day = String(date.day).padStart(2, '0');
  return `${date.year}-${month}-${day}`;
}

function toNumber(value: bigint) {
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
