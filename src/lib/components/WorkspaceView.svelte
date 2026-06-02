<script lang="ts">
  import { VList } from 'virtua/svelte';

  let { providerLabel }: { providerLabel: string } = $props();

  const animeListTabs = [
    {
      id: 'manual',
      label: 'Ręczna interwencja',
      placeholder: 'Pozycje wymagające ręcznego dopasowania będą tutaj.'
    },
    {
      id: 'automatic',
      label: 'Automatyczne',
      placeholder: 'Automatycznie dopasowane pozycje będą tutaj.'
    },
    {
      id: 'all',
      label: 'Wszystko',
      placeholder: 'Pełna lista z importu będzie tutaj.'
    }
  ] as const;

  type AnimeListTabId = (typeof animeListTabs)[number]['id'];

  let activeAnimeListTab = $state<AnimeListTabId>('manual');
  let activeAnimeListTabDetails = $derived(
    animeListTabs.find(({ id }) => id === activeAnimeListTab) ?? animeListTabs[0]
  );
</script>

<section class="workspace-content">
  <div class="workspace-layout">
    <section class="workspace-pane" aria-label={`Lista anime z ${providerLabel}`}>
      <div class="workspace-pane__header">
        <div role="tablist" class="tabs-lift tabs" aria-label="Filtr listy anime">
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
      <div id="anime-list-tab-panel" role="tabpanel" class="workspace-pane__body">
        <!-- <p class="text-muted">{activeAnimeListTabDetails.placeholder}</p> -->
        <VList
          data={Array.from({ length: 200 }, (v, i) => i)}
          class="size-full"
          getKey={(_, i) => i}
        >
          {#snippet children(item, index)}
            <div class="p-4">
              <h1>To jest test</h1>
            </div>
          {/snippet}
        </VList>
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
    border-left: var(--border) solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
    border-right: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 10%, transparent);
  }

  .workspace-pane__header {
    border-bottom: calc(var(--border) * 2) solid
      color-mix(in oklab, var(--color-base-content) 10%, transparent);
    padding-top: calc(var(--spacing) * 1);
  }

  .workspace-pane__body {
    display: grid;
    flex: 1;
    min-height: 10rem;
    place-items: center;
    padding: calc(var(--spacing) * 4);
  }

  .app-status-bar {
    border-top: var(--border) solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
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
