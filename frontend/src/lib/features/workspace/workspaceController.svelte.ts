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
  manualOverrideScopeKey: string;
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
  let manualOverrideStorageKey = $state<string | null>(null);
  let manualOverridesHydrated = $state(false);
  let manualOverrides = $state<Record<number, number>>({});
  let ignoredEntryIds = $state<Record<number, true>>({});
  let displacedAutomaticEntryIds = $state<Record<number, true>>({});
  let matchSelectorQueries = $state<Record<number, string>>({});
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

  $effect(() => {
    if (!manualOverridesHydrated || manualOverrideStorageKey === null) {
      return;
    }

    writeManualOverrides(manualOverrideStorageKey, manualOverrides);
  });

  $effect(() => {
    const activeState = state;
    const databaseVersion = animeData.databaseVersion;

    if (activeState.status !== 'active') {
      return;
    }

    databaseVersion;
    const nextManualOverrides = pruneUnavailableManualOverrides(
      manualOverrides,
      activeState.entryIdsByView.all,
      (databaseId) => animeData.getDatabaseEntry(databaseId) !== null
    );

    if (nextManualOverrides === manualOverrides) {
      return;
    }

    replaceManualState(nextManualOverrides, ignoredEntryIds);
  });

  function activate(next: WorkspaceActivation) {
    const nextManualOverrideStorageKey = storageKeyForManualOverrides(
      next.manualOverrideScopeKey
    );
    const storedManualOverrides =
      nextManualOverrideStorageKey === manualOverrideStorageKey
        ? manualOverrides
        : readManualOverrides(nextManualOverrideStorageKey);
    const nextManualOverrides = pruneUnavailableManualOverrides(
      storedManualOverrides,
      next.entryIdsByView.all,
      (databaseId) => animeData.getDatabaseEntry(databaseId) !== null
    );

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
    manualOverrideStorageKey = nextManualOverrideStorageKey;
    manualOverridesHydrated = true;
    replaceManualState(nextManualOverrides, {}, false);
    matchSelectorQueries = {};
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

    for (const ownerId of duplicateOwnerIds) {
      if (nextOverrides[ownerId] === databaseId) {
        delete nextOverrides[ownerId];
      }
    }

    replaceManualState({ ...nextOverrides }, nextIgnoredEntryIds);
  }

  function clearManualOverride(shindenId: number) {
    if (automaticWinnerIdForEntry(shindenId, matchResult) !== null) {
      restoreAutomaticWinner(shindenId);
      return;
    }

    replaceManualState(
      omitRecordKey(manualOverrides, shindenId),
      omitRecordKey(ignoredEntryIds, shindenId)
    );
  }

  function setIgnored(shindenId: number) {
    if (ignoredEntryIds[shindenId] === true) {
      replaceManualState(
        manualOverrides,
        omitRecordKey(ignoredEntryIds, shindenId)
      );
      return;
    }

    replaceManualState(omitRecordKey(manualOverrides, shindenId), {
      ...ignoredEntryIds,
      [shindenId]: true
    });
  }

  function setMatchSelectorQuery(shindenId: number, query: string) {
    if (query.length === 0) {
      resetMatchSelectorQuery(shindenId);
      return;
    }

    matchSelectorQueries = {
      ...matchSelectorQueries,
      [shindenId]: query
    };
  }

  function resetMatchSelectorQuery(shindenId: number) {
    matchSelectorQueries = omitRecordKey(matchSelectorQueries, shindenId);
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

    for (const ownerId of duplicateOwnerIds) {
      if (nextOverrides[ownerId] === automaticWinnerId) {
        delete nextOverrides[ownerId];
      }
    }

    replaceManualState({ ...nextOverrides }, nextIgnoredEntryIds);
  }

  function clearManualOverrides() {
    replaceManualState({}, ignoredEntryIds);
  }

  function replaceManualState(
    nextManualOverrides: Record<number, number>,
    nextIgnoredEntryIds: Record<number, true>,
    resetExportState = true
  ) {
    const nextDisplacedAutomaticEntryIds = buildDisplacedAutomaticEntryIds(
      matchResult,
      nextManualOverrides,
      nextIgnoredEntryIds
    );

    manualOverrides = nextManualOverrides;
    ignoredEntryIds = nextIgnoredEntryIds;

    if (
      !recordsEqual(displacedAutomaticEntryIds, nextDisplacedAutomaticEntryIds)
    ) {
      displacedAutomaticEntryIds = nextDisplacedAutomaticEntryIds;
    }

    if (resetExportState) {
      exportState = { status: 'idle' };
    }
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
    get matchSelectorQueries() {
      return matchSelectorQueries;
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
    clearManualOverrides,
    setMatchSelectorQuery,
    resetMatchSelectorQuery,
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
      displacedAutomaticEntryIds[entryId] === true ||
      manualOverrides[entryId] !== undefined
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

function buildDisplacedAutomaticEntryIds(
  matchResult: MatchListResult | null,
  manualOverrides: Record<number, number>,
  ignoredEntryIds: Record<number, true>
) {
  const displacedEntryIds: Record<number, true> = {};

  for (const [rawShindenId, databaseId] of Object.entries(manualOverrides)) {
    const shindenId = Number(rawShindenId);

    for (const ownerId of activeWinnerOwnerIdsForDatabase(
      databaseId,
      shindenId,
      matchResult,
      manualOverrides,
      ignoredEntryIds,
      {}
    )) {
      if (automaticWinnerIdForEntry(ownerId, matchResult) === databaseId) {
        displacedEntryIds[ownerId] = true;
      }
    }
  }

  return displacedEntryIds;
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

function pruneUnavailableManualOverrides(
  manualOverrides: Record<number, number>,
  availableShindenIds: readonly number[],
  hasDatabaseEntry: (databaseId: number) => boolean
) {
  const availableShindenIdSet = new Set(availableShindenIds);
  const nextManualOverrides: Record<number, number> = {};
  let changed = false;

  for (const [rawShindenId, databaseId] of Object.entries(manualOverrides)) {
    const shindenId = Number(rawShindenId);

    if (
      !availableShindenIdSet.has(shindenId) ||
      !hasDatabaseEntry(databaseId)
    ) {
      changed = true;
      continue;
    }

    nextManualOverrides[shindenId] = databaseId;
  }

  return changed ? nextManualOverrides : manualOverrides;
}

const manualOverrideStoragePrefix = 'shinden-to-anilist:manual-overrides:v1:';

function storageKeyForManualOverrides(scopeKey: string) {
  return `${manualOverrideStoragePrefix}${scopeKey}`;
}

function readManualOverrides(storageKey: string): Record<number, number> {
  const storage = getLocalStorage();
  if (storage === null) {
    return {};
  }

  try {
    return normalizeManualOverrides(
      JSON.parse(storage.getItem(storageKey) ?? 'null')
    );
  } catch {
    return {};
  }
}

function writeManualOverrides(
  storageKey: string,
  manualOverrides: Record<number, number>
) {
  const storage = getLocalStorage();
  if (storage === null) {
    return;
  }

  try {
    if (Object.keys(manualOverrides).length === 0) {
      storage.removeItem(storageKey);
      return;
    }

    storage.setItem(storageKey, JSON.stringify({ overrides: manualOverrides }));
  } catch {
    // Storage is best-effort; the in-memory workspace should keep working.
  }
}

function normalizeManualOverrides(value: unknown): Record<number, number> {
  const overrides =
    isRecord(value) && isRecord(value.overrides) ? value.overrides : value;
  if (!isRecord(overrides)) {
    return {};
  }

  const normalized: Record<number, number> = {};
  for (const [rawShindenId, rawDatabaseId] of Object.entries(overrides)) {
    const shindenId = Number(rawShindenId);
    const databaseId = Number(rawDatabaseId);

    if (Number.isSafeInteger(shindenId) && Number.isSafeInteger(databaseId)) {
      normalized[shindenId] = databaseId;
    }
  }

  return normalized;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function getLocalStorage() {
  return typeof localStorage === 'undefined' ? null : localStorage;
}

function recordsEqual<T>(left: Record<number, T>, right: Record<number, T>) {
  const leftKeys = Object.keys(left);
  const rightKeys = Object.keys(right);

  if (leftKeys.length !== rightKeys.length) {
    return false;
  }

  return leftKeys.every((key) => left[Number(key)] === right[Number(key)]);
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
