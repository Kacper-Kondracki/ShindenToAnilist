import type { DatabaseEntry } from '../../domain/anime';
import {
  formatEpisodeCount,
  formatEpisodeDuration,
  formatYear,
  translateAnimeStatus,
  translateAnimeType,
  translateSeason
} from '../../domain/animeView';

const fallbackCoverHeight = 172;

type DatabaseEntryPreviewControllerInput = {
  getEntry: () => DatabaseEntry;
};

export type DatabaseEntryPreviewController = ReturnType<
  typeof createDatabaseEntryPreviewController
>;

type CoverLoadState =
  | { status: 'idle' }
  | { status: 'loaded'; url: string }
  | { status: 'error'; url: string };

export function createDatabaseEntryPreviewController(
  input: DatabaseEntryPreviewControllerInput
) {
  let coverLoadState = $state<CoverLoadState>({ status: 'idle' });
  let detailsHeight = $state(fallbackCoverHeight);

  let coverUrl = $derived(
    input.getEntry().picture || input.getEntry().thumbnail
  );
  let isCoverLoaded = $derived(
    coverLoadState.status === 'loaded' && coverLoadState.url === coverUrl
  );
  let hasCoverError = $derived(
    coverLoadState.status === 'error' && coverLoadState.url === coverUrl
  );
  let coverHeight = $derived(`${detailsHeight}px`);
  let coverWidth = $derived(`${detailsHeight * 0.75}px`);
  let metadataItems = $derived.by(() => {
    const entry = input.getEntry();

    return [
      {
        label: 'Rok',
        value: formatYear(entry.year)
      },
      {
        label: 'Typ',
        value: translateAnimeType(entry.animeType)
      },
      {
        label: 'Status',
        value: translateAnimeStatus(entry.status)
      },
      {
        label: 'Sezon',
        value: translateSeason(entry.season)
      },
      {
        label: 'Liczba odcinków',
        value: formatEpisodeCount(entry.episodes)
      },
      {
        label: 'Czas odcinka',
        value: formatEpisodeDuration(entry.duration)
      }
    ];
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
      detailsHeight = nextHeight;
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
