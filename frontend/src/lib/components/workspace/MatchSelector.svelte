<script lang="ts">
  import type { DatabaseEntry, ShindenEntry } from '../../domain/anime';
  import { formatPercentageFromRatio } from '../../domain/animeView';
  import {
    createMatchSelectorController,
    type MatchSelectorInitialSearch
  } from '../../features/workspace/matchSelectorController.svelte';
  import DatabaseEntryRow from './DatabaseEntryRow.svelte';

  let {
    selectedEntry,
    selectedDatabaseEntryId,
    manualOverrideId,
    initialSearch,
    getDatabaseEntry,
    onSetManualOverride,
    onClearManualOverride
  }: {
    selectedEntry: ShindenEntry;
    selectedDatabaseEntryId: number | null;
    manualOverrideId: number | null;
    initialSearch: MatchSelectorInitialSearch | null;
    getDatabaseEntry: (entryId: number) => DatabaseEntry | null;
    onSetManualOverride: (shindenId: number, databaseId: number) => void;
    onClearManualOverride: (shindenId: number) => void;
  } = $props();

  const selector = createMatchSelectorController({
    getSelectedEntry: () => selectedEntry,
    getDatabaseEntry: (entryId) => getDatabaseEntry(entryId),
    getInitialSearch: () => initialSearch,
    setManualOverride: (shindenId, databaseId) =>
      onSetManualOverride(shindenId, databaseId),
    clearManualOverride: (shindenId) => onClearManualOverride(shindenId)
  });

  function handleQueryInput(event: Event) {
    selector.updateQuery((event.currentTarget as HTMLInputElement).value);
  }

  function formatMatchScore(score: number) {
    return formatPercentageFromRatio(score);
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
    {#if manualOverrideId !== null}
      <button
        type="button"
        class="btn btn-ghost btn-sm"
        onclick={selector.clearManualOverride}
      >
        Wyczyść ręczny wybór
      </button>
    {/if}
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
            <DatabaseEntryRow
              entry={result.entry}
              scoreLabel={formatMatchScore(result.score)}
              isSelected={result.id === selectedDatabaseEntryId}
              showIndicator={false}
              rounded={true}
              compact={true}
              onSelect={() => selector.applyManualOverride(result.id)}
            />
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  .match-selector {
    --match-selector-panel-bg: var(--color-base-200);
    --match-selector-border-color: color-mix(
      in oklab,
      var(--color-base-content) 12%,
      transparent
    );

    display: flex;
    width: 100%;
    max-width: 100%;
    height: 100%;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    flex-direction: column;
    padding: calc(var(--spacing) * 3);
  }
  .search-input {
    min-width: 0;
    flex: 1 1 auto;
  }
  .search-box {
    display: flex;
    min-width: 0;
    flex: 0 0 auto;
    align-items: center;
    gap: calc(var(--spacing) * 2);
    border: var(--border) solid var(--match-selector-border-color);
    border-bottom: 0;
    border-radius: var(--radius-box) var(--radius-box) 0 0;
    background-color: var(--match-selector-panel-bg);
    padding: calc(var(--spacing) * 2);
  }
  .search-content {
    flex: 1 1 auto;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    border: var(--border) solid var(--match-selector-border-color);
    border-radius: 0 0 var(--radius-box) var(--radius-box);
    background-color: var(--match-selector-panel-bg);
  }

  .search-message {
    padding: calc(var(--spacing) * 3);
  }

  .match-results {
    display: flex;
    box-sizing: border-box;
    height: 100%;
    min-width: 0;
    flex-direction: column;
    margin: 0;
    padding: calc(var(--spacing) * 1);
    overflow-y: auto;
    list-style: none;
    scrollbar-color: var(--color-primary) var(--match-selector-panel-bg);
  }

  .match-result {
    min-width: 0;
  }
</style>
