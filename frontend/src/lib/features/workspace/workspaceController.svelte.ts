import { exportXml } from '../../api/appService';
import type { LoadedAnimeData } from '../../data/loadedAnimeData.svelte';
import type {
  DatabaseEntry,
  LoadedUserList,
  MatchListResult,
  MatchSelection,
  ShindenMatchResult,
  WorkspaceState
} from '../../domain/anime';
import {
  loadInitialMatchSelectorSearch,
  type MatchSelectorInitialSearch
} from './matchSelectorController.svelte';

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
  | { status: 'ready'; selectedEntryId: number; entry: DatabaseEntry }
  | { status: 'missing'; selectedEntryId: number; databaseEntryId: number };

export type WorkspaceController = ReturnType<typeof createWorkspaceController>;

export function createWorkspaceController(animeData: LoadedAnimeData) {
  let state = $state<WorkspaceState>({ status: 'empty' });
  let matchResult = $state<MatchListResult | null>(null);
  let matchErrorMessage = $state<string | null>(null);
  let fetchDurationMs = $state<number | null>(null);
  let matchDurationMs = $state<number | null>(null);
  let selectedEntryId = $state<number | null>(null);
  let initialMatchSearch = $state<MatchSelectorInitialSearch | null>(null);
  let manualOverrides = $state<Record<number, number>>({});
  let exportState = $state<ExportState>({ status: 'idle' });
  let selectionRequestId = 0;
  let pendingSelectionEntryId: number | null = null;

  let matchResultByEntryId = $derived.by(() => {
    const entriesById = new Map<number, ShindenMatchResult>();
    for (const entry of matchResult?.entries ?? []) {
      entriesById.set(entry.shindenId, entry);
    }

    return entriesById;
  });
  let selectedMatchEntry = $derived.by(() =>
    selectedEntryId === null
      ? null
      : (matchResultByEntryId.get(selectedEntryId) ?? null)
  );
  let selectedWinnerId = $derived.by(() => {
    if (selectedEntryId === null) {
      return null;
    }

    return winnerIdForEntry(
      selectedEntryId,
      selectedMatchEntry,
      manualOverrides
    );
  });
  let selectedWinnerState = $derived.by((): SelectedWinnerState => {
    if (selectedEntryId === null) {
      return { status: 'no-selection' };
    }

    if (selectedWinnerId === null) {
      return { status: 'no-winner', selectedEntryId };
    }

    const entry = animeData.getDatabaseEntry(selectedWinnerId);
    if (entry !== null) {
      return {
        status: 'ready',
        selectedEntryId,
        entry
      };
    }

    return {
      status: 'missing',
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

  function activate(next: WorkspaceActivation) {
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
    selectionRequestId += 1;
    pendingSelectionEntryId = null;
    selectedEntryId = null;
    initialMatchSearch = null;
    manualOverrides = {};
    exportState = { status: 'idle' };
  }

  async function selectEntry(entryId: number) {
    if (state.status !== 'active') {
      return;
    }

    if (entryId === selectedEntryId || entryId === pendingSelectionEntryId) {
      return;
    }

    if (!state.entryIdsByView.all.some((id) => id === entryId)) {
      selectionRequestId += 1;
      pendingSelectionEntryId = null;
      selectedEntryId = null;
      initialMatchSearch = null;
      return;
    }

    const selectedEntry = animeData.getShindenEntry(entryId);
    if (selectedEntry === null) {
      selectionRequestId += 1;
      pendingSelectionEntryId = null;
      initialMatchSearch = null;
      selectedEntryId = entryId;
      return;
    }

    const currentSelectionRequestId = ++selectionRequestId;
    pendingSelectionEntryId = entryId;
    const nextInitialMatchSearch =
      await loadInitialMatchSelectorSearch(selectedEntry);

    if (currentSelectionRequestId !== selectionRequestId) {
      return;
    }

    pendingSelectionEntryId = null;
    initialMatchSearch = nextInitialMatchSearch;
    selectedEntryId = entryId;
  }

  function clearSelectionIfMissing() {
    if (
      state.status === 'active' &&
      selectedEntryId !== null &&
      !state.entryIdsByView.all.some((entryId) => entryId === selectedEntryId)
    ) {
      selectionRequestId += 1;
      pendingSelectionEntryId = null;
      selectedEntryId = null;
      initialMatchSearch = null;
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
      const result = await exportXml(selections);
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
    get initialMatchSearch() {
      return initialMatchSearch;
    },
    get selectedMatchEntry() {
      return selectedMatchEntry;
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
  matchEntry: ShindenMatchResult | null,
  manualOverrides: Record<number, number>
) {
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
