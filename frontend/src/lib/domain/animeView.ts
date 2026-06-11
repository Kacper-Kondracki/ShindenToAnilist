import {
  AnimeStatus,
  AnimeType,
  Season
} from '../gen/shinden_to_anilist/v1/common_pb';
import type { DatabaseState } from './anime';

const missingValueText = 'Brak danych';

const animeTypeLabels: Record<AnimeType, string> = {
  [AnimeType.UNSPECIFIED]: missingValueText,
  [AnimeType.TV]: 'Serial TV',
  [AnimeType.MOVIE]: 'Film',
  [AnimeType.OVA]: 'OVA',
  [AnimeType.ONA]: 'ONA',
  [AnimeType.SPECIAL]: 'Odcinek specjalny',
  [AnimeType.UNKNOWN]: 'Nieznany typ'
};

const animeStatusLabels: Record<AnimeStatus, string> = {
  [AnimeStatus.UNSPECIFIED]: missingValueText,
  [AnimeStatus.FINISHED]: 'Zakończone',
  [AnimeStatus.ONGOING]: 'Emitowane',
  [AnimeStatus.UPCOMING]: 'Zapowiedziane',
  [AnimeStatus.UNKNOWN]: 'Nieznany status'
};

const seasonLabels: Record<Season, string> = {
  [Season.UNSPECIFIED]: missingValueText,
  [Season.WINTER]: 'Zima',
  [Season.SPRING]: 'Wiosna',
  [Season.SUMMER]: 'Lato',
  [Season.FALL]: 'Jesień',
  [Season.UNKNOWN]: 'Nieznany sezon'
};

export function formatPremiereYear(premiereDate: string | null) {
  if (!premiereDate) {
    return 'Nieznany rok';
  }

  return premiereDate.slice(0, 4);
}

export function formatYear(year: number | null) {
  return year === null ? missingValueText : String(year);
}

export function formatEpisodeCount(episodeCount: number | null) {
  if (episodeCount === null || episodeCount <= 0) {
    return missingValueText;
  }

  return String(episodeCount);
}

export function formatEpisodeDuration(duration: number | null) {
  if (duration === null || duration <= 0) {
    return missingValueText;
  }

  return `${duration} min`;
}

export function translateAnimeType(animeType: AnimeType) {
  return animeTypeLabels[animeType] ?? missingValueText;
}

export function translateAnimeStatus(animeStatus: AnimeStatus) {
  return animeStatusLabels[animeStatus] ?? missingValueText;
}

export function translateSeason(season: Season) {
  return seasonLabels[season] ?? missingValueText;
}

export function databaseStatusTitle(state: DatabaseState) {
  if (state.status === 'error') {
    return state.message;
  }

  if (state.status === 'ready') {
    return state.info.path;
  }

  return null;
}
