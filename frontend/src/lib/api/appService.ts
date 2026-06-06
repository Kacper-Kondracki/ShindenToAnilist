import { AppService } from "../../../bindings/shindentoanilist";
import type {
  AnimeDatabase,
  DatabaseInfo,
  ExportResult,
  MatchListResult,
  MatchOptions,
  MatchQueryOptions,
  MatchResult,
  MatchSelection,
  SearchOptions,
  SearchResult,
  ShindenList,
} from "../domain/anime";

export async function ensureDatabase() {
  return (await AppService.EnsureDatabase()) as DatabaseInfo;
}

export async function loadShindenList(userId: number) {
  return (await AppService.LoadShindenList(userId)) as ShindenList;
}

export async function getAnimeDatabase() {
  return (await AppService.GetAnimeDatabase()) as AnimeDatabase;
}

export async function matchLoadedShindenList(options: MatchOptions = {}) {
  return (await AppService.MatchLoadedShindenList({
    candidateLimit: options.candidateLimit ?? 0,
    searchThreshold: options.searchThreshold ?? 0,
    resultLimit: options.resultLimit ?? null,
  })) as MatchListResult;
}

export async function searchAnime(query: string, options: SearchOptions = {}) {
  return (await AppService.SearchAnime(query, {
    mode: options.mode ?? "",
    limit: options.limit ?? 0,
    threshold: options.threshold ?? 0,
  })) as SearchResult;
}

export async function matchQuery(
  query: string,
  options: MatchQueryOptions = {},
) {
  const search = options.search ?? {};

  return (await AppService.MatchQuery(query, {
    search: {
      mode: search.mode ?? "",
      limit: search.limit ?? 0,
      threshold: search.threshold ?? 0,
    },
    resultLimit: options.resultLimit ?? null,
  })) as MatchResult;
}

export async function exportMatches(matches: MatchSelection[]) {
  return (await AppService.ExportMatches(matches)) as ExportResult;
}
