<script lang="ts">
  import type { DatabaseEntry, ShindenEntry } from '../../domain/anime';
  import {
    formatEpisodeCount,
    formatYear,
    translateAnimeStatus,
    translateAnimeType
  } from '../../domain/animeView';
  import { createMatchSelectorController } from '../../features/workspace/matchSelectorController.svelte';

  let {
    selectedEntry,
    getDatabaseEntry,
    onSetManualOverride,
    onClearManualOverride
  }: {
    selectedEntry: ShindenEntry;
    getDatabaseEntry: (entryId: number) => DatabaseEntry | null;
    onSetManualOverride: (shindenId: number, databaseId: number) => void;
    onClearManualOverride: (shindenId: number) => void;
  } = $props();

  const selector = createMatchSelectorController({
    getSelectedEntry: () => selectedEntry,
    getDatabaseEntry: (entryId) => getDatabaseEntry(entryId),
    setManualOverride: (shindenId, databaseId) =>
      onSetManualOverride(shindenId, databaseId),
    clearManualOverride: (shindenId) => onClearManualOverride(shindenId)
  });

  function handleQueryInput(event: Event) {
    selector.updateQuery((event.currentTarget as HTMLInputElement).value);
  }

  function formatMatchScore(score: number) {
    return `${Math.round(score * 100)}%`;
  }
</script>

<div class="match-selector">
  <div class="search-box">
    <input
      type="text"
      placeholder="Wyszukaj tytuł"
      class="input search-input"
      value={selector.query}
      oninput={handleQueryInput}
    />
  </div>
  <div class="search-content">
    {#if selector.errorMessage !== null}
      <p class="search-message text-sm font-medium text-error">
        {selector.errorMessage}
      </p>
    {:else if selector.results.length === 0}
      <p class="search-message text-sm font-medium text-muted">Brak wyników</p>
    {:else}
      <ul class="match-results" aria-label="Wyniki dopasowania">
        {#each selector.results as result (result.id)}
          <li class="match-result">
            <div class="match-result__main">
              <h3 class="truncate text-sm font-semibold">
                {result.entry.title}
              </h3>
              <p class="truncate text-xs text-muted">
                {formatYear(result.entry.year)} · {translateAnimeType(
                  result.entry.animeType
                )}
                · {translateAnimeStatus(result.entry.status)}
              </p>
            </div>
            <div class="match-result__meta">
              <span class="text-xs font-semibold">
                {formatMatchScore(result.score)}
              </span>
              <span class="text-xs text-muted">
                {formatEpisodeCount(result.entry.episodes)} odc.
              </span>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  .match-selector {
    display: flex;
    min-height: 0;
    flex-direction: column;
    gap: calc(var(--spacing) * 3);
    padding: calc(var(--spacing) * 3);
  }
  .search-input {
    width: 100%;
  }
  .search-box {
    flex: 0 0 auto;
  }
  .search-content {
    min-height: 0;
    overflow: auto;
  }

  .search-message {
    padding: calc(var(--spacing) * 2) calc(var(--spacing) * 1);
  }

  .match-results {
    display: flex;
    flex-direction: column;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .match-result {
    display: flex;
    min-width: 0;
    align-items: center;
    justify-content: space-between;
    gap: calc(var(--spacing) * 3);
    border-bottom: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 10%, transparent);
    padding: calc(var(--spacing) * 3) calc(var(--spacing) * 1);
  }

  .match-result__main {
    min-width: 0;
  }

  .match-result__meta {
    display: flex;
    flex: 0 0 auto;
    min-width: 3.75rem;
    flex-direction: column;
    align-items: flex-end;
    gap: calc(var(--spacing) * 1);
  }
</style>
