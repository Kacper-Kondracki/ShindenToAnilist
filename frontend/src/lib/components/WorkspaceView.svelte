<script lang="ts">
  import type {
    DatabaseEntry,
    MatchListResult,
    ShindenEntry,
  } from "../domain/anime";
  import AnimeListPane from "./workspace/AnimeListPane.svelte";
  import WorkspaceEditorPane from "./workspace/WorkspaceEditorPane.svelte";
  import WorkspaceStatusBar from "./workspace/WorkspaceStatusBar.svelte";

  let {
    providerLabel,
    entries,
    databaseEntries,
    matchResult,
    matchErrorMessage,
    isMatching,
    fetchDurationMs,
    matchDurationMs,
    manualSelections = $bindable(),
  }: {
    providerLabel: string;
    entries: ShindenEntry[];
    databaseEntries: DatabaseEntry[];
    matchResult: MatchListResult | null;
    matchErrorMessage: string | null;
    isMatching: boolean;
    fetchDurationMs: number | null;
    matchDurationMs: number | null;
    manualSelections: Record<number, number>;
  } = $props();

  let selectedEntryId = $state<number | null>(null);
  let hasTrackedEntries = $state(false);
  let previousEntries = $state<ShindenEntry[] | null>(null);
  let selectedEntry = $derived(
    entries.find((entry) => entry.id === selectedEntryId) ?? null,
  );
  let selectedMatchEntry = $derived(
    selectedEntryId === null
      ? null
      : (matchResult?.entries.find(
          (entry) => entry.shindenId === selectedEntryId,
        ) ?? null),
  );
  let selectedWinnerId = $derived(selectedMatchEntry?.result.winner?.id ?? null);
  let selectedWinner = $derived(
    selectedWinnerId === null
      ? null
      : (databaseEntries.find((entry) => entry.id === selectedWinnerId) ??
          null),
  );

  $effect(() => {
    if (hasTrackedEntries && previousEntries === entries) {
      return;
    }

    const shouldClearSelection = hasTrackedEntries;
    hasTrackedEntries = true;
    previousEntries = entries;

    if (shouldClearSelection) {
      selectedEntryId = null;
    }
  });

  $effect(() => {
    if (
      selectedEntryId !== null &&
      !entries.some((entry) => entry.id === selectedEntryId)
    ) {
      selectedEntryId = null;
    }
  });
</script>

<section class="workspace-content">
  <div class="workspace-layout">
    <AnimeListPane
      {providerLabel}
      {entries}
      {matchResult}
      {selectedEntryId}
      onSelectEntry={(entryId) => {
        selectedEntryId = entryId;
      }}
    />
    <WorkspaceEditorPane {selectedEntry} {selectedWinner} />
  </div>
</section>

<WorkspaceStatusBar
  {entries}
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
