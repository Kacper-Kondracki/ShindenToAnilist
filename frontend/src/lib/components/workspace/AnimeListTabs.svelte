<script lang="ts">
  import { animeListTabs, type AnimeListTabId } from "./tabs";

  let {
    activeTab = $bindable(),
  }: {
    activeTab: AnimeListTabId;
  } = $props();
</script>

<div role="tablist" class="tabs-lift tabs" aria-label="Filtr listy anime">
  {#each animeListTabs as tab}
    <button
      type="button"
      role="tab"
      class:tab-active={activeTab === tab.id}
      class="tab"
      aria-selected={activeTab === tab.id}
      aria-controls="anime-list-tab-panel"
      onclick={() => (activeTab = tab.id)}
    >
      {tab.label}
    </button>
  {/each}
</div>
