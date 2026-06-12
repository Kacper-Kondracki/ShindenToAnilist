<script lang="ts">
  import type { ShindenEntry } from '../../domain/anime';
  import {
    formatPremiereYear,
    translateAnimeStatus,
    translateAnimeType
  } from '../../domain/animeView';

  export type AnimeMatchStatus = 'matched' | 'review' | 'unmatched';

  let {
    entry,
    matchStatus,
    isSelected,
    onSelect
  }: {
    entry: ShindenEntry;
    matchStatus: AnimeMatchStatus;
    isSelected: boolean;
    onSelect: () => void;
  } = $props();

  const matchStatusLabels: Record<AnimeMatchStatus, string> = {
    matched: 'Dobrano automatycznie',
    review: 'Znaleziono kandydatów do sprawdzenia',
    unmatched: 'Nie znaleziono kandydatów'
  };
</script>

<button
  type="button"
  class:anime-row--matched={matchStatus === 'matched'}
  class:anime-row--review={matchStatus === 'review'}
  class:anime-row--unmatched={matchStatus === 'unmatched'}
  class:anime-row--selected={isSelected}
  class="anime-row h-0"
  aria-label={`${entry.title}: ${matchStatusLabels[matchStatus]}`}
  aria-pressed={isSelected}
  title={entry.title}
  onclick={onSelect}
>
  <div class="size-full flex pl-2 pr-2 justify-between items-center">
    <div class="min-w-0">
      <h2 class="truncate text-sm font-semibold">{entry.title}</h2>
      <p class="truncate text-xs text-muted">
        {formatPremiereYear(entry.premiereDate)} · {translateAnimeType(
          entry.animeType
        )}
        · {translateAnimeStatus(entry.animeStatus)}
      </p>
    </div>
  </div>
</button>

<style>
  .anime-row {
    --match-indicator-color: var(--color-error);
    --row-separator-color: color-mix(
      in oklab,
      var(--color-base-content) 8%,
      transparent
    );

    display: flex;
    position: relative;
    width: 100%;
    min-height: 4.5rem;
    align-items: center;
    justify-content: space-between;
    gap: calc(var(--spacing) * 3);
    border-left: 0;
    border-right: 0;
    border-top: 0;
    background-color: transparent;
    background-image:
      linear-gradient(var(--row-separator-color), var(--row-separator-color)),
      linear-gradient(var(--row-separator-color), var(--row-separator-color));
    background-position:
      top left,
      bottom left;
    background-repeat: no-repeat;
    background-size:
      100% var(--border),
      100% var(--border);
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
    padding-inline: calc(var(--spacing) * 2);
    padding-left: calc(var(--spacing) * 4);
    padding-block: calc(var(--spacing) * 2);
    transition:
      background-color 160ms ease,
      box-shadow 160ms ease;
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
    content: '';
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

  .anime-row:hover {
    background-color: color-mix(
      in oklab,
      var(--color-base-content) 5%,
      transparent
    );
  }

  .anime-row:focus-visible {
    outline: 1px solid
      color-mix(in oklab, var(--match-indicator-color) 80%, white);
    outline-offset: -1px;
  }

  .anime-row--selected {
    background-color: color-mix(
      in oklab,
      var(--match-indicator-color) 13%,
      transparent
    );
    box-shadow: inset 0 0 0 1px
      color-mix(in oklab, var(--match-indicator-color) 30%, transparent);
  }

  .anime-row--selected:hover {
    background-color: color-mix(
      in oklab,
      var(--match-indicator-color) 17%,
      transparent
    );
  }
</style>
