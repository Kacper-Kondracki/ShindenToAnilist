<script lang="ts">
  import type { ShindenEntry } from "../../domain/anime";

  export type AnimeMatchStatus = "matched" | "review" | "unmatched";

  let {
    entry,
    matchStatus,
  }: { entry: ShindenEntry; matchStatus: AnimeMatchStatus } = $props();

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

  const matchStatusLabels: Record<AnimeMatchStatus, string> = {
    matched: "Dobrano automatycznie",
    review: "Znaleziono kandydatów do sprawdzenia",
    unmatched: "Nie znaleziono kandydatów",
  };

  function formatPremiereYear(premiereDate: string | null) {
    if (!premiereDate) {
      return "Nieznany rok";
    }

    return premiereDate.slice(0, 4);
  }

  function translateAnimeType(animeType: string) {
    return animeTypeLabels[animeType] ?? animeType;
  }

  function translateAnimeStatus(animeStatus: string) {
    return animeStatusLabels[animeStatus] ?? animeStatus;
  }
</script>

<article
  class:anime-row--matched={matchStatus === "matched"}
  class:anime-row--review={matchStatus === "review"}
  class:anime-row--unmatched={matchStatus === "unmatched"}
  class="anime-row"
  aria-label={`${entry.title}: ${matchStatusLabels[matchStatus]}`}
  title={matchStatusLabels[matchStatus]}
>
  <div class="min-w-0">
    <h2 class="truncate text-sm font-semibold">{entry.title}</h2>
    <p class="truncate text-xs text-muted">
      {formatPremiereYear(entry.premiereDate)} · {translateAnimeType(
        entry.animeType,
      )}
      · {translateAnimeStatus(entry.animeStatus)}
    </p>
  </div>

  {#if entry.score !== null}
    <span class="badge shrink-0 badge-soft badge-info">
      {entry.score}/10
    </span>
  {/if}
</article>

<style>
  .anime-row {
    --match-indicator-color: var(--color-error);

    display: flex;
    position: relative;
    min-height: 4.5rem;
    align-items: center;
    justify-content: space-between;
    gap: calc(var(--spacing) * 3);
    border-bottom: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 8%, transparent);
    padding-inline: calc(var(--spacing) * 4);
    padding-left: calc(var(--spacing) * 5);
    padding-block: calc(var(--spacing) * 3);
  }

  .anime-row::before {
    position: absolute;
    inset-block: calc(var(--spacing) * 2);
    left: calc(var(--spacing) * 1);
    width: 0.375rem;
    border-radius: 999px;
    background-color: var(--match-indicator-color);
    box-shadow: 0 0 0 1px
      color-mix(in oklab, var(--match-indicator-color) 38%, transparent);
    content: "";
  }

  .anime-row--matched {
    --match-indicator-color: var(--color-success);
  }

  .anime-row--review {
    --match-indicator-color: var(--color-warning);
  }

  .anime-row--unmatched {
    --match-indicator-color: var(--color-error);
  }
</style>
