import { exportXml } from '../../api/appService';
import type { LoadedAnimeData } from '../../data/loadedAnimeData.svelte';
import type {
  DatabaseEntry,
  LoadedUserList,
  MatchListResult,
  MatchSelection,
  ShindenMatchResult,
  ShindenListViews,
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
  let ignoredEntryIds = $state<Record<number, true>>({});
  let displacedAutomaticEntryIds = $state<Record<number, true>>({});
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
      manualOverrides,
      ignoredEntryIds,
      displacedAutomaticEntryIds
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
    buildEffectiveSelections(
      matchResult,
      manualOverrides,
      ignoredEntryIds,
      displacedAutomaticEntryIds
    )
  );
  let winnerClaimsByDatabaseId = $derived.by(() =>
    buildWinnerClaimsByDatabaseId(
      matchResult,
      manualOverrides,
      ignoredEntryIds,
      displacedAutomaticEntryIds
    )
  );
  let entryIdsByView = $derived.by(() =>
    buildEntryIdsByView(
      state,
      manualOverrides,
      ignoredEntryIds,
      displacedAutomaticEntryIds
    )
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
    ignoredEntryIds = {};
    displacedAutomaticEntryIds = {};
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
    if (databaseId === automaticWinnerIdForEntry(shindenId, matchResult)) {
      restoreAutomaticWinner(shindenId);
      return;
    }

    const duplicateOwnerIds = activeWinnerOwnerIdsForDatabase(
      databaseId,
      shindenId,
      matchResult,
      manualOverrides,
      ignoredEntryIds,
      displacedAutomaticEntryIds
    );
    const nextOverrides = {
      ...manualOverrides,
      [shindenId]: databaseId
    };
    const nextIgnoredEntryIds = omitRecordKey(ignoredEntryIds, shindenId);
    const nextDisplacedAutomaticEntryIds = {
      ...displacedAutomaticEntryIds
    };

    for (const ownerId of duplicateOwnerIds) {
      if (nextOverrides[ownerId] === databaseId) {
        delete nextOverrides[ownerId];
      }

      if (automaticWinnerIdForEntry(ownerId, matchResult) === databaseId) {
        nextDisplacedAutomaticEntryIds[ownerId] = true;
      }
    }

    manualOverrides = {
      ...nextOverrides
    };
    ignoredEntryIds = nextIgnoredEntryIds;
    displacedAutomaticEntryIds = nextDisplacedAutomaticEntryIds;
    exportState = { status: 'idle' };
  }

  function clearManualOverride(shindenId: number) {
    if (automaticWinnerIdForEntry(shindenId, matchResult) !== null) {
      restoreAutomaticWinner(shindenId);
      return;
    }

    manualOverrides = omitRecordKey(manualOverrides, shindenId);
    ignoredEntryIds = omitRecordKey(ignoredEntryIds, shindenId);
    exportState = { status: 'idle' };
  }

  function setIgnored(shindenId: number) {
    if (ignoredEntryIds[shindenId] === true) {
      ignoredEntryIds = omitRecordKey(ignoredEntryIds, shindenId);
      exportState = { status: 'idle' };
      return;
    }

    manualOverrides = omitRecordKey(manualOverrides, shindenId);
    ignoredEntryIds = {
      ...ignoredEntryIds,
      [shindenId]: true
    };
    exportState = { status: 'idle' };
  }

  function restoreAutomaticWinner(shindenId: number) {
    const automaticWinnerId = automaticWinnerIdForEntry(shindenId, matchResult);

    if (automaticWinnerId === null) {
      return;
    }

    const duplicateOwnerIds = activeWinnerOwnerIdsForDatabase(
      automaticWinnerId,
      shindenId,
      matchResult,
      manualOverrides,
      ignoredEntryIds,
      displacedAutomaticEntryIds
    );
    const nextOverrides = omitRecordKey(manualOverrides, shindenId);
    const nextIgnoredEntryIds = omitRecordKey(ignoredEntryIds, shindenId);
    const nextDisplacedAutomaticEntryIds = omitRecordKey(
      displacedAutomaticEntryIds,
      shindenId
    );

    for (const ownerId of duplicateOwnerIds) {
      if (nextOverrides[ownerId] === automaticWinnerId) {
        delete nextOverrides[ownerId];
      }

      if (
        automaticWinnerIdForEntry(ownerId, matchResult) === automaticWinnerId
      ) {
        nextDisplacedAutomaticEntryIds[ownerId] = true;
      }
    }

    manualOverrides = {
      ...nextOverrides
    };
    ignoredEntryIds = nextIgnoredEntryIds;
    displacedAutomaticEntryIds = nextDisplacedAutomaticEntryIds;
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
    get entryIdsByView() {
      return entryIdsByView;
    },
    get manualOverrides() {
      return manualOverrides;
    },
    get ignoredEntryIds() {
      return ignoredEntryIds;
    },
    get displacedAutomaticEntryIds() {
      return displacedAutomaticEntryIds;
    },
    get winnerClaimsByDatabaseId() {
      return winnerClaimsByDatabaseId;
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
    setIgnored,
    clearManualOverride,
    exportCurrentSelections
  };
}

