<script lang="ts">
  import type { LoadedAnimeData } from '../../data/loadedAnimeData.svelte';
  import type {
    DatabaseEntry,
    ShindenMatchResult,
    WireNumber,
    WireNumberRecord
  } from '../../domain/anime';
  import { wireNumberKey } from '../../domain/anime';
  import type { MatchSelectorInitialSearch } from '../../features/workspace/matchSelectorController.svelte';
  import type { SelectedWinnerState } from '../../features/workspace/workspaceController.svelte';
  import {
    AnimeStatus,
    AnimeType,
    Season
  } from '../../gen/shinden_to_anilist/v1/common_pb';
  import DatabaseEntryPreview from './DatabaseEntryPreview.svelte';
  import MatchSelector from './MatchSelector.svelte';

  const placeholderDatabaseEntry = {
    id: 0n,
    sources: [],
    title: 'Placeholder database entry',
    animeType: AnimeType.UNSPECIFIED,
    episodes: 0,
    status: AnimeStatus.UNSPECIFIED,
    season: Season.UNSPECIFIED,
    year: null,
    picture: '',
    thumbnail: '',
    duration: null,
    synonyms: []
  } satisfies DatabaseEntry;

  let {
    animeData,
    selectedEntryId,
    selectedMatchEntry,
    onSetManualOverride,
    onSetIgnored,
    onClearManualOverride,
    onSetMatchSelectorQuery,
    onResetMatchSelectorQuery,
    manualOverrides,
    ignoredEntryIds,
    displacedAutomaticEntryIds,
    matchSelectorQueries,
    winnerClaimsByDatabaseId,
    initialMatchSearch,
    selectedWinnerState,
    onGoToEntry
  }: {
    animeData: LoadedAnimeData;
    selectedEntryId: WireNumber | null;
    selectedMatchEntry: ShindenMatchResult | null;
    selectedWinnerState: SelectedWinnerState;
    manualOverrides: WireNumberRecord<WireNumber>;
    ignoredEntryIds: WireNumberRecord<true>;
    displacedAutomaticEntryIds: WireNumberRecord<true>;
    matchSelectorQueries: WireNumberRecord<string>;
    winnerClaimsByDatabaseId: ReadonlyMap<WireNumber, readonly WireNumber[]>;
    initialMatchSearch: MatchSelectorInitialSearch | null;
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

  const compactPreviewPaneHeight = 42 * 16;
  const compactPreviewPaneHeightHysteresis = 16;
  const compactPreviewViewportWidth = 58 * 16;

  let selectedShindenEntry = $derived(
    selectedEntryId === null ? null : animeData.getShindenEntry(selectedEntryId)
  );
  let selectedDatabaseEntryId = $derived.by(() => {
    if (selectedWinnerState.status === 'ready') {
      return selectedWinnerState.entry.id;
    }

    if (selectedWinnerState.status === 'missing') {
      return selectedWinnerState.databaseEntryId;
    }

    return null;
  });
  let manualOverrideId = $derived(
    selectedEntryId === null
      ? null
      : (manualOverrides[wireNumberKey(selectedEntryId)] ?? null)
  );
  let isIgnored = $derived(
    selectedEntryId !== null &&
      ignoredEntryIds[wireNumberKey(selectedEntryId)] === true
  );
  let isAutomaticWinnerSuppressed = $derived(
    selectedEntryId !== null &&
      displacedAutomaticEntryIds[wireNumberKey(selectedEntryId)] === true
  );
  let matchSelectorQuery = $derived(
    selectedEntryId === null
      ? ''
      : (matchSelectorQueries[wireNumberKey(selectedEntryId)] ?? '')
  );
  let automaticMatchResult = $derived(selectedMatchEntry?.result ?? null);
  let previewEntry = $derived(
    selectedWinnerState.status === 'ready'
      ? selectedWinnerState.entry
      : placeholderDatabaseEntry
  );
  let isPreviewPlaceholder = $derived(selectedWinnerState.status !== 'ready');
  let hasCompactPreviewPaneHeight = $state(false);
  let viewportWidth = $state(0);
  let isPreviewCompact = $derived(
    hasCompactPreviewPaneHeight ||
      (viewportWidth > 0 && viewportWidth <= compactPreviewViewportWidth)
  );

  function trackCompactPreviewPaneHeight(node: HTMLElement) {
    function update(height: number) {
      if (hasCompactPreviewPaneHeight) {
        if (
          height >
          compactPreviewPaneHeight + compactPreviewPaneHeightHysteresis
        ) {
          hasCompactPreviewPaneHeight = false;
        }

        return;
      }

      if (height < compactPreviewPaneHeight) {
        hasCompactPreviewPaneHeight = true;
      }
    }

    const resizeObserver = new ResizeObserver((entries) => {
      update(
        entries[0]?.contentRect.height ?? node.getBoundingClientRect().height
      );
    });

    resizeObserver.observe(node);
    update(node.getBoundingClientRect().height);

    return {
      destroy() {
        resizeObserver.disconnect();
      }
    };
  }
</script>

<svelte:window bind:innerWidth={viewportWidth} />

<section
  class="workspace-pane"
  aria-label="Editor"
  use:trackCompactPreviewPaneHeight
>
  <div class="workspace-pane__body">
    {#if selectedWinnerState.status === 'no-selection'}
      <p class="workspace-empty text-muted text-sm font-medium">
        Wybierz wpis z listy
      </p>
    {:else if selectedShindenEntry === null}
      <p class="workspace-empty text-muted text-sm font-medium">
        Nie znaleziono wpisu źródłowego
      </p>
    {:else}
      <div class="editor-layout">
        <div class="editor-layout__selector">
          {#key selectedShindenEntry.id}
            <MatchSelector
              selectedEntry={selectedShindenEntry}
              {selectedDatabaseEntryId}
              {manualOverrideId}
              {isIgnored}
              {isAutomaticWinnerSuppressed}
              rememberedQuery={matchSelectorQuery}
              {automaticMatchResult}
              initialSearch={initialMatchSearch}
              {winnerClaimsByDatabaseId}
              getDatabaseEntry={animeData.getDatabaseEntry}
              getShindenEntry={animeData.getShindenEntry}
              {onSetManualOverride}
              {onSetIgnored}
              {onClearManualOverride}
              {onSetMatchSelectorQuery}
              {onResetMatchSelectorQuery}
              {onGoToEntry}
            />
          {/key}
        </div>
        <div class="editor-layout__preview" aria-hidden={isPreviewPlaceholder}>
          <DatabaseEntryPreview
            entry={previewEntry}
            placeholder={isPreviewPlaceholder}
            compact={isPreviewCompact}
          />
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
    overflow: hidden;
    padding: 0;
  }

  .editor-layout {
    display: grid;
    box-sizing: border-box;
    width: 100%;
    max-width: 100%;
    height: 100%;
    min-width: 0;
    min-height: 0;
    gap: calc(var(--spacing) * 3);
    overflow: hidden;
    padding: calc(var(--spacing) * 3);
    grid-template-rows:
      minmax(0, 1fr)
      auto;
  }

  .editor-layout__selector {
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .editor-layout__preview {
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .workspace-empty {
    padding: calc(var(--spacing) * 4);
  }

  @media (width <= 48rem) {
    .editor-layout {
      grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
      grid-template-rows: minmax(0, 1fr);
    }
  }
</style>
