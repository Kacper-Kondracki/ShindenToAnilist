<script lang="ts">
  import { getAnimeDatabaseEntries } from "../api/appService";
  import type { DatabaseEntry, MatchListResult } from "../domain/anime";
  import AnimeListPane from "./workspace/AnimeListPane.svelte";
  import WorkspaceEditorPane from "./workspace/WorkspaceEditorPane.svelte";
  import WorkspaceStatusBar from "./workspace/WorkspaceStatusBar.svelte";

  let {
    providerLabel,
    entryIds,
    matchResult,
    matchErrorMessage,
    isMatching,
    fetchDurationMs,
    matchDurationMs,
    manualSelections = $bindable(),
  }: {
    providerLabel: string;
    entryIds: number[];
    matchResult: MatchListResult | null;
    matchErrorMessage: string | null;
    isMatching: boolean;
    fetchDurationMs: number | null;
    matchDurationMs: number | null;
    manualSelections: Record<number, number>;
  } = $props();

  let selectedEntryId = $state<number | null>(null);
  let hasTrackedEntries = $state(false);
  let previousEntryIds = $state<number[] | null>(null);
  let databaseEntriesById = $state<Record<number, DatabaseEntry>>({});
  let activeWinnerRequestId = 0;
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
  let selectedWinner = $derived(
    selectedWinnerId === null
      ? null
      : (databaseEntriesById[selectedWinnerId] ?? null),
  );

  $effect(() => {
    if (hasTrackedEntries && previousEntryIds === entryIds) {
      return;
    }

    const shouldClearSelection = hasTrackedEntries;
    hasTrackedEntries = true;
    previousEntryIds = entryIds;
    databaseEntriesById = {};

    if (shouldClearSelection) {
      selectedEntryId = null;
    }
  });

  $effect(() => {
    if (
      selectedEntryId !== null &&
      !entryIds.some((entryId) => entryId === selectedEntryId)
    ) {
      selectedEntryId = null;
    }
  });

  $effect(() => {
    const winnerId = selectedWinnerId;

    if (winnerId === null || databaseEntriesById[winnerId] !== undefined) {
      return;
    }

    const requestId = activeWinnerRequestId + 1;
    activeWinnerRequestId = requestId;

    void loadWinnerEntry(winnerId, requestId);
  });

  async function loadWinnerEntry(winnerId: number, requestId: number) {
    const [entry] = await getAnimeDatabaseEntries([winnerId]);

    if (activeWinnerRequestId !== requestId || entry === undefined) {
      return;
    }

    databaseEntriesById = {
      ...databaseEntriesById,
      [entry.id]: entry,
    };
  }
</script>

<section class="workspace-content">
  <div class="workspace-layout">
    <AnimeListPane
      {providerLabel}
      {entryIds}
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
  {entryIds}
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
