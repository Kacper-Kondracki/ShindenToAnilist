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
</script>

<section class="workspace-pane" aria-label={`Lista anime z ${providerLabel}`}>
  <div class="workspace-pane__header">
    <AnimeListTabs bind:activeTab={activeAnimeListTab} />
  </div>
  <div id="anime-list-tab-panel" role="tabpanel" class="workspace-pane__body">
    {#if entries.length > 0}
      <VList data={entries} class="size-full" getKey={(entry) => entry.id}>
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
        Lista jest pusta
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
