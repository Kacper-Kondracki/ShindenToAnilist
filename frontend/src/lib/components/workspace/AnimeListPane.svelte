<script lang="ts">
  import { tick } from "svelte";
  import { VList } from "virtua/svelte";
  import type { VListHandle } from "virtua/svelte";
  import type { EntryStore } from "../../data/entryStore.svelte";
  import type { MatchListResult, ShindenListViews } from "../../domain/anime";
  import AnimeListTabs from "./AnimeListTabs.svelte";
  import AnimeRow, { type AnimeMatchStatus } from "./AnimeRow.svelte";
  import type { AnimeListTabId } from "./tabs";

  type PendingScrollRestore = {
    tabId: AnimeListTabId;
    selectedEntryId: number | null;
    selectedViewportOffset: number | null;
  };

  type SelectedScrollAnchor = {
    entryId: number;
    viewportOffset: number;
  };

  let {
    providerLabel,
    entryIdsByView,
    entryStore,
    matchResult,
    selectedEntryId,
    onSelectEntry,
  }: {
    providerLabel: string;
    entryIdsByView: ShindenListViews;
    entryStore: EntryStore;
    matchResult: MatchListResult | null;
    selectedEntryId: number | null;
    onSelectEntry: (entryId: number) => void;
  } = $props();

  let listRef = $state<VListHandle | null>(null);
  let activeAnimeListTab = $state<AnimeListTabId>("manual");
  let tabScrollOffsets = $state<Record<AnimeListTabId, number>>(
    initialTabScrollOffsets(),
  );
  let selectedScrollAnchors = $state<
    Record<AnimeListTabId, SelectedScrollAnchor | null>
  >(initialSelectedScrollAnchors());
  let pendingScrollRestore = $state<PendingScrollRestore | null>(null);

  let matchStatuses = $derived.by(() => {
    const statuses = new Map<number, AnimeMatchStatus>();

    for (const matchEntry of matchResult?.entries ?? []) {
      if (matchEntry.result.winner !== null) {
        statuses.set(matchEntry.shindenId, "matched");
      } else if (matchEntry.result.top.length > 0) {
        statuses.set(matchEntry.shindenId, "review");
      } else {
        statuses.set(matchEntry.shindenId, "unmatched");
      }
    }

    return statuses;
  });

  let visibleEntryIds = $derived.by(() => entryIdsByView[activeAnimeListTab]);

  $effect(() => {
    const restore = pendingScrollRestore;

    if (restore === null || restore.tabId !== activeAnimeListTab) {
      return;
    }

    visibleEntryIds;
    void restoreScrollPosition(restore);
  });

  function initialTabScrollOffsets(): Record<AnimeListTabId, number> {
    return {
      manual: 0,
      automatic: 0,
      all: 0,
    };
  }

  function initialSelectedScrollAnchors(): Record<
    AnimeListTabId,
    SelectedScrollAnchor | null
  > {
    return {
      manual: null,
      automatic: null,
      all: null,
    };
  }

  let emptyListText = $derived.by(() => {
    if (entryIdsByView.all.length === 0) {
      return "Lista jest pusta";
    }

    if (activeAnimeListTab === "automatic") {
      return "Brak automatycznie dopasowanych wpisów";
    }

    if (activeAnimeListTab === "manual") {
      return "Brak wpisów wymagających ręcznej interwencji";
    }

    return "Brak wpisów do wyświetlenia";
  });

  function handleScroll() {
    rememberActiveTabScrollState();
  }

  function handleSelectTab(nextTabId: AnimeListTabId) {
    if (nextTabId === activeAnimeListTab) {
      return;
    }

    const currentOffset = rememberActiveTabScrollState();
    const currentSelectedIndex =
      selectedEntryId === null
        ? -1
        : visibleEntryIds.findIndex((entryId) => entryId === selectedEntryId);
    const selectedViewportOffset = getSelectedRestoreOffset(
      nextTabId,
      currentSelectedIndex,
      currentOffset,
    );

    pendingScrollRestore = {
      tabId: nextTabId,
      selectedEntryId,
      selectedViewportOffset,
    };
    activeAnimeListTab = nextTabId;
  }

  async function restoreScrollPosition(restore: PendingScrollRestore) {
    pendingScrollRestore = null;
    await tick();

    if (listRef === null || restore.tabId !== activeAnimeListTab) {
      return;
    }

    const selectedIndex =
      restore.selectedEntryId === null
        ? -1
        : visibleEntryIds.findIndex(
            (entryId) => entryId === restore.selectedEntryId,
          );

    if (selectedIndex >= 0 && restore.selectedViewportOffset !== null) {
      listRef.scrollTo(
        Math.max(
          0,
          listRef.getItemOffset(selectedIndex) - restore.selectedViewportOffset,
        ),
      );
      return;
    }

    listRef.scrollTo(tabScrollOffsets[activeAnimeListTab] ?? 0);
  }

  function rememberActiveTabScrollState() {
    if (listRef === null) {
      return tabScrollOffsets[activeAnimeListTab] ?? 0;
    }

    const scrollOffset = listRef.getScrollOffset();
    const selectedIndex =
      selectedEntryId === null
        ? -1
        : visibleEntryIds.findIndex((entryId) => entryId === selectedEntryId);
    const selectedAnchor =
      selectedEntryId !== null && selectedIndex >= 0
        ? {
            entryId: selectedEntryId,
            viewportOffset:
              getSelectedViewportOffset(selectedIndex, scrollOffset) ?? 0,
          }
        : null;

    tabScrollOffsets = {
      ...tabScrollOffsets,
      [activeAnimeListTab]: scrollOffset,
    };
    selectedScrollAnchors = {
      ...selectedScrollAnchors,
      [activeAnimeListTab]: selectedAnchor,
    };

    return scrollOffset;
  }

  function getSelectedRestoreOffset(
    nextTabId: AnimeListTabId,
    currentSelectedIndex: number,
    currentOffset: number,
  ) {
    if (selectedEntryId === null) {
      return null;
    }

    if (listRef !== null && currentSelectedIndex >= 0) {
      return getSelectedViewportOffset(currentSelectedIndex, currentOffset);
    }

    const nextTabAnchor = selectedScrollAnchors[nextTabId];

    return nextTabAnchor?.entryId === selectedEntryId
      ? nextTabAnchor.viewportOffset
      : null;
  }

  function getSelectedViewportOffset(
    selectedIndex: number,
    scrollOffset: number,
  ) {
    if (listRef === null) {
      return null;
    }

    const itemOffset = listRef.getItemOffset(selectedIndex);
    const itemEndOffset = itemOffset + listRef.getItemSize(selectedIndex);
    const viewportEndOffset = scrollOffset + listRef.getViewportSize();
    const isOutsideViewport =
      itemEndOffset <= scrollOffset || itemOffset >= viewportEndOffset;

    return isOutsideViewport ? 0 : itemOffset - scrollOffset;
  }
</script>

<section class="workspace-pane" aria-label={`Lista anime z ${providerLabel}`}>
  <div class="workspace-pane__header">
    <AnimeListTabs
      activeTab={activeAnimeListTab}
      onSelectTab={handleSelectTab}
    />
  </div>
  <div id="anime-list-tab-panel" role="tabpanel" class="workspace-pane__body">
    {#if visibleEntryIds.length > 0}
      <VList
        bind:this={listRef}
        data={visibleEntryIds}
        class="anime-list size-full"
        getKey={(entryId) => entryId}
        onscroll={handleScroll}
      >
        {#snippet children(entryId)}
          <AnimeRow
            {entryId}
            entry={entryStore.getShindenEntry(entryId)}
            matchStatus={matchStatuses.get(entryId) ?? "unmatched"}
            isSelected={entryId === selectedEntryId}
            onSelect={() => onSelectEntry(entryId)}
            {entryStore}
          />
        {/snippet}
      </VList>
    {:else}
      <p class="workspace-empty text-sm font-medium text-muted">
        {emptyListText}
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

  :global(.anime-list > *) {
    pointer-events: auto !important;
  }
</style>
