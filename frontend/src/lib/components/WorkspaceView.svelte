<script lang="ts">
  import { VList } from "virtua/svelte";

  type ShindenEntry = {
    id: number;
    title: string;
    animeStatus: string;
    animeType: string;
    episodes: number | null;
    watchStatus: string;
    watchedEpisodes: number;
    score: number | null;
  };

  let {
    providerLabel,
    entries,
  }: {
    providerLabel: string;
    entries: ShindenEntry[];
  } = $props();

  const animeListTabs = [
    {
      id: "manual",
      label: "Ręczna interwencja",
    },
    {
      id: "automatic",
      label: "Automatyczne",
    },
    {
      id: "all",
      label: "Wszystko",
    },
  ] as const;

  type AnimeListTabId = (typeof animeListTabs)[number]["id"];

  let activeAnimeListTab = $state<AnimeListTabId>("manual");
</script>

<section class="workspace-content">
  <div class="workspace-layout">
    <section
      class="workspace-pane"
      aria-label={`Lista anime z ${providerLabel}`}
    >
      <div class="workspace-pane__header">
        <div
          role="tablist"
          class="tabs-lift tabs"
          aria-label="Filtr listy anime"
        >
          {#each animeListTabs as tab}
            <button
              type="button"
              role="tab"
              class:tab-active={activeAnimeListTab === tab.id}
              class="tab"
              aria-selected={activeAnimeListTab === tab.id}
              aria-controls="anime-list-tab-panel"
              onclick={() => (activeAnimeListTab = tab.id)}
            >
              {tab.label}
            </button>
          {/each}
        </div>
      </div>
      <div
        id="anime-list-tab-panel"
        role="tabpanel"
        class="workspace-pane__body"
      >
        {#if entries.length > 0}
          <VList data={entries} class="size-full" getKey={(entry) => entry.id}>
            {#snippet children(entry)}
              <article class="anime-row">
                <div class="min-w-0">
                  <h2 class="truncate text-sm font-semibold">{entry.title}</h2>
                  <p class="truncate text-xs text-muted">
                    {entry.animeType} · {entry.watchStatus} · {entry.watchedEpisodes}/{entry.episodes ?? "?"}
                  </p>
                </div>

                {#if entry.score !== null}
                  <span class="badge badge-soft badge-info shrink-0">
                    {entry.score}/10
                  </span>
                {/if}
              </article>
            {/snippet}
          </VList>
        {:else}
          <p class="workspace-empty text-sm font-medium text-muted">
            Lista jest pusta
          </p>
        {/if}
      </div>
    </section>

    <section class="workspace-pane" aria-label="Editor">
      <div class="workspace-pane__body">
        <p class="text-muted">Widok docelowy będzie tutaj.</p>
      </div>
    </section>
  </div>
</section>

<footer class="app-status-bar">
  <div class="app-status-bar__body">
    <span class="text-sm font-medium text-muted">Gotowe do eksportu</span>
    <button class="btn btn-error md:btn-wide" type="button">Eksport</button>
  </div>
</footer>

<style>
  .workspace-content {
    display: grid;
    flex: 1;
    min-height: 0;
    align-items: stretch;
  }

  .workspace-layout {
    display: grid;
    min-height: 0;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  }

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

  .anime-row {
    display: flex;
    min-height: 4.5rem;
    align-items: center;
    justify-content: space-between;
    gap: calc(var(--spacing) * 3);
    border-bottom: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 8%, transparent);
    padding-inline: calc(var(--spacing) * 4);
    padding-block: calc(var(--spacing) * 3);
  }

  .workspace-empty {
    padding: calc(var(--spacing) * 4);
  }

  .app-status-bar {
    border-top: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 10%, transparent);
    background-color: var(--color-base-200);
  }

  .app-status-bar__body {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: calc(var(--spacing) * 4);
    padding-inline: calc(var(--spacing) * 4);
    padding-block: calc(var(--spacing) * 3);
  }

  @media (width <= 48rem) {
    .workspace-layout {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
