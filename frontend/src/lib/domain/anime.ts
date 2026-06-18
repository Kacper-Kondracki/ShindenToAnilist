import type { Provider } from '../config/providers';
import type {
  AnimeStatus,
  AnimeType,
  SourceProvider,
  Season,
  WatchStatus
} from '../gen/shinden_to_anilist/v1/common_pb';

export type DatabaseInfo = {
  lastUpdate: string;
  release: string;
  sha256: string;
  path: string;
  needsUpdate: boolean;
  databaseVersion: number;
};

export type SourceEntry = {
  id: number;
  provider: SourceProvider;
  title: string;
  animeStatus: AnimeStatus;
  animeType: AnimeType;
  premiereDate: string | null;
  year: number | null;
  finishDate: string | null;
  episodes: number | null;
  watchStatus: WatchStatus;
  watchedEpisodes: number;
  score: number | null;
  sourceUrl: string;
  malId: number | null;
  coverId?: number | null;
  isFavourite?: boolean;
};

export type ShindenEntry = SourceEntry;

export type DatabaseEntry = {
  id: number;
  sources: string[];
  title: string;
  animeType: AnimeType;
  episodes: number;
  status: AnimeStatus;
  season: Season;
  year: number | null;
  picture: string;
  thumbnail: string;
  duration: number | null;
  synonyms: string[];
};

export type ShindenListIndex = {
  entryIds: number[];
  shindenVersion: number;
  sourceVersion?: number;
};

export type ShindenListView = 'manual' | 'automatic' | 'ignored' | 'all';

export type ShindenListViews = Record<ShindenListView, number[]>;

export type SearchOptions = {
  limit?: number;
  threshold?: number;
};

export type SearchItem = {
  id: number;
  score: number;
};

export type SearchResult = {
  items: SearchItem[];
  databaseVersion: number;
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
  sourceId?: number;
  result: MatchResult;
};

export type MatchListResult = {
  entries: ShindenMatchResult[];
  total: number;
  winners: number;
  hasTop: number;
  unmatched: number;
  shindenVersion: number;
  sourceVersion?: number;
  databaseVersion: number;
};

export type MatchSelection = {
  sourceId: number;
  shindenId?: number;
  databaseId: number;
};

export type ExportResult = {
  path: string;
  exportedCount: number;
  cancelled: boolean;
  shindenVersion: number;
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

export type SourceImportProgress = {
  phase: number;
  current: number;
  total: number;
  latestTitle: string;
  startedAt: number;
};

export type UserListRequestState =
  | { status: 'idle' }
  | {
      status: 'loading';
      provider: Provider;
      query: string;
      progress: SourceImportProgress | null;
    }
  | ({ status: 'loaded' } & LoadedUserList)
  | { status: 'error'; provider: Provider; query: string; message: string };

export type WorkspaceState =
  | { status: 'empty' }
  | ({ status: 'active' } & LoadedUserList);
