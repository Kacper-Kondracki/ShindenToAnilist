<script lang="ts">
  import { tick } from "svelte";
  import { VList } from "virtua/svelte";
  import type { VListHandle } from "virtua/svelte";
  import { getLoadedShindenEntries } from "../../api/appService";
  import type { MatchListResult, ShindenEntry } from "../../domain/anime";
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
    entryIds,
    matchResult,
    selectedEntryId,
    onSelectEntry,
  }: {
    providerLabel: string;
    entryIds: number[];
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
  let hasTrackedWorkspaceData = $state(false);
  let previousEntryIds = $state<number[] | null>(null);
  let previousMatchResult = $state<MatchListResult | null>(null);
  let shindenEntriesById = $state<Record<number, ShindenEntry>>({});
  let pendingDetailIds = new Set<number>();
  let detailBatchScheduled = false;

  const matchStatusSortRanks: Record<AnimeMatchStatus, number> = {
    unmatched: 0,
    review: 1,
    matched: 2,
  };

  let automaticMatchedEntryIds = $derived.by(() => {
    const ids = new Set<number>();

    for (const matchEntry of matchResult?.entries ?? []) {
      if (matchEntry.result.winner !== null) {
        ids.add(matchEntry.shindenId);
      }
    }

    return ids;
  });

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

  let visibleEntryIds = $derived.by(() =>
    getVisibleEntryIds(activeAnimeListTab),
  );

  $effect(() => {
    if (
      hasTrackedWorkspaceData &&
      previousEntryIds === entryIds &&
      previousMatchResult === matchResult
    ) {
      return;
    }

    const shouldReset = hasTrackedWorkspaceData;
    hasTrackedWorkspaceData = true;
    previousEntryIds = entryIds;
    previousMatchResult = matchResult;
    shindenEntriesById = {};
    pendingDetailIds = new Set();

    if (shouldReset) {
      resetListNavigationState();
    }
  });

  $effect(() => {
    const restore = pendingScrollRestore;

    if (restore === null || restore.tabId !== activeAnimeListTab) {
      return;
    }

    visibleEntryIds;
    void restoreScrollPosition(restore);
  });

  function getVisibleEntryIds(tabId: AnimeListTabId): number[] {
    return entryIds
      .map((entryId, index) => ({ entryId, index }))
      .filter(({ entryId }) => {
        if (tabId === "all") {
          return true;
        }

        if (tabId === "automatic") {
          return automaticMatchedEntryIds.has(entryId);
        }

        return !automaticMatchedEntryIds.has(entryId);
      })
      .sort((left, right) => {
        const leftStatus = matchStatuses.get(left.entryId) ?? "unmatched";
        const rightStatus = matchStatuses.get(right.entryId) ?? "unmatched";
        const statusComparison =
          matchStatusSortRanks[leftStatus] - matchStatusSortRanks[rightStatus];

        return statusComparison || left.index - right.index;
      })
      .map(({ entryId }) => entryId);
  }

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

  function resetListNavigationState() {
    activeAnimeListTab = "manual";
    tabScrollOffsets = initialTabScrollOffsets();
    selectedScrollAnchors = initialSelectedScrollAnchors();
    pendingScrollRestore = null;
    listRef?.scrollTo(0);
  }

  let emptyListText = $derived.by(() => {
    if (entryIds.length === 0) {
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

  function handleRowVisible(entryId: number) {
    if (
      shindenEntriesById[entryId] !== undefined ||
      pendingDetailIds.has(entryId)
    ) {
      return;
    }

    pendingDetailIds.add(entryId);

    if (detailBatchScheduled) {
      return;
    }

    detailBatchScheduled = true;
    queueMicrotask(loadPendingRowDetails);
  }

  async function loadPendingRowDetails() {
    detailBatchScheduled = false;

    const entryIdsToLoad = [...pendingDetailIds].filter(
      (entryId) => shindenEntriesById[entryId] === undefined,
    );
    pendingDetailIds = new Set();

    if (entryIdsToLoad.length === 0) {
      return;
    }

    const loadedEntries = await getLoadedShindenEntries(entryIdsToLoad);
    const nextEntriesById = { ...shindenEntriesById };

    for (const entry of loadedEntries) {
      nextEntriesById[entry.id] = entry;
    }

    shindenEntriesById = nextEntriesById;
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
            entry={shindenEntriesById[entryId] ?? null}
            matchStatus={matchStatuses.get(entryId) ?? "unmatched"}
            isSelected={entryId === selectedEntryId}
            onSelect={() => onSelectEntry(entryId)}
            onVisible={handleRowVisible}
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
