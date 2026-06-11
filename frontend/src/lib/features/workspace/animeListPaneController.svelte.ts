import { tick } from 'svelte';
import type { VListHandle } from 'virtua/svelte';

import type { MatchListResult, ShindenListViews } from '../../domain/anime';
import type { AnimeMatchStatus } from '../../components/workspace/AnimeRow.svelte';
import type { AnimeListTabId } from '../../components/workspace/tabs';

type PendingScrollRestore = {
  tabId: AnimeListTabId;
  selectedEntryId: number | null;
  selectedViewportOffset: number | null;
};

type SelectedScrollAnchor = {
  entryId: number;
  viewportOffset: number;
};

type AnimeListPaneControllerInput = {
  getEntryIdsByView: () => ShindenListViews;
  getMatchResult: () => MatchListResult | null;
  getSelectedEntryId: () => number | null;
};

export type AnimeListPaneController = ReturnType<
  typeof createAnimeListPaneController
>;

export function createAnimeListPaneController(
  input: AnimeListPaneControllerInput
) {
  let listRef = $state<VListHandle | null>(null);
  let activeTab = $state<AnimeListTabId>('manual');
  let tabScrollOffsets = $state<Record<AnimeListTabId, number>>(
    initialTabScrollOffsets()
  );
  let selectedScrollAnchors = $state<
    Record<AnimeListTabId, SelectedScrollAnchor | null>
  >(initialSelectedScrollAnchors());
  let pendingScrollRestore = $state<PendingScrollRestore | null>(null);

  let matchStatuses = $derived.by(() => {
    const statuses = new Map<number, AnimeMatchStatus>();

    for (const matchEntry of input.getMatchResult()?.entries ?? []) {
      if (matchEntry.result.winner !== null) {
        statuses.set(matchEntry.shindenId, 'matched');
      } else if (matchEntry.result.top.length > 0) {
        statuses.set(matchEntry.shindenId, 'review');
      } else {
        statuses.set(matchEntry.shindenId, 'unmatched');
      }
    }

    return statuses;
  });
  let visibleEntryIds = $derived.by(() => input.getEntryIdsByView()[activeTab]);
  let emptyListText = $derived.by(() => {
    if (input.getEntryIdsByView().all.length === 0) {
      return 'Lista jest pusta';
    }

    if (activeTab === 'automatic') {
      return 'Brak automatycznie dopasowanych wpisów';
    }

    if (activeTab === 'manual') {
      return 'Brak wpisów wymagających ręcznej interwencji';
    }

    return 'Brak wpisów do wyświetlenia';
  });

  $effect(() => {
    const restore = pendingScrollRestore;

    if (restore === null || restore.tabId !== activeTab) {
      return;
    }

    visibleEntryIds;
    void restoreScrollPosition(restore);
  });

  function setListRef(nextListRef: VListHandle | null) {
    listRef = nextListRef;
  }

  function handleScroll() {
    rememberActiveTabScrollState();
  }

  function selectTab(nextTabId: AnimeListTabId) {
    if (nextTabId === activeTab) {
      return;
    }

    const currentOffset = rememberActiveTabScrollState();
    const selectedEntryId = input.getSelectedEntryId();
    const currentSelectedIndex =
      selectedEntryId === null
        ? -1
        : visibleEntryIds.findIndex((entryId) => entryId === selectedEntryId);
    const selectedViewportOffset = getSelectedRestoreOffset(
      nextTabId,
      currentSelectedIndex,
      currentOffset
    );

    pendingScrollRestore = {
      tabId: nextTabId,
      selectedEntryId,
      selectedViewportOffset
    };
    activeTab = nextTabId;
  }

  async function restoreScrollPosition(restore: PendingScrollRestore) {
    pendingScrollRestore = null;
    await tick();

    if (listRef === null || restore.tabId !== activeTab) {
      return;
    }

    const selectedIndex =
      restore.selectedEntryId === null
        ? -1
        : visibleEntryIds.findIndex(
            (entryId) => entryId === restore.selectedEntryId
          );

    if (selectedIndex >= 0 && restore.selectedViewportOffset !== null) {
      listRef.scrollTo(
        Math.max(
          0,
          listRef.getItemOffset(selectedIndex) - restore.selectedViewportOffset
        )
      );
      return;
    }

    listRef.scrollTo(tabScrollOffsets[activeTab] ?? 0);
  }

  function rememberActiveTabScrollState() {
    if (listRef === null) {
      return tabScrollOffsets[activeTab] ?? 0;
    }

    const selectedEntryId = input.getSelectedEntryId();
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
              getSelectedViewportOffset(selectedIndex, scrollOffset) ?? 0
          }
        : null;

    tabScrollOffsets = {
      ...tabScrollOffsets,
      [activeTab]: scrollOffset
    };
    selectedScrollAnchors = {
      ...selectedScrollAnchors,
      [activeTab]: selectedAnchor
    };

    return scrollOffset;
  }

  function getSelectedRestoreOffset(
    nextTabId: AnimeListTabId,
    currentSelectedIndex: number,
    currentOffset: number
  ) {
    const selectedEntryId = input.getSelectedEntryId();

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
    scrollOffset: number
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

  return {
    get listRef() {
      return listRef;
    },
    set listRef(nextListRef: VListHandle | null) {
      setListRef(nextListRef);
    },
    get activeTab() {
      return activeTab;
    },
    get visibleEntryIds() {
      return visibleEntryIds;
    },
    get matchStatuses() {
      return matchStatuses;
    },
    get emptyListText() {
      return emptyListText;
    },
    handleScroll,
    selectTab
  };
}

function initialTabScrollOffsets(): Record<AnimeListTabId, number> {
  return {
    manual: 0,
    automatic: 0,
    all: 0
  };
}

function initialSelectedScrollAnchors(): Record<
  AnimeListTabId,
  SelectedScrollAnchor | null
> {
  return {
    manual: null,
    automatic: null,
    all: null
  };
}
