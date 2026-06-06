<script lang="ts">
  import { VList } from "virtua/svelte";
  import type { ShindenEntry } from "../../domain/anime";
  import AnimeListTabs from "./AnimeListTabs.svelte";
  import AnimeRow from "./AnimeRow.svelte";
  import type { AnimeListTabId } from "./tabs";

  let {
    providerLabel,
    entries,
  }: {
    providerLabel: string;
    entries: ShindenEntry[];
  } = $props();

  let activeAnimeListTab = $state<AnimeListTabId>("manual");
</script>

<section class="workspace-pane" aria-label={`Lista anime z ${providerLabel}`}>
  <div class="workspace-pane__header">
    <AnimeListTabs bind:activeTab={activeAnimeListTab} />
  </div>
  <div id="anime-list-tab-panel" role="tabpanel" class="workspace-pane__body">
    {#if entries.length > 0}
      <VList data={entries} class="size-full" getKey={(entry) => entry.id}>
        {#snippet children(entry)}
          <AnimeRow {entry} />
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
    border-left: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 10%, transparent);
    border-right: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 10%, transparent);
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
