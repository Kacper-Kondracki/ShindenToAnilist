<script lang="ts">
  import type { DatabaseEntry, ShindenEntry } from "../../domain/anime";

  let {
    selectedEntry,
    selectedWinner,
  }: {
    selectedEntry: ShindenEntry | null;
    selectedWinner: DatabaseEntry | null;
  } = $props();

  const missingValueText = "Brak danych";

  const animeTypeLabels: Record<string, string> = {
    tv: "Serial TV",
    movie: "Film",
    ova: "OVA",
    ona: "ONA",
    special: "Odcinek specjalny",
    unknown: "Nieznany typ",
  };

  const animeStatusLabels: Record<string, string> = {
    finished: "Zakończone",
    ongoing: "Emitowane",
    upcoming: "Zapowiedziane",
    unknown: "Nieznany status",
  };

  const seasonLabels: Record<string, string> = {
    winter: "Zima",
    spring: "Wiosna",
    summer: "Lato",
    fall: "Jesień",
    unknown: "Nieznany sezon",
  };

  let coverUrl = $derived.by(() => {
    if (selectedWinner === null) {
      return "";
    }

    return selectedWinner.picture || selectedWinner.thumbnail;
  });

  let metadataItems = $derived.by(() => {
    if (selectedWinner === null) {
      return [];
    }

    return [
      {
        label: "Rok",
        value: formatYear(selectedWinner.year),
      },
      {
        label: "Typ",
        value: translateAnimeType(selectedWinner.animeType),
      },
      {
        label: "Status",
        value: translateAnimeStatus(selectedWinner.status),
      },
      {
        label: "Sezon",
        value: translateSeason(selectedWinner.season),
      },
      {
        label: "Liczba odcinków",
        value: formatEpisodeCount(selectedWinner.episodes),
      },
      {
        label: "Czas odcinka",
        value: formatEpisodeDuration(selectedWinner.duration),
      },
    ];
  });

  function formatYear(year: number | null) {
    return year === null ? missingValueText : String(year);
  }

  function formatEpisodeCount(episodeCount: number | null) {
    if (episodeCount === null || episodeCount <= 0) {
      return missingValueText;
    }

    return String(episodeCount);
  }

  function formatEpisodeDuration(duration: number | null) {
    if (duration === null || duration <= 0) {
      return missingValueText;
    }

    return `${duration} min`;
  }

  function translateAnimeType(animeType: string) {
    return animeTypeLabels[animeType.toLowerCase()] ?? animeType;
  }

  function translateAnimeStatus(animeStatus: string) {
    return animeStatusLabels[animeStatus.toLowerCase()] ?? animeStatus;
  }

  function translateSeason(season: string) {
    if (!season) {
      return missingValueText;
    }

    return seasonLabels[season.toLowerCase()] ?? season;
  }
</script>

<section class="workspace-pane" aria-label="Editor">
  <div class="workspace-pane__body">
    {#if selectedEntry === null}
      <p class="workspace-empty text-sm font-medium text-muted">
        Wybierz wpis z listy
      </p>
    {:else if selectedWinner === null}
      <p class="workspace-empty text-sm font-medium text-muted">
        Brak automatycznego dopasowania
      </p>
    {:else}
      <div class="winner-preview">
        <div class="winner-cover" aria-label="Okładka dopasowanego anime">
          {#if coverUrl}
            <img
              src={coverUrl}
              alt={`Okładka: ${selectedWinner.title}`}
              loading="lazy"
              decoding="async"
            />
          {:else}
            <span class="winner-cover__placeholder text-xs font-medium text-muted">
              Brak okładki
            </span>
          {/if}
        </div>

        <div class="winner-details">
          <div class="winner-title">
            <p class="text-xs font-medium text-muted">Tytuł</p>
            <h2 class="text-lg font-semibold">{selectedWinner.title}</h2>
          </div>

          <dl class="winner-metadata">
            {#each metadataItems as item}
              <div class="winner-metadata__item">
                <dt class="text-xs font-medium text-muted">{item.label}</dt>
                <dd class="text-sm font-semibold">{item.value}</dd>
              </div>
            {/each}
          </dl>
        </div>
      </div>
    {/if}
  </div>
</section>

<style>
  .workspace-pane {
    display: flex;
    min-width: 0;
    flex-direction: column;
    overflow: hidden;
    background-color: var(--color-base-300);
  }

  .workspace-pane__body {
    display: block;
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 0;
  }

  .workspace-empty {
    padding: calc(var(--spacing) * 4);
  }

  .winner-preview {
    display: grid;
    grid-template-columns: minmax(7rem, 9rem) minmax(0, 1fr);
    gap: calc(var(--spacing) * 4);
    padding: calc(var(--spacing) * 4);
  }

  .winner-cover {
    display: grid;
    width: 100%;
    aspect-ratio: 2 / 3;
    place-items: center;
    overflow: hidden;
    border: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 12%, transparent);
    border-radius: var(--radius-box);
    background-color: var(--color-base-200);
  }

  .winner-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .winner-cover__placeholder {
    padding: calc(var(--spacing) * 2);
    text-align: center;
  }

  .winner-details {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: calc(var(--spacing) * 4);
  }

  .winner-title {
    min-width: 0;
  }

  .winner-title h2 {
    overflow-wrap: anywhere;
    line-height: 1.35;
  }

  .winner-metadata {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: calc(var(--spacing) * 3);
  }

  .winner-metadata__item {
    min-width: 0;
    border-top: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 10%, transparent);
    padding-top: calc(var(--spacing) * 2);
  }

  .winner-metadata__item dd {
    margin: 0;
    overflow-wrap: anywhere;
  }

  .winner-metadata__item dt {
    margin: 0;
  }

  @media (width <= 36rem) {
    .winner-preview {
      grid-template-columns: minmax(0, 1fr);
    }

    .winner-cover {
      max-width: 10rem;
    }

    .winner-metadata {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
