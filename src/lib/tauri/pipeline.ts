import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export const PIPELINE_PROGRESS_EVENT = 'pipeline://progress';

export type PipelineStage =
  | 'databaseUpdateStarted'
  | 'databaseUpdateFinished'
  | 'databaseLoadStarted'
  | 'databaseLoadFinished'
  | 'shindenFetchStarted'
  | 'shindenFetchFinished'
  | 'matchStarted'
  | 'matchFinished'
  | 'exportFinished';

export type PipelineProgressEvent = {
  stage: PipelineStage;
  message: string;
};

export type ApiError = {
  message: string;
};

export type AppStatus = {
  databasePath: string;
  databaseExists: boolean;
  databaseLoaded: boolean;
  databaseEntryCount: number;
  databaseLastUpdate: string | null;
  shindenListLoaded: boolean;
  shindenEntryCount: number;
};

export type DatabaseUpdateResponse = {
  status: 'upToDate' | 'updated';
  release: string;
  sha256: string;
  path: string;
};

export type DatabaseLoadResponse = {
  path: string;
  entryCount: number;
  lastUpdate: string;
};

export type ShindenLoadResponse = {
  userId: number;
  entryCount: number;
  entries: ShindenEntry[];
};

export type SearchMode = 'strict' | 'fuzzy';

export type SearchOptions = {
  limit?: number;
  threshold?: number;
  mode?: SearchMode;
};

export type MatchOptions = {
  searchLimit?: number;
  searchThreshold?: number;
  searchMode?: SearchMode;
  candidateLimit?: number;
};

export type SearchResponse = {
  query: string;
  normalizedQuery: string;
  results: SearchResult[];
};

export type MatchResponse = {
  sourceCount: number;
  matchedCount: number;
  results: MatchResult[];
};

export type ExportResponse = {
  xml: string;
  exportedCount: number;
};

export type SearchResult = {
  entry: DatabaseEntry;
  searchScore: number;
};

export type MatchResult = {
  source: ShindenEntry;
  winner: MatchedCandidate | null;
  candidates: MatchedCandidate[];
};

export type MatchedCandidate = {
  entry: DatabaseEntry;
  scores: ScoreBreakdown;
};

export type ScoreBreakdown = {
  searchScore: number;
  seasonScore: number;
  yearScore: number;
  typeScore: number;
  statusScore: number;
  seasonalScore: number;
  episodesScore: number;
  finalScore: number;
};

export type DatabaseEntry = {
  id: number;
  title: string;
  animeType: string;
  episodes: number;
  status: string;
  season: string;
  year: number | null;
  picture: string;
  thumbnail: string;
  duration: number | null;
  synonyms: string[];
  studios: string[];
  producers: string[];
  tags: string[];
  sources: string[];
};

export type ShindenEntry = {
  id: number;
  title: string;
  normalizedTitle: string;
  animeStatus: string;
  animeType: string;
  premiereDate: string | null;
  finishDate: string | null;
  episodes: number | null;
  isFavourite: boolean;
  watchStatus: string;
  watchedEpisodes: number;
  score: number | null;
  note: string | null;
  coverId: number | null;
};

export type SelectedMatch = {
  sourceId: number;
  databaseId: number;
};

export function getAppStatus() {
  return invoke<AppStatus>('app_status');
}

export function updateDatabase(databasePath?: string) {
  return invoke<DatabaseUpdateResponse>('update_database', { databasePath });
}

export function loadDatabase(databasePath?: string) {
  return invoke<DatabaseLoadResponse>('load_database', { databasePath });
}

export function searchDatabase(query: string, options?: SearchOptions) {
  return invoke<SearchResponse>('search_database', { query, options });
}

export function fetchShindenList(userId: number) {
  return invoke<ShindenLoadResponse>('fetch_shinden_list', { userId });
}

export function matchShindenList(options?: MatchOptions) {
  return invoke<MatchResponse>('match_shinden_list', { options });
}

export function exportMatchesXml(matches: SelectedMatch[]) {
  return invoke<ExportResponse>('export_matches_xml', { matches });
}

export function listenToPipelineProgress(
  handler: (event: PipelineProgressEvent) => void
): Promise<UnlistenFn> {
  return listen<PipelineProgressEvent>(PIPELINE_PROGRESS_EVENT, (event) => {
    handler(event.payload);
  });
}
