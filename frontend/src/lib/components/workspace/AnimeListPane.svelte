<script lang="ts">
  import { VList } from 'virtua/svelte';
  import type { LoadedAnimeData } from '../../data/loadedAnimeData.svelte';
  import type {
    MatchListResult,
    ShindenEntry,
    ShindenListViews
  } from '../../domain/anime';
  import {
    animeListItemSize,
    createAnimeListPaneController
  } from '../../features/workspace/animeListPaneController.svelte';
  import AnimeListTabs from './AnimeListTabs.svelte';
  import AnimeRow from './AnimeRow.svelte';

  let {
    providerLabel,
    entryIdsByView,
    animeData,
    matchResult,
    selectedEntryId,
    onSelectEntry
  }: {
    providerLabel: string;
    entryIdsByView: ShindenListViews;
    animeData: LoadedAnimeData;
    matchResult: MatchListResult | null;
    selectedEntryId: number | null;
    onSelectEntry: (entryId: number) => void | Promise<void>;
  } = $props();

  const listPane = createAnimeListPaneController({
    getEntryIdsByView: () => entryIdsByView,
    getMatchResult: () => matchResult,
    getSelectedEntryId: () => selectedEntryId
  });

  function handleListScroll() {
    listPane.handleScroll();
  }

  function handleSelectEntry(entryId: number) {
    void onSelectEntry(entryId);
  }

  function getLoadedShindenEntry(entryId: number): ShindenEntry {
    const entry = animeData.getShindenEntry(entryId);
    if (entry === null) {
      throw new Error(`Brak wczytanego wpisu Shinden #${entryId}`);
    }

    return entry;
  }
</script>

<section class="workspace-pane" aria-label={`Lista anime z ${providerLabel}`}>
  <div class="workspace-pane__header">
    <AnimeListTabs
      activeTab={listPane.activeTab}
      onSelectTab={listPane.selectTab}
    />
  </div>
  <div id="anime-list-tab-panel" role="tabpanel" class="workspace-pane__body">
    {#if listPane.visibleEntryIds.length > 0}
      <VList
        bind:this={listPane.listRef}
        data={listPane.visibleEntryIds}
        itemSize={animeListItemSize}
        class="anime-list size-full"
        getKey={(entryId) => entryId}
        bufferSize={0}
        onscroll={handleListScroll}
      >
        {#snippet children(entryId)}
          <AnimeRow
            entry={getLoadedShindenEntry(entryId)}
            matchStatus={listPane.matchStatuses.get(entryId) ?? 'unmatched'}
            isSelected={entryId === selectedEntryId}
            onSelect={() => handleSelectEntry(entryId)}
          />
        {/snippet}
      </VList>
    {:else}
      <p class="workspace-empty text-sm font-medium text-muted">
        {listPane.emptyListText}
      </p>
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

  .workspace-pane__header {
    border-bottom: calc(var(--border) * 2) solid
      color-mix(in oklab, var(--color-base-content) 10%, transparent);
    padding-top: calc(var(--spacing) * 1);
  }

  .workspace-pane__body {
    display: block;
    flex: 1;
    min-height: 0;
    overflow: hidden;
    padding: 0;
  }

  .workspace-empty {
    padding: calc(var(--spacing) * 4);
  }

  :global(.anime-list) {
    scrollbar-color: var(--color-primary) var(--color-base-300);
  }

  :global(.anime-list > *) {
    pointer-events: auto !important;
  }
</style>
