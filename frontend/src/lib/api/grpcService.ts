import { create } from '@bufbuild/protobuf';
import {
  AnimeListSortedBy,
  SourceProvider
} from '../gen/shinden_to_anilist/v1/common_pb';
import { SourceIdPairSchema } from '../gen/shinden_to_anilist/v1/export_pb';
import type {
  DatabaseMetadata,
  DatabaseReleaseInfo,
  DatabaseUpdateCheck
} from '../gen/shinden_to_anilist/v1/database_pb';
import type {
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
  SetShindenCloudflareClearanceRequestSchema,
  ShindenCloudflareClearanceSchema
} from '../gen/shinden_to_anilist/v1/service_pb';
import type {
  DatabaseEntry,
  ExportResult,
  MatchListResult,
  MatchSelection,
  SearchOptions,
  SearchResult,
  SourceEntry,
  ShindenCloudflareClearance,
  ShindenListIndex,
  WireNumber
} from '../domain/anime';
import {
  toDatabaseEntry,
  toMatchListResult,
  toMatchResult,
  toShindenEntry,
  toSourceEntry,
  toSourceFetchProgress,
  toSourceMatchListResult,
  type SourceFetchProgress
} from './mapping';
import { callRpc, selectExportPath } from './runtime';
import {
  currentSourceVersion,
  observeDatabaseVersion,
  observeShindenVersion,
  observeSourceVersion
} from './versions';
import { toWireNumber } from './wire';

export async function fetchSourceList(
  provider: SourceProvider,
  user: string,
  onProgress?: (progress: SourceFetchProgress) => void,
  options?: { requestId?: number; signal?: AbortSignal }
) {
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

export async function cancelSourceListFetch(_requestId: number) {}

export async function fetchShindenList(userId: number) {
  return callRpc(async (client) => {
    const response = await client.fetchShindenList(
      create(FetchShindenListRequestSchema, { id: BigInt(userId) })
    );
    observeShindenVersion(response.shindenVersion);
    return { shindenVersion: toWireNumber(response.shindenVersion) };
  });
}

export async function setShindenCloudflareClearance(
  clearance: ShindenCloudflareClearance
) {
  return callRpc(async (client) => {
    const response = await client.setShindenCloudflareClearance(
      create(SetShindenCloudflareClearanceRequestSchema, {
        clearance: create(ShindenCloudflareClearanceSchema, {
          ...clearance,
          capturedAtMs: BigInt(clearance.capturedAtMs)
        })
      })
    );

    return { accepted: response.accepted };
  });
}

export async function getSourceIds(sortedBy = AnimeListSortedBy.URGENCY) {
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

export async function checkDatabaseUpdate(path: string) {
  return callRpc(async (client): Promise<DatabaseUpdateCheck> => {
    const response = await client.checkDatabaseUpdate(
      create(CheckDatabaseUpdateRequestSchema, { path })
    );

    if (response.status === undefined) {
      throw new Error('Brak statusu aktualizacji bazy danych');
    }

    return response.status;
  });
}

export async function downloadDatabase(path: string) {
  return callRpc(async (client): Promise<DatabaseReleaseInfo> => {
    const response = await client.downloadDatabase(
      create(DownloadDatabaseRequestSchema, { path })
    );

    if (response.status === undefined) {
      throw new Error('Brak informacji o pobranej bazie danych');
    }

    return response.status;
  });
}

export async function loadDatabase(path: string) {
  return callRpc(async (client) => {
    const response = await client.loadDatabase(
      create(LoadDatabaseRequestSchema, { path })
    );
    observeDatabaseVersion(response.databaseVersion);
    return { databaseVersion: toWireNumber(response.databaseVersion) };
  });
}

export async function getDatabaseMetadata(path: string) {
  return callRpc(async (client): Promise<DatabaseMetadata> => {
    const response = await client.getDatabaseMetadata(
      create(GetDatabaseMetadataRequestSchema, { path })
    );

    if (response.metadata === undefined) {
      throw new Error('Brak metadanych bazy danych');
    }

    return response.metadata;
  });
}

export async function getDatabaseFull() {
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
  sourceId?: WireNumber
) {
  return callRpc(async (client) => {
    const response = await client.fuzzyMatch(
      create(FuzzyMatchRequestSchema, {
        query,
        options: createSearchOptions(options),
        shindenId: sourceId === undefined ? undefined : BigInt(sourceId),
        sourceId: sourceId === undefined ? undefined : BigInt(sourceId)
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

export async function exportXml(
  matches: MatchSelection[],
  resolvedPath: string
) {
  return callRpc(async (client): Promise<ExportResult> => {
    const selectedPath = await selectExportPath(resolvedPath);

    if (selectedPath === null) {
      return {
        path: resolvedPath,
        exportedCount: 0,
        cancelled: true,
        shindenVersion: currentSourceVersion()
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

function createSearchOptions(options: SearchOptions) {
  return create(SearchOptionsSchema, {
    limit: options.limit ?? 0,
    threshold: options.threshold
  });
}
