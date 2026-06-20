import { Channel } from '@tauri-apps/api/core';
import {
  AnimeListSortedBy,
  SourceProvider
} from '../gen/shinden_to_anilist/v1/common_pb';
import type {
  ExportResult,
  MatchSelection,
  SearchOptions,
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
  type AppDatabaseMetadata,
  type AppDatabaseReleaseInfo,
  type AppDatabaseUpdateCheck,
  type SourceFetchProgress,
  type TauriDatabaseEntry,
  type TauriMatchResult,
  type TauriSearchOptions,
  type TauriShindenEntry,
  type TauriShindenMatchResult,
  type TauriSourceEntry,
  type TauriSourceFetchProgress,
  type TauriSourceMatchResult
} from './mapping';
import { callTauri, selectExportPath } from './runtime';
import {
  currentSourceVersion,
  observeDatabaseVersion,
  observeShindenVersion,
  observeSourceVersion
} from './versions';
import { toTauriWireNumber, toWireNumber, type WireNumberInput } from './wire';

type TauriFetchShindenListResponse = {
  shindenVersion: WireNumberInput;
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

export async function fetchSourceList(
  provider: SourceProvider,
  user: string,
  onProgress?: (progress: SourceFetchProgress) => void,
  options?: { requestId?: number; signal?: AbortSignal }
) {
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

export async function cancelSourceListFetch(requestId: number) {
  await callTauri<void>('cancel_source_list_fetch', { requestId });
}

export async function fetchShindenList(userId: number) {
  const response = await callTauri<TauriFetchShindenListResponse>(
    'fetch_shinden_list',
    { id: toTauriWireNumber(userId) }
  );
  observeShindenVersion(response.shindenVersion);
  return { shindenVersion: toWireNumber(response.shindenVersion) };
}

export async function getSourceIds(sortedBy = AnimeListSortedBy.URGENCY) {
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

export async function getShindenIds(sortedBy = AnimeListSortedBy.URGENCY) {
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

export async function getSourceFull() {
  const response =
    await callTauri<TauriGetSourceFullResponse>('get_source_full');
  observeSourceVersion(response.sourceVersion);
  return {
    sourceVersion: toWireNumber(response.sourceVersion),
    shindenVersion: toWireNumber(response.sourceVersion),
    entries: response.entries.map(toSourceEntry)
  };
}

export async function getShindenFull() {
  const response =
    await callTauri<TauriGetShindenFullResponse>('get_shinden_full');
  observeShindenVersion(response.shindenVersion);
  return {
    shindenVersion: toWireNumber(response.shindenVersion),
    entries: response.entries.map(toShindenEntry)
  };
}

export async function checkDatabaseUpdate(path: string) {
  const response = await callTauri<TauriCheckDatabaseUpdateResponse>(
    'check_database_update',
    { path }
  );

  if (response.status == null) {
    throw new Error('Brak statusu aktualizacji bazy danych');
  }

  return response.status;
}

export async function downloadDatabase(path: string) {
  const response = await callTauri<TauriDownloadDatabaseResponse>(
    'download_database',
    { path }
  );

  if (response.status == null) {
    throw new Error('Brak informacji o pobranej bazie danych');
  }

  return response.status;
}

export async function loadDatabase(path: string) {
  const response = await callTauri<TauriLoadDatabaseResponse>('load_database', {
    path
  });
  observeDatabaseVersion(response.databaseVersion);
  return { databaseVersion: toWireNumber(response.databaseVersion) };
}

export async function getDatabaseMetadata(path: string) {
  const response = await callTauri<TauriGetDatabaseMetadataResponse>(
    'get_database_metadata',
    { path }
  );

  if (response.metadata == null) {
    throw new Error('Brak metadanych bazy danych');
  }

  return response.metadata;
}

export async function getDatabaseFull() {
  const response =
    await callTauri<TauriGetDatabaseFullResponse>('get_database_full');
  observeDatabaseVersion(response.databaseVersion);
  return {
    databaseVersion: toWireNumber(response.databaseVersion),
    entries: response.entries.map(toDatabaseEntry)
  };
}

export async function fuzzySearch(query: string, options: SearchOptions = {}) {
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

export async function fuzzyMatch(
  query: string,
  options: SearchOptions = {},
  sourceId?: WireNumber
) {
  const response = await callTauri<TauriFuzzyMatchResponse>('fuzzy_match', {
    query,
    options: createTauriSearchOptions(options),
    shindenId: sourceId === undefined ? undefined : toTauriWireNumber(sourceId),
    sourceId: sourceId === undefined ? undefined : toTauriWireNumber(sourceId)
  });
  observeDatabaseVersion(response.databaseVersion);

  return {
    databaseVersion: toWireNumber(response.databaseVersion),
    result: toMatchResult(response.results)
  };
}

export async function matchShindenList(options: SearchOptions = {}) {
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

export async function matchSourceList(options: SearchOptions = {}) {
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

export async function exportXml(
  matches: MatchSelection[],
  resolvedPath: string
): Promise<ExportResult> {
  const selectedPath = await selectExportPath(resolvedPath);

  if (selectedPath === null) {
    return {
      path: resolvedPath,
      exportedCount: 0,
      cancelled: true,
      shindenVersion: currentSourceVersion()
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

function createTauriSearchOptions(options: SearchOptions): TauriSearchOptions {
  const tauriOptions: TauriSearchOptions = {
    limit: options.limit ?? 0
  };

  if (options.threshold !== undefined) {
    tauriOptions.threshold = options.threshold;
  }

  return tauriOptions;
}
