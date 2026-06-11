import type {
  DatabaseEntry,
  DatabaseInfo,
  ExportResult,
  MatchListResult,
  MatchOptions,
  MatchQueryOptions,
  MatchResult,
  MatchSelection,
  SearchOptions,
  SearchResult,
  ShindenEntry,
  ShindenListIndex,
  ShindenListView
} from '../domain/anime';

type DriverSearchOptions = Required<SearchOptions>;

type DriverMatchOptions = Required<MatchOptions>;

type DriverMatchQueryOptions = {
  search: DriverSearchOptions;
  resultLimit: number | null;
};

export async function ensureDatabase() {
  return (await AppService.EnsureDatabase()) as DatabaseInfo;
}

export async function loadShindenList(userId: number) {
  return (await AppService.LoadShindenList(userId)) as ShindenListIndex;
}

export async function getLoadedShindenEntryIds(view: ShindenListView) {
  return (await AppService.GetLoadedShindenEntryIDs(view)) as ShindenListIndex;
}

export async function getLoadedShindenEntries(entryIds: number[]) {
  return (await AppService.GetLoadedShindenEntries(entryIds)) as ShindenEntry[];
}

export async function getAnimeDatabaseEntries(entryIds: number[]) {
  return (await AppService.GetAnimeDatabaseEntries(
    entryIds
  )) as DatabaseEntry[];
}

export async function matchLoadedShindenList(options: MatchOptions = {}) {
  return (await AppService.MatchLoadedShindenList(
    toDriverMatchOptions(options)
  )) as MatchListResult;
}

export async function searchAnime(query: string, options: SearchOptions = {}) {
  return (await AppService.SearchAnime(
    query,
    toDriverSearchOptions(options)
  )) as SearchResult;
}

export async function matchQuery(
  query: string,
  options: MatchQueryOptions = {}
) {
  return (await AppService.MatchQuery(
    query,
    toDriverMatchQueryOptions(options)
  )) as MatchResult;
}

export async function exportMatches(matches: MatchSelection[]) {
  return (await AppService.ExportMatches(matches)) as ExportResult;
}

function toDriverSearchOptions(
  options: SearchOptions = {}
): DriverSearchOptions {
  return {
    mode: options.mode ?? '',
    limit: options.limit ?? 0,
    threshold: options.threshold ?? 0
  };
}

function toDriverMatchOptions(options: MatchOptions = {}): DriverMatchOptions {
  return {
    candidateLimit: options.candidateLimit ?? 0,
    searchThreshold: options.searchThreshold ?? 0,
    resultLimit: options.resultLimit ?? null
  };
}

function toDriverMatchQueryOptions(
  options: MatchQueryOptions = {}
): DriverMatchQueryOptions {
  return {
    search: toDriverSearchOptions(options.search),
    resultLimit: options.resultLimit ?? null
  };
}
