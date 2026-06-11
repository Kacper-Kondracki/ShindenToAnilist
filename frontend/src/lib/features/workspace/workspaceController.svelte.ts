import { exportMatches } from '../../api/appService';
import type { EntryStore } from '../../data/entryStore.svelte';
import type {
  DatabaseEntry,
  LoadedUserList,
  MatchListResult,
  MatchSelection,
  WorkspaceState
} from '../../domain/anime';

export type ExportState =
  | { status: 'idle' }
  | { status: 'exporting' }
  | { status: 'exported'; path: string; exportedCount: number }
  | { status: 'cancelled' }
  | { status: 'error'; message: string };

export type WorkspaceActivation = LoadedUserList & {
  matchResult: MatchListResult;
  fetchDurationMs: number;
  matchDurationMs: number;
};

export type SelectedWinnerState =
  | { status: 'no-selection' }
  | { status: 'no-winner'; selectedEntryId: number }
  | { status: 'loading'; selectedEntryId: number; databaseEntryId: number }
  | { status: 'ready'; selectedEntryId: number; entry: DatabaseEntry }
  | { status: 'missing'; selectedEntryId: number; databaseEntryId: number }
  | {
      status: 'error';
      selectedEntryId: number;
      databaseEntryId: number;
      message: string;
    };

export type WorkspaceController = ReturnType<typeof createWorkspaceController>;

export function createWorkspaceController(entryStore: EntryStore) {
  let state = $state<WorkspaceState>({ status: 'empty' });
  let matchResult = $state<MatchListResult | null>(null);
  let matchErrorMessage = $state<string | null>(null);
  let fetchDurationMs = $state<number | null>(null);
  let matchDurationMs = $state<number | null>(null);
  let selectedEntryId = $state<number | null>(null);
  let manualOverrides = $state<Record<number, number>>({});
  let exportState = $state<ExportState>({ status: 'idle' });
  let selectionRequestId = 0;

  let selectedMatchEntry = $derived(
    selectedEntryId === null
      ? null
      : (matchResult?.entries.find(
          (entry) => entry.shindenId === selectedEntryId
        ) ?? null)
  );
  let selectedWinnerId = $derived.by(() => {
    if (selectedEntryId === null) {
      return null;
    }

    return winnerIdForEntry(selectedEntryId, matchResult, manualOverrides);
  });
  let selectedWinnerState = $derived.by((): SelectedWinnerState => {
    if (selectedEntryId === null) {
      return { status: 'no-selection' };
    }

    if (selectedWinnerId === null) {
      return { status: 'no-winner', selectedEntryId };
    }

    const entryState = entryStore.getDatabaseEntryState(selectedWinnerId);
    if (entryState.status === 'ready') {
      return {
        status: 'ready',
        selectedEntryId,
        entry: entryState.entry
      };
    }

    if (entryState.status === 'error') {
      return {
        status: 'error',
        selectedEntryId,
        databaseEntryId: selectedWinnerId,
        message: entryState.message
      };
    }

    if (entryState.status === 'missing') {
      return {
        status: 'missing',
        selectedEntryId,
        databaseEntryId: selectedWinnerId
      };
    }

    return {
      status: 'loading',
      selectedEntryId,
      databaseEntryId: selectedWinnerId
    };
  });
  let effectiveSelections = $derived.by(() =>
    buildEffectiveSelections(matchResult, manualOverrides)
  );
  let canExport = $derived(
    state.status === 'active' &&
      effectiveSelections.length > 0 &&
      exportState.status !== 'exporting'
  );

  $effect(() => {
    return entryStore.pinDatabaseEntry(selectedWinnerId);
  });

  function activate(next: WorkspaceActivation) {
    selectionRequestId += 1;
    entryStore.reset();
    state = {
      status: 'active',
      provider: next.provider,
      query: next.query,
      entryIdsByView: next.entryIdsByView
    };
    matchResult = next.matchResult;
    matchErrorMessage = null;
    fetchDurationMs = next.fetchDurationMs;
    matchDurationMs = next.matchDurationMs;
    selectedEntryId = null;
    manualOverrides = {};
    exportState = { status: 'idle' };
  }

  async function selectEntry(entryId: number) {
    if (state.status !== 'active') {
      return;
    }

    if (!state.entryIdsByView.all.some((id) => id === entryId)) {
      selectionRequestId += 1;
      selectedEntryId = null;
      return;
    }

    const requestId = selectionRequestId + 1;
    selectionRequestId = requestId;

    const nextWinnerId = winnerIdForEntry(
      entryId,
      matchResult,
      manualOverrides
    );
    if (nextWinnerId !== null) {
      await entryStore.ensureReadyDatabaseEntry(nextWinnerId);
    }

    if (
      selectionRequestId !== requestId ||
      state.status !== 'active' ||
      !state.entryIdsByView.all.some((id) => id === entryId)
    ) {
      return;
    }

    selectedEntryId = entryId;
  }

  function clearSelectionIfMissing() {
    if (
      state.status === 'active' &&
      selectedEntryId !== null &&
      !state.entryIdsByView.all.some((entryId) => entryId === selectedEntryId)
    ) {
      selectedEntryId = null;
    }
  }

  function setManualOverride(shindenId: number, databaseId: number) {
    manualOverrides = {
      ...manualOverrides,
      [shindenId]: databaseId
    };
    exportState = { status: 'idle' };
  }

  function clearManualOverride(shindenId: number) {
    const { [shindenId]: _removed, ...nextOverrides } = manualOverrides;
    manualOverrides = nextOverrides;
    exportState = { status: 'idle' };
  }

  async function exportCurrentSelections() {
    if (!canExport) {
      return;
    }

    const selections = effectiveSelections;
    exportState = { status: 'exporting' };

    try {
      const result = await exportMatches(selections);
      exportState = result.cancelled
        ? { status: 'cancelled' }
        : {
            status: 'exported',
            path: result.path,
            exportedCount: result.exportedCount
          };
    } catch (error) {
      exportState = { status: 'error', message: errorMessage(error) };
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
    get selectedWinnerState() {
      return selectedWinnerState;
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
    exportCurrentSelections
  };
}

function winnerIdForEntry(
  entryId: number,
  matchResult: MatchListResult | null,
  manualOverrides: Record<number, number>
) {
  const matchEntry =
    matchResult?.entries.find((entry) => entry.shindenId === entryId) ?? null;

  return manualOverrides[entryId] ?? matchEntry?.result.winner?.id ?? null;
}

function buildEffectiveSelections(
  matchResult: MatchListResult | null,
  manualOverrides: Record<number, number>
): MatchSelection[] {
  const selections: MatchSelection[] = [];

  for (const entry of matchResult?.entries ?? []) {
    const databaseId =
      manualOverrides[entry.shindenId] ?? entry.result.winner?.id ?? null;

    if (databaseId !== null) {
      selections.push({
        shindenId: entry.shindenId,
        databaseId
      });
    }
  }

  return selections;
}

function errorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === 'string') {
    return error;
  }

  return 'Nie udało się wyeksportować dopasowań';
}
