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
      <ul class="match-results overflow-y-auto" aria-label="Wyniki dopasowania">
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
    display: flex;
    height: 100%;
    min-height: 0;
    flex-direction: column;
    gap: calc(var(--spacing) * 3);
    padding: calc(var(--spacing) * 3);
  }
  .search-input {
    min-width: 0;
    flex: 1 1 auto;
  }
  .search-box {
    display: flex;
    flex: 0 0 auto;
    align-items: center;
    gap: calc(var(--spacing) * 2);
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
    min-width: 0;
  }
</style>