function winnerIdForEntry(
  entryId: number,
  matchEntry: ShindenMatchResult | null,
  manualOverrides: Record<number, number>,
  ignoredEntryIds: Record<number, true>,
  displacedAutomaticEntryIds: Record<number, true>
) {
  if (ignoredEntryIds[entryId] === true) {
    return null;
  }

  if (manualOverrides[entryId] !== undefined) {
    return manualOverrides[entryId];
  }

  if (displacedAutomaticEntryIds[entryId] === true) {
    return null;
  }

  return matchEntry?.result.winner?.id ?? null;
}

function buildEffectiveSelections(
  matchResult: MatchListResult | null,
  manualOverrides: Record<number, number>,
  ignoredEntryIds: Record<number, true>,
  displacedAutomaticEntryIds: Record<number, true>
): MatchSelection[] {
  const selections: MatchSelection[] = [];

  for (const entry of matchResult?.entries ?? []) {
    const databaseId = winnerIdForEntry(
      entry.shindenId,
      entry,
      manualOverrides,
      ignoredEntryIds,
      displacedAutomaticEntryIds
    );

    if (databaseId !== null) {
      selections.push({
        shindenId: entry.shindenId,
        databaseId
      });
    }
  }

  return selections;
}

function buildWinnerClaimsByDatabaseId(
  matchResult: MatchListResult | null,
  manualOverrides: Record<number, number>,
  ignoredEntryIds: Record<number, true>,
  displacedAutomaticEntryIds: Record<number, true>
) {
  const claims = new Map<number, number[]>();

  for (const entry of matchResult?.entries ?? []) {
    const databaseId = winnerIdForEntry(
      entry.shindenId,
      entry,
      manualOverrides,
      ignoredEntryIds,
      displacedAutomaticEntryIds
    );

    if (databaseId === null) {
      continue;
    }

    claims.set(databaseId, [
      ...(claims.get(databaseId) ?? []),
      entry.shindenId
    ]);
  }

  return claims;
}

function buildEntryIdsByView(
  state: WorkspaceState,
  manualOverrides: Record<number, number>,
  ignoredEntryIds: Record<number, true>,
  displacedAutomaticEntryIds: Record<number, true>
): ShindenListViews {
  if (state.status !== 'active') {
    return {
      manual: [],
      automatic: [],
      ignored: [],
      all: []
    };
  }

  const ignoredIds = state.entryIdsByView.all.filter(
    (entryId) => ignoredEntryIds[entryId] === true
  );
  const baseManualIds = state.entryIdsByView.manual;
  const baseManualIdSet = new Set(baseManualIds);
  const automaticManualIds = state.entryIdsByView.automatic.filter(
    (entryId) =>
      (displacedAutomaticEntryIds[entryId] === true ||
        manualOverrides[entryId] !== undefined)
  );

  return {
    manual: [
      ...baseManualIds,
      ...automaticManualIds.filter((entryId) => !baseManualIdSet.has(entryId))
    ],
    automatic: state.entryIdsByView.automatic,
    ignored: ignoredIds,
    all: state.entryIdsByView.all
  };
}

function activeWinnerOwnerIdsForDatabase(
  databaseId: number,
  excludedShindenId: number,
  matchResult: MatchListResult | null,
  manualOverrides: Record<number, number>,
  ignoredEntryIds: Record<number, true>,
  displacedAutomaticEntryIds: Record<number, true>
) {
  const ownerIds: number[] = [];

  for (const entry of matchResult?.entries ?? []) {
    if (entry.shindenId === excludedShindenId) {
      continue;
    }

    const winnerId = winnerIdForEntry(
      entry.shindenId,
      entry,
      manualOverrides,
      ignoredEntryIds,
      displacedAutomaticEntryIds
    );

    if (winnerId === databaseId) {
      ownerIds.push(entry.shindenId);
    }
  }

  return ownerIds;
}

function automaticWinnerIdForEntry(
  shindenId: number,
  matchResult: MatchListResult | null
) {
  return (
    matchResult?.entries.find((entry) => entry.shindenId === shindenId)?.result
      .winner?.id ?? null
  );
}

function omitRecordKey<T>(record: Record<number, T>, key: number) {
  const { [key]: _removed, ...nextRecord } = record;

  return nextRecord;
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
