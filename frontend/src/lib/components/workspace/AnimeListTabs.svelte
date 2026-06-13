<script lang="ts">
  import { animeListTabs, type AnimeListTabId } from './tabs';

  let {
    activeTab,
    selectedEntryTabIds = new Set<AnimeListTabId>(),
    onSelectTab
  }: {
    activeTab: AnimeListTabId;
    selectedEntryTabIds?: ReadonlySet<AnimeListTabId>;
    onSelectTab: (tabId: AnimeListTabId) => void;
  } = $props();
</script>

<div role="tablist" class="tabs-lift tabs" aria-label="Filtr listy anime">
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

<style>
  .tab--selected-entry {
    box-shadow: inset 0 -2px 0
      color-mix(in oklab, var(--color-primary) 82%, white);
  }
</style>
