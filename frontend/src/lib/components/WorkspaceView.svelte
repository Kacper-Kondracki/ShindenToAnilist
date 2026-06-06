<script lang="ts">
  import type { EntryStore } from "../data/entryStore.svelte";
  import type { MatchListResult, ShindenListViews } from "../domain/anime";
  import AnimeListPane from "./workspace/AnimeListPane.svelte";
  import WorkspaceEditorPane from "./workspace/WorkspaceEditorPane.svelte";
  import WorkspaceStatusBar from "./workspace/WorkspaceStatusBar.svelte";

  let {
    providerLabel,
    entryIdsByView,
    entryStore,
    matchResult,
    matchErrorMessage,
    isMatching,
    fetchDurationMs,
    matchDurationMs,
    manualSelections = $bindable(),
  }: {
    providerLabel: string;
    entryIdsByView: ShindenListViews;
    entryStore: EntryStore;
    matchResult: MatchListResult | null;
    matchErrorMessage: string | null;
    isMatching: boolean;
    fetchDurationMs: number | null;
    matchDurationMs: number | null;
    manualSelections: Record<number, number>;
  } = $props();

  let selectedEntryId = $state<number | null>(null);
  let hasTrackedEntries = $state(false);
  let previousEntryIdsByView = $state<ShindenListViews | null>(null);
  let selectedMatchEntry = $derived(
    selectedEntryId === null
      ? null
      : (matchResult?.entries.find(
          (entry) => entry.shindenId === selectedEntryId,
        ) ?? null),
  );
  let selectedWinnerId = $derived(
    selectedMatchEntry?.result.winner?.id ?? null,
  );
  let selectedWinner = $derived(entryStore.getDatabaseEntry(selectedWinnerId));

  $effect(() => {
    if (hasTrackedEntries && previousEntryIdsByView === entryIdsByView) {
      return;
    }

    const shouldClearSelection = hasTrackedEntries;
    hasTrackedEntries = true;
    previousEntryIdsByView = entryIdsByView;

    if (shouldClearSelection) {
      selectedEntryId = null;
    }
  });

  $effect(() => {
    if (
      selectedEntryId !== null &&
      !entryIdsByView.all.some((entryId) => entryId === selectedEntryId)
    ) {
      selectedEntryId = null;
    }
  });

  $effect(() => {
    const winnerId = selectedWinnerId;
    return entryStore.pinDatabaseEntry(winnerId);
  });
</script>

<section class="workspace-content">
  <div class="workspace-layout">
    <AnimeListPane
      {providerLabel}
      {entryIdsByView}
      {entryStore}
      {matchResult}
      {selectedEntryId}
      onSelectEntry={(entryId) => {
        selectedEntryId = entryId;
      }}
    />
    <WorkspaceEditorPane {selectedEntryId} {selectedWinner} />
  </div>
</section>

<WorkspaceStatusBar
  entryIds={entryIdsByView.all}
  {matchResult}
  {matchErrorMessage}
  {isMatching}
  {fetchDurationMs}
  {matchDurationMs}
  bind:manualSelections
/>

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
    gap: 2px;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    background-color: color-mix(
      in oklab,
      var(--color-base-content) 10%,
      transparent
    );
  }

  @media (width <= 48rem) {
    .workspace-layout {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
