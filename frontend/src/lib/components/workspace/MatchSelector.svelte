<script lang="ts">
  import { flushSync, untrack } from 'svelte';
  import type {
    DatabaseEntry,
    MatchResult,
    ShindenEntry,
    WireNumber
  } from '../../domain/anime';
  import { wireNumberEquals } from '../../domain/anime';
  import type { EntryRowTone } from './EntryRow.svelte';
  import { formatPercentageFromRatio } from '../../domain/animeView';
  import {
    createMatchSelectorController,
    type MatchSelectorInitialSearch
  } from '../../features/workspace/matchSelectorController.svelte';
  import DatabaseEntryRow from './DatabaseEntryRow.svelte';
  import WorkspaceDialog from './WorkspaceDialog.svelte';
  import {
    copyText,
    databaseEntryMalUrl,
    openExternalUrl
  } from './contextMenuActions';
  import type { ContextMenuItem } from './contextMenuState.svelte';

  let {
    selectedEntry,
    selectedDatabaseEntryId,
    manualOverrideId,
    isIgnored,
    isAutomaticWinnerSuppressed,
    rememberedQuery,
    automaticMatchResult,
    initialSearch,
    winnerClaimsByDatabaseId,
    getDatabaseEntry,
    getShindenEntry,
    onSetManualOverride,
    onSetIgnored,
    onClearManualOverride,
    onSetMatchSelectorQuery,
    onResetMatchSelectorQuery,
    onGoToEntry
  }: {
    selectedEntry: ShindenEntry;
    selectedDatabaseEntryId: WireNumber | null;
    manualOverrideId: WireNumber | null;
    isIgnored: boolean;
    isAutomaticWinnerSuppressed: boolean;
    rememberedQuery: string;
    automaticMatchResult: MatchResult | null;
    initialSearch: MatchSelectorInitialSearch | null;
    winnerClaimsByDatabaseId: ReadonlyMap<WireNumber, readonly WireNumber[]>;
    getDatabaseEntry: (entryId: WireNumber) => DatabaseEntry | null;
    getShindenEntry: (entryId: WireNumber) => ShindenEntry | null;
    onSetManualOverride: (
      shindenId: WireNumber,
      databaseId: WireNumber
    ) => void;
    onSetIgnored: (shindenId: WireNumber) => void;
    onClearManualOverride: (shindenId: WireNumber) => void;
    onSetMatchSelectorQuery: (shindenId: WireNumber, query: string) => void;
    onResetMatchSelectorQuery: (shindenId: WireNumber) => void;
    onGoToEntry: (entryId: WireNumber) => void;
  } = $props();

  const selector = createMatchSelectorController({
    getSelectedEntry: () => selectedEntry,
    getRememberedQuery: () => rememberedQuery,
    getDatabaseEntry: (entryId) => getDatabaseEntry(entryId),
    getAutomaticMatchResult: () => automaticMatchResult,
    getInitialSearch: () => initialSearch,
    getWinnerClaimsByDatabaseId: () => winnerClaimsByDatabaseId,
    setRememberedQuery: (shindenId, query) =>
      onSetMatchSelectorQuery(shindenId, query),
    resetRememberedQuery: (shindenId) => onResetMatchSelectorQuery(shindenId),
    setManualOverride: (shindenId, databaseId) =>
      onSetManualOverride(shindenId, databaseId),
    setIgnored: (shindenId) => onSetIgnored(shindenId),
    clearManualOverride: (shindenId) => onClearManualOverride(shindenId)
  });

  let matchResultsElement = $state<HTMLUListElement | null>(null);
  let searchResultsAnchorElement = $state<HTMLLIElement | null>(null);
  let pendingSearchAlignmentQuery = $state<string | null>(null);
  let pendingAlreadyUsedDatabaseId = $state<WireNumber | null>(null);
  let searchAlignmentSpacerHeight = $state(0);
  let pendingAlreadyUsedOwners = $derived.by(() =>
    pendingAlreadyUsedDatabaseId === null
      ? []
      : conflictOwnerIdsForDatabase(pendingAlreadyUsedDatabaseId).map(
          (ownerId) => ({
            id: ownerId,
            title: getShindenEntry(ownerId)?.title.trim() || `wpis #${ownerId}`
          })
        )
  );

  $effect(() => {
    const pendingQuery = pendingSearchAlignmentQuery;
    const currentQuery = selector.query;
    const isSearchCurrent = selector.isSearchCurrent;

    if (
      pendingQuery === null ||
      pendingQuery !== currentQuery ||
      !isSearchCurrent ||
      matchResultsElement === null
    ) {
      return;
    }

    const resultSignature = getResultSignature();

    untrack(() =>
      alignSearchResultsAfterQueryInput(currentQuery, resultSignature)
    );
  });

  function handleQueryInput(event: Event) {
    const nextQuery = (event.currentTarget as HTMLInputElement).value;

    pendingSearchAlignmentQuery = nextQuery;
    selector.updateQuery(nextQuery);
  }

  function handleQueryKeydown(event: KeyboardEvent) {
    const currentQuery = (event.currentTarget as HTMLInputElement).value;

    if (event.key !== 'Backspace' || currentQuery.length > 0) {
      return;
    }

    pendingSearchAlignmentQuery = null;
    setSearchAlignmentSpacerHeight(0);

    if (matchResultsElement !== null) {
      matchResultsElement.scrollTop = 0;
    }
  }

  function handleReset({ clearQuery = true } = {}) {
    pendingSearchAlignmentQuery = null;
    searchAlignmentSpacerHeight = 0;

    if (clearQuery) {
      selector.resetQuery();
    }

    if (manualOverrideId !== null || isIgnored || isAutomaticWinnerSuppressed) {
      selector.clearManualOverride();
    }
  }

  function formatMatchScore(score: number) {
    return formatPercentageFromRatio(score);
  }

  function resultTone(databaseId: WireNumber): EntryRowTone {
    if (
      selectedDatabaseEntryId !== null &&
      wireNumberEquals(databaseId, selectedDatabaseEntryId) &&
      manualOverrideId !== null
    ) {
      return 'info';
    }

    return selectedDatabaseEntryId !== null &&
      wireNumberEquals(databaseId, selectedDatabaseEntryId)
      ? 'matched'
      : 'neutral';
  }

  function handleResultSelect(databaseId: WireNumber) {
    if (
      selectedDatabaseEntryId !== null &&
      wireNumberEquals(databaseId, selectedDatabaseEntryId)
    ) {
      handleReset({ clearQuery: false });
      return;
    }

    if (
      (selectedDatabaseEntryId === null ||
        !wireNumberEquals(databaseId, selectedDatabaseEntryId)) &&
      conflictOwnerIdsForDatabase(databaseId).length > 0
    ) {
      pendingAlreadyUsedDatabaseId = databaseId;
      return;
    }

    selector.applyManualOverride(databaseId);
  }

  function closeAlreadyUsedWarning() {
    pendingAlreadyUsedDatabaseId = null;
  }

  function confirmAlreadyUsedSelection() {
    const databaseId = pendingAlreadyUsedDatabaseId;

    if (databaseId === null) {
      return;
    }

    pendingAlreadyUsedDatabaseId = null;
    selector.applyManualOverride(databaseId);
  }

  function conflictOwnerIdsForDatabase(databaseId: WireNumber) {
    return (winnerClaimsByDatabaseId.get(databaseId) ?? []).filter(
      (ownerId) => !wireNumberEquals(ownerId, selectedEntry.id)
    );
  }

  function contextMenuItemsForResult(entry: DatabaseEntry): ContextMenuItem[] {
    const ownerIds = conflictOwnerIdsForDatabase(entry.id);
    const items: ContextMenuItem[] = [
      {
        id: 'copy-title',
        label: 'Kopiuj tytuł',
        icon: 'icon-[lucide--copy]',
        onSelect: () => copyText(entry.title)
      },
      {
        id: 'open-mal',
        label: 'Otwórz stronę MAL',
        icon: 'icon-[lucide--external-link]',
        onSelect: () => openExternalUrl(databaseEntryMalUrl(entry))
      }
    ];

    if (ownerIds.length > 0) {
      const ownerId = ownerIds[0];

      if (ownerId === undefined) {
        return items;
      }

      items.push({
        id: 'go-to-owner',
        label: 'Przejdź do wpisu',
        icon: 'icon-[lucide--corner-down-right]',
        dividerBefore: true,
        onSelect: () => onGoToEntry(ownerId)
      });
    }

    return items;
  }

  function alignSearchResultsAfterQueryInput(
    query: string,
    resultSignature: string
  ) {
    if (
      pendingSearchAlignmentQuery !== query ||
      selector.query !== query ||
      getResultSignature() !== resultSignature ||
      matchResultsElement === null
    ) {
      return;
    }

    if (selector.automaticResults.length === 0) {
      setSearchAlignmentSpacerHeight(0);
      matchResultsElement.scrollTop = 0;
      return;
    }

    if (searchResultsAnchorElement === null) {
      return;
    }

    const targetScrollTop = getElementScrollTop(
      matchResultsElement,
      searchResultsAnchorElement
    );
    const nextSpacerHeight =
      getRequiredSearchAlignmentSpacerHeight(targetScrollTop);

    setSearchAlignmentSpacerHeight(nextSpacerHeight);

    if (
      pendingSearchAlignmentQuery === query &&
      selector.query === query &&
      getResultSignature() === resultSignature &&
      matchResultsElement !== null &&
      searchResultsAnchorElement !== null
    ) {
      const alignedTargetScrollTop = getElementScrollTop(
        matchResultsElement,
        searchResultsAnchorElement
      );
      const alignedSpacerHeight = getRequiredSearchAlignmentSpacerHeight(
        alignedTargetScrollTop
      );

      setSearchAlignmentSpacerHeight(alignedSpacerHeight);

      if (
        pendingSearchAlignmentQuery === query &&
        selector.query === query &&
        getResultSignature() === resultSignature &&
        matchResultsElement !== null &&
        searchResultsAnchorElement !== null
      ) {
        matchResultsElement.scrollTop = getElementScrollTop(
          matchResultsElement,
          searchResultsAnchorElement
        );
      }
    }
  }

  function getRequiredSearchAlignmentSpacerHeight(targetScrollTop: number) {
    if (matchResultsElement === null) {
      return 0;
    }

    const maximumScrollTopWithoutSpacer = Math.max(
      0,
      matchResultsElement.scrollHeight -
        searchAlignmentSpacerHeight -
        matchResultsElement.clientHeight
    );

    return Math.ceil(
      Math.max(0, targetScrollTop - maximumScrollTopWithoutSpacer)
    );
  }

  function setSearchAlignmentSpacerHeight(nextSpacerHeight: number) {
    if (searchAlignmentSpacerHeight === nextSpacerHeight) {
      return;
    }

    flushSync(() => {
      searchAlignmentSpacerHeight = nextSpacerHeight;
    });
  }

  function getElementScrollTop(container: HTMLElement, element: HTMLElement) {
    return (
      element.getBoundingClientRect().top -
      container.getBoundingClientRect().top +
      container.scrollTop
    );
  }

  function getResultSignature() {
    const automaticResultIds = selector.automaticResults
      .map((result) => result.id)
      .join(',');
    const searchResultIds = selector.searchResults
      .map((result) => result.id)
      .join(',');

    return `${automaticResultIds}:${searchResultIds}`;
  }

  let canReset = $derived(
    selector.hasRememberedQuery ||
      manualOverrideId !== null ||
      isIgnored ||
      isAutomaticWinnerSuppressed
  );
  let showsAmbiguousTopCandidates = $derived(
    automaticMatchResult !== null &&
      automaticMatchResult.winner === null &&
      automaticMatchResult.top.length > 0
  );
