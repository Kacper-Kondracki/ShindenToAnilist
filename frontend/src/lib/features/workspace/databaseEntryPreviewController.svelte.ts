import type { DatabaseEntry } from "../../domain/anime";
import {
  formatEpisodeCount,
  formatEpisodeDuration,
  formatYear,
  translateAnimeStatus,
  translateAnimeType,
  translateSeason,
} from "../../domain/animeView";

const fallbackCoverHeight = 172;

type DatabaseEntryPreviewControllerInput = {
  getEntry: () => DatabaseEntry;
};

export type DatabaseEntryPreviewController = ReturnType<
  typeof createDatabaseEntryPreviewController
>;

export function createDatabaseEntryPreviewController(
  input: DatabaseEntryPreviewControllerInput,
) {
  let isCoverLoaded = $state(false);
  let hasCoverError = $state(false);
  let detailsHeight = $state(fallbackCoverHeight);

  let coverUrl = $derived(
    input.getEntry().picture || input.getEntry().thumbnail,
  );
  let coverHeight = $derived(`${detailsHeight}px`);
  let coverWidth = $derived(`${detailsHeight * 0.75}px`);
  let metadataItems = $derived.by(() => {
    const entry = input.getEntry();

    return [
      {
        label: "Rok",
        value: formatYear(entry.year),
      },
      {
        label: "Typ",
        value: translateAnimeType(entry.animeType),
      },
      {
        label: "Status",
        value: translateAnimeStatus(entry.status),
      },
      {
        label: "Sezon",
        value: translateSeason(entry.season),
      },
      {
        label: "Liczba odcinków",
        value: formatEpisodeCount(entry.episodes),
      },
      {
        label: "Czas odcinka",
        value: formatEpisodeDuration(entry.duration),
      },
    ];
  });

  $effect(() => {
    coverUrl;
    resetCoverState();
  });

  function resetCoverState() {
    isCoverLoaded = false;
    hasCoverError = false;
  }

  function handleCoverLoad() {
    isCoverLoaded = true;
  }

  function handleCoverError() {
    hasCoverError = true;
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
    handleCoverError,
  };
}
