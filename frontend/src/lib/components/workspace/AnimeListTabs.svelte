<script lang="ts">
  import { animeListTabs, type AnimeListTabId } from './tabs';

  let {
    activeTab,
    selectedEntryTabIds = new Set<AnimeListTabId>(),
    listOverrideCount,
    onSelectTab,
    onClearListOverrides,
    onOpenGuide
  }: {
    activeTab: AnimeListTabId;
    selectedEntryTabIds?: ReadonlySet<AnimeListTabId>;
    listOverrideCount: number;
    onSelectTab: (tabId: AnimeListTabId) => void;
    onClearListOverrides: () => void;
    onOpenGuide: () => void;
  } = $props();
</script>

<div class="anime-list-tabs">
  <div class="anime-list-filters">
    <div role="tablist" class="tabs tabs-border" aria-label="Filtr listy anime">
      {#each animeListTabs as tab}
        <button
          type="button"
          role="tab"
          class:tab-active={activeTab === tab.id}
          class:tab--selected-entry={selectedEntryTabIds.has(tab.id)}
          class="tab"
          aria-selected={activeTab === tab.id}
          aria-controls="anime-list-tab-panel"
          aria-label={selectedEntryTabIds.has(tab.id)
            ? `${tab.label}, zawiera zaznaczony wpis`
            : tab.label}
          onclick={() => onSelectTab(tab.id)}
        >
          <span class="tab-label">{tab.label}</span>
        </button>
      {/each}
    </div>
    <button
      type="button"
      class="btn btn-primary btn-soft btn-sm shrink-0"
      aria-haspopup="dialog"
      onclick={onOpenGuide}
    >
      <span class="icon-[lucide--book-open] size-4" aria-hidden="true"></span>
      Poradnik
    </button>
  </div>
  <button
    type="button"
    class="btn btn-error btn-soft btn-xs reset-manual-overrides-button"
    disabled={listOverrideCount === 0}
    aria-label="Wyczyść ręczne dopasowania i ignorowane wpisy"
    title="Wyczyść ręczne dopasowania i ignorowane wpisy"
    onclick={onClearListOverrides}
  >
    <span aria-hidden="true" class="icon-[lucide--rotate-ccw] size-3.5"></span>
    <span>Wyczyść listę</span>
  </button>
</div>

<style>
  .anime-list-tabs {
    display: flex;
    width: 100%;
    min-width: 0;
    align-items: center;
    gap: calc(var(--spacing) * 2);
    padding-inline: calc(var(--spacing) * 2);
  }

  .anime-list-filters {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: calc(var(--spacing) * 2);
  }

  .tabs {
    display: flex;
    min-width: 0;
    flex: 0 1 auto;
    flex-wrap: nowrap;
    overflow-x: clip;
    overflow-y: hidden;
  }

  .tab:last-child {
    flex-shrink: 0;
  }

  .tab {
    min-width: 0;
    overflow: hidden;
    flex: 0 1 auto;
  }

  .tab-label {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tab--selected-entry {
    --tab-border-color: color-mix(
      in oklab,
      var(--color-primary) 60%,
      transparent
    );
  }

  .reset-manual-overrides-button {
    flex: 0 0 auto;
    margin-left: auto;
    white-space: nowrap;
  }

  @media (width < 38rem) {
    .anime-list-tabs {
      flex-wrap: wrap;
      row-gap: var(--spacing);
      padding-block-end: var(--spacing);
    }

    .anime-list-filters {
      flex: 1 0 100%;
    }

    .tab {
      padding-inline: calc(var(--spacing) * 1.5);
      font-size: 0.75rem;
    }
  }
</style>