</script>

<div class="match-selector">
  <div class="search-box">
    <input
      type="text"
      placeholder={selectedEntry.title || 'Wyszukaj tytuł'}
      class="input search-input"
      value={selector.query}
      onkeydown={handleQueryKeydown}
      oninput={handleQueryInput}
    />
    <button
      type="button"
      class="btn btn-primary btn-soft btn-square btn-sm clear-manual-override-button border-0"
      aria-label="Resetuj wpis"
      title="Resetuj wpis"
      disabled={!canReset}
      onclick={() => handleReset()}
    >
      <span aria-hidden="true" class="icon-[lucide--rotate-ccw] size-4"></span>
    </button>
    <button
      type="button"
      class:btn-active={isIgnored}
      class="btn btn-neutral btn-soft btn-square btn-sm ignore-entry-button border-0"
      aria-label={isIgnored ? 'Przestań ignorować wpis' : 'Ignoruj wpis'}
      title={isIgnored ? 'Przestań ignorować wpis' : 'Ignoruj wpis'}
      onclick={selector.applyIgnore}
    >
      <span aria-hidden="true" class="icon-[lucide--eye-off] size-4"></span>
    </button>
  </div>
  <div class="search-content">
    {#if selector.hasResults}
      <ul
        class="match-results"
        aria-label="Wyniki dopasowania"
        bind:this={matchResultsElement}
      >
        {#if selector.automaticResults.length > 0}
          <li class="match-results-section-label">
            <span>Najlepsze kandydaty</span>
          </li>
        {/if}
        {#each selector.automaticResults as result (result.id)}
          <li class="match-result">
            <DatabaseEntryRow
              entry={result.entry}
              scoreLabel={formatMatchScore(result.score)}
              isSelected={result.id === selectedDatabaseEntryId}
              tone={resultTone(result.id)}
              softWarning={selector.conflictingWinnerIds.has(result.id)}
              showIndicator={true}
              indicator={showsAmbiguousTopCandidates ? 'star' : 'bar'}
              rounded={true}
              compact={true}
              onSelect={() => handleResultSelect(result.id)}
              contextMenuItems={contextMenuItemsForResult(result.entry)}
            />
          </li>
        {/each}
        {#if selector.automaticResults.length > 0}
          <li class="match-results-separator" aria-hidden="true"></li>
          <li
            class="match-results-search-anchor"
            aria-hidden="true"
            bind:this={searchResultsAnchorElement}
          ></li>
        {/if}
        {#if selector.automaticResults.length > 0 || selector.searchResults.length > 0}
          <li class="match-results-section-label">
            <span>Wyniki wyszukiwania</span>
          </li>
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
              onSelect={() => handleResultSelect(result.id)}
              contextMenuItems={contextMenuItemsForResult(result.entry)}
            />
          </li>
        {/each}
        {#if selector.automaticResults.length > 0 && selector.searchResults.length === 0}
          <li class="search-message text-muted text-sm font-medium">
            Brak wyników
          </li>
        {/if}
        {#if searchAlignmentSpacerHeight > 0}
          <li
            class="match-results-alignment-spacer"
            style={`height: ${searchAlignmentSpacerHeight}px`}
            aria-hidden="true"
          ></li>
        {/if}
      </ul>
    {:else if selector.errorMessage !== null}
      <p class="search-message text-error text-sm font-medium">
        {selector.errorMessage}
      </p>
    {:else}
      <p class="search-message text-muted text-sm font-medium">Brak wyników</p>
    {/if}
  </div>
</div>

<WorkspaceDialog
  open={pendingAlreadyUsedDatabaseId !== null}
  titleId="already-used-entry-warning-title"
  title="Wpis jest już używany"
  tone="warning"
  confirmLabel="Wybierz mimo to"
  cancelLabel="Zostaw bez zmian"
  onCancel={closeAlreadyUsedWarning}
  onConfirm={confirmAlreadyUsedSelection}
>
  {#if pendingAlreadyUsedOwners.length === 1}
    <p>
      Wpis „{pendingAlreadyUsedOwners[0]?.title ?? ''}” utraci to dopasowanie.
    </p>
  {:else}
    <p>Te wpisy utracą to dopasowanie:</p>
    <ul class="already-used-owner-list">
      {#each pendingAlreadyUsedOwners as owner (owner.id)}
        <li>{owner.title}</li>
      {/each}
    </ul>
  {/if}
</WorkspaceDialog>

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
    padding: calc(var(--spacing) * 2.5);
  }

  .match-results {
    --match-results-list-padding: calc(var(--spacing) * 1);

    display: flex;
    box-sizing: border-box;
    height: 100%;
    min-width: 0;
    flex-direction: column;
    margin: 0;
    padding: var(--match-results-list-padding);
    overflow-y: auto;
    list-style: none;
    scrollbar-color: var(--color-primary) var(--match-selector-panel-bg);
  }

  .match-result {
    min-width: 0;
  }

  .match-results-section-label {
    display: flex;
    min-width: 0;
    flex: 0 0 auto;
    align-items: center;
    gap: calc(var(--spacing) * 2);
    padding: calc(var(--spacing) * 1.5) calc(var(--spacing) * 3)
      calc(var(--spacing) * 1);
    color: color-mix(in oklab, var(--color-base-content) 68%, transparent);
    font-size: 0.6875rem;
    font-weight: 800;
    letter-spacing: 0;
    line-height: 1;
    text-transform: uppercase;
  }

  .match-results-section-label::after {
    min-width: calc(var(--spacing) * 8);
    height: 1px;
    flex: 1 1 auto;
    background-color: color-mix(
      in oklab,
      var(--color-base-content) 18%,
      transparent
    );
    content: '';
  }

  .match-results-search-anchor,
  .match-results-alignment-spacer {
    min-width: 0;
    flex: 0 0 auto;
    pointer-events: none;
  }

  .match-results-search-anchor {
    height: var(--match-results-list-padding);
  }

  .match-results-separator {
    min-width: 0;
    margin: calc(var(--spacing) * 2) calc(var(--spacing) * 3);
    margin-bottom: 0;
    --match-results-separator-color: color-mix(
      in oklab,
      var(--color-base-content) 38%,
      transparent
    );
    border-top: 1px solid var(--match-results-separator-color);
    border-bottom: 1px solid var(--match-results-separator-color);
    border-radius: 999px;
  }

  .already-used-owner-list {
    margin: calc(var(--spacing) * 2) 0 0;
    padding-inline-start: calc(var(--spacing) * 4);
  }
</style>
