<script lang="ts">
  import { VList } from "virtua/svelte";
  import type { MatchListResult, ShindenEntry } from "../../domain/anime";
  import AnimeListTabs from "./AnimeListTabs.svelte";
  import AnimeRow, { type AnimeMatchStatus } from "./AnimeRow.svelte";
  import type { AnimeListTabId } from "./tabs";

  let {
    providerLabel,
    entries,
    matchResult,
    selectedEntryId,
    onSelectEntry,
  }: {
    providerLabel: string;
    entries: ShindenEntry[];
    matchResult: MatchListResult | null;
    selectedEntryId: number | null;
    onSelectEntry: (entryId: number) => void;
  } = $props();

  let activeAnimeListTab = $state<AnimeListTabId>("manual");
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

  let visibleEntries = $derived.by(() =>
    entries
      .map((entry, index) => ({ entry, index }))
      .filter(({ entry }) => {
        if (activeAnimeListTab === "all") {
          return true;
        }

        if (activeAnimeListTab === "automatic") {
          return automaticMatchedEntryIds.has(entry.id);
        }

        return !automaticMatchedEntryIds.has(entry.id);
      })
      .sort((left, right) => {
        const leftStatus = matchStatuses.get(left.entry.id) ?? "unmatched";
        const rightStatus = matchStatuses.get(right.entry.id) ?? "unmatched";

        return (
          matchStatusSortRanks[leftStatus] -
            matchStatusSortRanks[rightStatus] || left.index - right.index
        );
      })
      .map(({ entry }) => entry),
  );

  let emptyListText = $derived.by(() => {
    if (entries.length === 0) {
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
</script>

<section class="workspace-pane" aria-label={`Lista anime z ${providerLabel}`}>
  <div class="workspace-pane__header">
    <AnimeListTabs bind:activeTab={activeAnimeListTab} />
  </div>
  <div id="anime-list-tab-panel" role="tabpanel" class="workspace-pane__body">
    {#if visibleEntries.length > 0}
      <VList
        data={visibleEntries}
        class="size-full"
        getKey={(entry) => entry.id}
      >
        {#snippet children(entry)}
          <AnimeRow
            {entry}
            matchStatus={matchStatuses.get(entry.id) ?? "unmatched"}
            isSelected={entry.id === selectedEntryId}
            onSelect={() => onSelectEntry(entry.id)}
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
</style>
