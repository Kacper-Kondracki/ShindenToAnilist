import type { Provider } from '../config/providers';

export type DatabaseInfo = {
  lastUpdate: string;
  release: string;
  sha256: string;
  path: string;
  updated: boolean;
};

export type ShindenEntry = {
  id: number;
  coverId: number | null;
  title: string;
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
  description: string | null;
};

export type ShindenList = {
  entries: ShindenEntry[];
};

export type ShindenListIndex = {
  entryIds: number[];
};

export type ShindenListView = 'manual' | 'automatic' | 'all';

export type ShindenListViews = Record<ShindenListView, number[]>;

export type TitleMetadata = {
  season: number | null;
  part: number | null;
  episode: number | null;
  hasSeasonKeyword: boolean;
  hasPartKeyword: boolean;
  hasEpisodeKeyword: boolean;
};

export type ConsolidatedMetadata = {
  season: number | null;
  part: number | null;
  episode: number | null;
  isFinalSeason: boolean;
  isFinalPart: boolean;
  isFinalEpisode: boolean;
};

export type DatabaseEntry = {
  id: number;
  consolidatedMetadata: ConsolidatedMetadata;
  sources: string[];
  title: string;
  normalizedTitle: string;
  metadata: TitleMetadata;
  animeType: string;
  episodes: number;
  status: string;
  season: string;
  year: number | null;
  picture: string;
  thumbnail: string;
  duration: number | null;
  synonyms: string[];
  normalizedSynonyms: string[];
  studios: string[];
  producers: string[];
  relatedAnime: string[];
  tags: string[];
};

export type SearchMode = 'strict' | 'fuzzy' | '';

export type SearchOptions = {
  mode?: SearchMode;
  limit?: number;
  threshold?: number;
};

export type MatchOptions = {
  candidateLimit?: number;
  searchThreshold?: number;
  resultLimit?: number | null;
};

export type MatchQueryOptions = {
  search?: SearchOptions;
  resultLimit?: number | null;
};

export type SearchItem = {
  id: number;
  score: number;
};

export type SearchResult = {
  items: SearchItem[];
};

export type ScoredCandidate = {
  id: number;
  score: number;
};

export type MatchResult = {
  items: ScoredCandidate[];
  top: ScoredCandidate[];
  winner: ScoredCandidate | null;
};

export type ShindenMatchResult = {
  shindenId: number;
  result: MatchResult;
};

export type MatchListResult = {
  entries: ShindenMatchResult[];
  total: number;
  winners: number;
  hasTop: number;
  unmatched: number;
};

export type MatchSelection = {
  shindenId: number;
  databaseId: number;
};

export type ExportResult = {
  path: string;
  exportedCount: number;
  cancelled: boolean;
};

export type LoadedUserList = {
  provider: Provider;
  query: string;
  entryIdsByView: ShindenListViews;
};

export type DatabaseState =
  | { status: 'loading' }
  | { status: 'ready'; info: DatabaseInfo }
  | { status: 'error'; message: string };

export type UserListRequestState =
  | { status: 'idle' }
  | { status: 'loading'; provider: Provider; query: string }
  | ({ status: 'loaded' } & LoadedUserList)
  | { status: 'error'; provider: Provider; query: string; message: string };

export type WorkspaceState =
  | { status: 'empty' }
  | ({ status: 'active' } & LoadedUserList);
