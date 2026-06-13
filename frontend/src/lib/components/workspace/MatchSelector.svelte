<script lang="ts">
  import type {
    DatabaseEntry,
    MatchResult,
    ShindenEntry
  } from '../../domain/anime';
  import type { EntryRowTone } from './EntryRow.svelte';
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
    isIgnored,
    isAutomaticWinnerSuppressed,
    automaticMatchResult,
    initialSearch,
    winnerClaimsByDatabaseId,
    getDatabaseEntry,
    onSetManualOverride,
    onSetIgnored,
    onClearManualOverride
  }: {
    selectedEntry: ShindenEntry;
    selectedDatabaseEntryId: number | null;
    manualOverrideId: number | null;
    isIgnored: boolean;
    isAutomaticWinnerSuppressed: boolean;
    automaticMatchResult: MatchResult | null;
    initialSearch: MatchSelectorInitialSearch | null;
    winnerClaimsByDatabaseId: ReadonlyMap<number, readonly number[]>;
    getDatabaseEntry: (entryId: number) => DatabaseEntry | null;
    onSetManualOverride: (shindenId: number, databaseId: number) => void;
    onSetIgnored: (shindenId: number) => void;
    onClearManualOverride: (shindenId: number) => void;
  } = $props();

  const selector = createMatchSelectorController({
    getSelectedEntry: () => selectedEntry,
    getDatabaseEntry: (entryId) => getDatabaseEntry(entryId),
    getAutomaticMatchResult: () => automaticMatchResult,
    getInitialSearch: () => initialSearch,
    getWinnerClaimsByDatabaseId: () => winnerClaimsByDatabaseId,
    setManualOverride: (shindenId, databaseId) =>
      onSetManualOverride(shindenId, databaseId),
    setIgnored: (shindenId) => onSetIgnored(shindenId),
    clearManualOverride: (shindenId) => onClearManualOverride(shindenId)
  });

  function handleQueryInput(event: Event) {
    selector.updateQuery((event.currentTarget as HTMLInputElement).value);
  }

  function formatMatchScore(score: number) {
    return formatPercentageFromRatio(score);
  }

  function resultTone(databaseId: number): EntryRowTone {
    if (databaseId === selectedDatabaseEntryId && manualOverrideId !== null) {
      return 'info';
    }

    return databaseId === selectedDatabaseEntryId ? 'matched' : 'neutral';
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
    <button
      type="button"
      class="btn btn-primary btn-soft border-0 btn-square btn-sm clear-manual-override-button"
      aria-label="Wyczyść ręczną decyzję"
      title="Wyczyść ręczną decyzję"
      disabled={manualOverrideId === null &&
        !isIgnored &&
        !isAutomaticWinnerSuppressed}
      onclick={selector.clearManualOverride}
    >
      <span aria-hidden="true" class="icon-[lucide--rotate-ccw] size-4"></span>
    </button>
    <button
      type="button"
      class:btn-active={isIgnored}
      class="btn btn-neutral btn-soft border-0 btn-square btn-sm ignore-entry-button"
      aria-label="Ignoruj wpis"
      title="Ignoruj wpis"
      onclick={selector.applyIgnore}
    >
      <span aria-hidden="true" class="icon-[lucide--eye-off] size-4"></span>
    </button>
  </div>
  <div class="search-content">
    {#if selector.hasResults}
      <ul class="match-results" aria-label="Wyniki dopasowania">
        {#each selector.automaticResults as result (result.id)}
          <li class="match-result">
            <DatabaseEntryRow
              entry={result.entry}
              scoreLabel={formatMatchScore(result.score)}
              isSelected={result.id === selectedDatabaseEntryId}
              tone={resultTone(result.id)}
              softWarning={selector.conflictingWinnerIds.has(result.id)}
              showIndicator={true}
              rounded={true}
              compact={true}
              onSelect={() => selector.applyManualOverride(result.id)}
            />
          </li>
        {/each}
        {#if selector.automaticResults.length > 0 && selector.searchResults.length > 0}
          <li class="match-results-separator" aria-hidden="true"></li>
        {/if}
        {#each selector.searchResults as result (result.id)}
          <li class="match-result">
            <DatabaseEntryRow
              entry={result.entry}
              scoreLabel={formatMatchScore(result.score)}
              isSelected={result.id === selectedDatabaseEntryId}
              tone={resultTone(result.id)}
              softWarning={selector.conflictingWinnerIds.has(result.id)}
              showIndicator={false}
              rounded={true}
              compact={true}
              onSelect={() => selector.applyManualOverride(result.id)}
            />
          </li>
        {/each}
      </ul>
    {:else if selector.errorMessage !== null}
      <p class="search-message text-sm font-medium text-error">
        {selector.errorMessage}
      </p>
    {:else}
      <p class="search-message text-sm font-medium text-muted">Brak wyników</p>
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

  .clear-manual-override-button,
  .ignore-entry-button {
    flex: 0 0 auto;
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

  .match-results-separator {
    min-width: 0;
    margin: calc(var(--spacing) * 1.5) calc(var(--spacing) * 2);
    border-top: var(--border) solid var(--match-selector-border-color);
  }
</style>
