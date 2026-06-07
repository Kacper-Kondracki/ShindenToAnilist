import { exportMatches } from "../../api/appService";
import type { EntryStore } from "../../data/entryStore.svelte";
import type {
  DatabaseEntry,
  LoadedUserList,
  MatchListResult,
  MatchSelection,
  WorkspaceState,
} from "../../domain/anime";

export type ExportState =
  | { status: "idle" }
  | { status: "exporting" }
  | { status: "exported"; path: string; exportedCount: number }
  | { status: "cancelled" }
  | { status: "error"; message: string };

export type WorkspaceActivation = LoadedUserList & {
  matchResult: MatchListResult;
  fetchDurationMs: number;
  matchDurationMs: number;
};

export type WorkspaceController = ReturnType<typeof createWorkspaceController>;

export function createWorkspaceController(entryStore: EntryStore) {
  let state = $state<WorkspaceState>({ status: "empty" });
  let matchResult = $state<MatchListResult | null>(null);
  let matchErrorMessage = $state<string | null>(null);
  let fetchDurationMs = $state<number | null>(null);
  let matchDurationMs = $state<number | null>(null);
  let selectedEntryId = $state<number | null>(null);
  let manualOverrides = $state<Record<number, number>>({});
  let exportState = $state<ExportState>({ status: "idle" });

  let selectedMatchEntry = $derived(
    selectedEntryId === null
      ? null
      : (matchResult?.entries.find(
          (entry) => entry.shindenId === selectedEntryId,
        ) ?? null),
  );
  let selectedWinnerId = $derived.by(() => {
    if (selectedEntryId === null) {
      return null;
    }

    return (
      manualOverrides[selectedEntryId] ??
      selectedMatchEntry?.result.winner?.id ??
      null
    );
  });
  let selectedWinner = $derived<DatabaseEntry | null>(
    entryStore.getDatabaseEntry(selectedWinnerId),
  );
  let effectiveSelections = $derived.by(() =>
    buildEffectiveSelections(matchResult, manualOverrides),
  );
  let canExport = $derived(
    state.status === "active" &&
      effectiveSelections.length > 0 &&
      exportState.status !== "exporting",
  );

  $effect(() => {
    return entryStore.pinDatabaseEntry(selectedWinnerId);
  });

  function activate(next: WorkspaceActivation) {
    entryStore.reset();
    state = {
      status: "active",
      provider: next.provider,
      query: next.query,
      entryIdsByView: next.entryIdsByView,
    };
    matchResult = next.matchResult;
    matchErrorMessage = null;
    fetchDurationMs = next.fetchDurationMs;
    matchDurationMs = next.matchDurationMs;
    selectedEntryId = null;
    manualOverrides = {};
    exportState = { status: "idle" };
  }

  function selectEntry(entryId: number) {
    if (state.status !== "active") {
      return;
    }

    selectedEntryId = state.entryIdsByView.all.some((id) => id === entryId)
      ? entryId
      : null;
  }

  function clearSelectionIfMissing() {
    if (
      state.status === "active" &&
      selectedEntryId !== null &&
      !state.entryIdsByView.all.some((entryId) => entryId === selectedEntryId)
    ) {
      selectedEntryId = null;
    }
  }

  function setManualOverride(shindenId: number, databaseId: number) {
    manualOverrides = {
      ...manualOverrides,
      [shindenId]: databaseId,
    };
    exportState = { status: "idle" };
  }

  function clearManualOverride(shindenId: number) {
    const { [shindenId]: _removed, ...nextOverrides } = manualOverrides;
    manualOverrides = nextOverrides;
    exportState = { status: "idle" };
  }

  async function exportCurrentSelections() {
    if (!canExport) {
      return;
    }

    const selections = effectiveSelections;
    exportState = { status: "exporting" };

    try {
      const result = await exportMatches(selections);
      exportState = result.cancelled
        ? { status: "cancelled" }
        : {
            status: "exported",
            path: result.path,
            exportedCount: result.exportedCount,
          };
    } catch (error) {
      exportState = { status: "error", message: errorMessage(error) };
    }
  }

  return {
    get state() {
      return state;
    },
    get matchResult() {
      return matchResult;
    },
    get matchErrorMessage() {
      return matchErrorMessage;
    },
    get fetchDurationMs() {
      return fetchDurationMs;
    },
    get matchDurationMs() {
      return matchDurationMs;
    },
    get selectedEntryId() {
      return selectedEntryId;
    },
    get selectedWinner() {
      return selectedWinner;
    },
    get manualOverrides() {
      return manualOverrides;
    },
    get exportState() {
      return exportState;
    },
    get effectiveSelections() {
      return effectiveSelections;
    },
    get canExport() {
      return canExport;
    },
    activate,
    selectEntry,
    clearSelectionIfMissing,
    setManualOverride,
    clearManualOverride,
    exportCurrentSelections,
  };
}

function buildEffectiveSelections(
  matchResult: MatchListResult | null,
  manualOverrides: Record<number, number>,
): MatchSelection[] {
  const selections: MatchSelection[] = [];

  for (const entry of matchResult?.entries ?? []) {
    const databaseId =
      manualOverrides[entry.shindenId] ?? entry.result.winner?.id ?? null;

    if (databaseId !== null) {
      selections.push({
        shindenId: entry.shindenId,
        databaseId,
      });
    }
  }

  return selections;
}

function errorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === "string") {
    return error;
  }

  return "Nie udało się wyeksportować dopasowań";
}
