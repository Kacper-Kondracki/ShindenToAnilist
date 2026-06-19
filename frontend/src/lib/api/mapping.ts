import { SourceProvider } from '../gen/shinden_to_anilist/v1/common_pb';
import type {
  DatabaseEntry as ProtoDatabaseEntry,
  DatabaseMetadata,
  DatabaseReleaseInfo
} from '../gen/shinden_to_anilist/v1/database_pb';
import type {
  MatchResult as ProtoMatchResult,
  ShindenMatchResult as ProtoShindenMatchResult,
  SourceMatchResult as ProtoSourceMatchResult
} from '../gen/shinden_to_anilist/v1/matching_pb';
import type { ShindenEntry as ProtoShindenEntry } from '../gen/shinden_to_anilist/v1/shinden_pb';
import type {
  SourceEntry as ProtoSourceEntry,
  SourceFetchProgress as ProtoSourceFetchProgress
} from '../gen/shinden_to_anilist/v1/source_pb';
import type {
  DatabaseEntry,
  DatabaseInfo,
  MatchListResult,
  MatchResult,
  SourceEntry,
  WireNumber
} from '../domain/anime';
import {
  formatProtoDate,
  toSafeNumber,
  toWireNumber,
  type WireDate,
  type WireNumberInput
} from './wire';

export type AppDatabaseReleaseInfo = Pick<
  DatabaseReleaseInfo,
  'release' | 'sha256'
> & {
  compressedSize?: WireNumberInput;
};

export type AppDatabaseMetadata = {
  lastUpdate?: WireDate;
};

export type AppDatabaseUpdateCheck = {
  local?: AppDatabaseReleaseInfo | null;
  remote?: AppDatabaseReleaseInfo | null;
  needsUpdate: boolean;
};

export type TauriShindenEntry = {
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

export type TauriSourceEntry = {
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

export type TauriDatabaseEntry = {
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

export type TauriMatchResult = {
  id: WireNumberInput;
  finalScore: number;
};

type WireMatchResult = ProtoMatchResult | TauriMatchResult;

export type TauriShindenMatchResult = {
  shindenId: WireNumberInput;
  candidates: TauriMatchResult[];
  topCandidates: TauriMatchResult[];
  winner?: TauriMatchResult | null;
};

export type WireShindenMatchResult =
  | ProtoShindenMatchResult
  | TauriShindenMatchResult;

export type TauriSourceMatchResult = {
  sourceId: WireNumberInput;
  candidates: TauriMatchResult[];
  topCandidates: TauriMatchResult[];
  winner?: TauriMatchResult | null;
};

export type WireSourceMatchResult =
  | ProtoSourceMatchResult
  | TauriSourceMatchResult;

export type TauriSearchOptions = {
  limit: number;
  threshold?: number;
};

export type SourceFetchProgress = {
  provider: SourceProvider;
  phase: number;
  current: number;
  total: number;
  latestTitle: string;
};

export type TauriSourceFetchProgress = {
  provider: number;
  phase: number;
  current: WireNumberInput;
  total: WireNumberInput;
  latestTitle: string;
};

export function toDatabaseInfo(input: {
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

export function toShindenEntry(
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

export function toSourceEntry(
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

export function toDatabaseEntry(
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

export function toMatchListResult(
  entries: WireShindenMatchResult[],
  nextShindenVersion: WireNumberInput,
  nextDatabaseVersion: WireNumberInput
): MatchListResult {
  const mappedEntries = entries.map((entry) => ({
    sourceId: toWireNumber(entry.shindenId),
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

export function toSourceMatchListResult(
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

export function toSourceFetchProgress(
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

export function toMatchResult(
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
