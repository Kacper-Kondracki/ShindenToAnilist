import type { DatabaseEntry } from '../../domain/anime';
import {
  formatEpisodeCount,
  formatEpisodeDuration,
  formatYear,
  translateAnimeStatus,
  translateAnimeType,
  translateSeason
} from '../../domain/animeView';
import { AnimeStatus } from '../../gen/shinden_to_anilist/v1/common_pb';

const fallbackCoverHeight = 172;

type DatabaseEntryPreviewControllerInput = {
  getEntry: () => DatabaseEntry;
  getCompact: () => boolean;
};

export type DatabaseEntryPreviewController = ReturnType<
  typeof createDatabaseEntryPreviewController
>;

type CoverLoadState =
  | { status: 'idle' }
  | { status: 'loaded'; url: string }
  | { status: 'error'; url: string };

type MetadataTone =
  | 'year'
  | 'anime-type'
  | 'status-finished'
  | 'status-ongoing'
  | 'status-upcoming'
  | 'status-unknown'
  | 'season'
  | 'episodes'
  | 'duration';

type MetadataItem = {
  label: string;
  value: string;
  tone: MetadataTone;
};

export function createDatabaseEntryPreviewController(
  input: DatabaseEntryPreviewControllerInput
) {
  let coverLoadState = $state<CoverLoadState>({ status: 'idle' });
  let detailsHeight = $state(fallbackCoverHeight);
  let metadataHeight = $state(fallbackCoverHeight);

  let coverUrl = $derived(
    input.getEntry().picture || input.getEntry().thumbnail
  );
  let isCoverLoaded = $derived(
    coverLoadState.status === 'loaded' && coverLoadState.url === coverUrl
  );
  let hasCoverError = $derived(
    coverLoadState.status === 'error' && coverLoadState.url === coverUrl
  );
  let coverSizingHeight = $derived(
    input.getCompact() ? metadataHeight : detailsHeight
  );
  let coverHeight = $derived(`${coverSizingHeight}px`);
  let coverWidth = $derived(`${coverSizingHeight * 0.75}px`);
  let metadataItems = $derived.by(() => {
    const entry = input.getEntry();

    return [
      {
        label: 'Rok',
        value: formatYear(entry.year),
        tone: 'year'
      },
      {
        label: 'Typ',
        value: translateAnimeType(entry.animeType),
        tone: 'anime-type'
      },
      {
        label: 'Status',
        value: translateAnimeStatus(entry.status),
        tone: statusTone(entry.status)
      },
      {
        label: 'Sezon',
        value: translateSeason(entry.season),
        tone: 'season'
      },
      {
        label: 'Liczba odcinków',
        value: formatEpisodeCount(entry.episodes),
        tone: 'episodes'
      },
      {
        label: 'Czas odcinka',
        value: formatEpisodeDuration(entry.duration),
        tone: 'duration'
      }
    ] satisfies MetadataItem[];
  });

  function handleCoverLoad() {
    if (coverUrl) {
      coverLoadState = { status: 'loaded', url: coverUrl };
    }
  }

  function handleCoverError() {
    if (coverUrl) {
      coverLoadState = { status: 'error', url: coverUrl };
    }
  }

  return {
    get isCoverLoaded() {
      return isCoverLoaded;
    },
    get hasCoverError() {
      return hasCoverError;
    },
    get detailsHeight() {
      return detailsHeight;
    },
    set detailsHeight(nextHeight: number) {
      if (nextHeight > 0) {
        detailsHeight = nextHeight;
      }
    },
    get metadataHeight() {
      return metadataHeight;
    },
    set metadataHeight(nextHeight: number) {
      if (nextHeight > 0) {
        metadataHeight = nextHeight;
      }
    },
    get coverUrl() {
      return coverUrl;
    },
    get coverHeight() {
      return coverHeight;
    },
    get coverWidth() {
      return coverWidth;
    },
    get metadataItems() {
      return metadataItems;
    },
    handleCoverLoad,
    handleCoverError
  };
}

function statusTone(status: AnimeStatus): MetadataTone {
  switch (status) {
    case AnimeStatus.FINISHED:
      return 'status-finished';
    case AnimeStatus.ONGOING:
      return 'status-ongoing';
    case AnimeStatus.UPCOMING:
      return 'status-upcoming';
    default:
      return 'status-unknown';
  }
}
