<script lang="ts">
  import { animeListTabs, type AnimeListTabId } from './tabs';

  let {
    activeTab,
    selectedEntryTabIds = new Set<AnimeListTabId>(),
    manualOverrideCount,
    onSelectTab,
    onClearManualOverrides
  }: {
    activeTab: AnimeListTabId;
    selectedEntryTabIds?: ReadonlySet<AnimeListTabId>;
    manualOverrideCount: number;
    onSelectTab: (tabId: AnimeListTabId) => void;
    onClearManualOverrides: () => void;
  } = $props();
</script>

<div class="anime-list-tabs">
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
        {tab.label}
      </button>
    {/each}
  </div>
  <button
    type="button"
    class="btn btn-error btn-soft btn-xs reset-manual-overrides-button"
    disabled={manualOverrideCount === 0}
    aria-label="Wyczyść wszystkie ręczne dopasowania"
    title="Wyczyść wszystkie ręczne dopasowania"
    onclick={onClearManualOverrides}
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

  .tabs {
    display: flex;
    min-width: 0;
    flex: 0 1 auto;
    overflow-x: clip;
    overflow-y: hidden;
  }

  .tab {
    min-width: 0;
    overflow: hidden;
    flex: 0 1 auto;
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
</style>
