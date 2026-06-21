import { exportXml } from '../../api/appService';
import { toUserFacingErrorMessage } from '../../api/runtime';
import type { LoadedAnimeData } from '../../data/loadedAnimeData.svelte';
import type {
  DatabaseEntry,
  LoadedUserList,
  MatchListResult,
  MatchSelection,
  ShindenMatchResult,
  ShindenListViews,
  WireNumber,
  WireNumberRecord,
  WorkspaceState
} from '../../domain/anime';
import {
  parseWireNumber,
  wireNumberEquals,
  wireNumberKey
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
  | { status: 'no-winner'; selectedEntryId: WireNumber }
  | { status: 'ready'; selectedEntryId: WireNumber; entry: DatabaseEntry }
  | {
      status: 'missing';
      selectedEntryId: WireNumber;
      databaseEntryId: WireNumber;
    };

type PersistedListOverrides = {
  manualOverrides: WireNumberRecord<WireNumber>;
  ignoredEntryIds: WireNumberRecord<true>;
};

export type WorkspaceController = ReturnType<typeof createWorkspaceController>;

export function createWorkspaceController(animeData: LoadedAnimeData) {
  let state = $state<WorkspaceState>({ status: 'empty' });
  let matchResult = $state<MatchListResult | null>(null);
  let matchErrorMessage = $state<string | null>(null);
  let fetchDurationMs = $state<number | null>(null);
  let matchDurationMs = $state<number | null>(null);
  let selectedEntryId = $state<WireNumber | null>(null);
  let initialMatchSearch = $state<MatchSelectorInitialSearch | null>(null);
  let manualOverrideStorageKey = $state<string | null>(null);
  let manualOverridesHydrated = $state(false);
  let manualOverrides = $state<WireNumberRecord<WireNumber>>({});
  let ignoredEntryIds = $state<WireNumberRecord<true>>({});
  let displacedAutomaticEntryIds = $state<WireNumberRecord<true>>({});
  let matchSelectorQueries = $state<WireNumberRecord<string>>({});
  let exportState = $state<ExportState>({ status: 'idle' });
  let selectionRequestId = 0;
  let pendingSelectionEntryId: WireNumber | null = null;

  let matchResultByEntryId = $derived.by(() => {
    const entriesById = new Map<WireNumber, ShindenMatchResult>();
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

    writeListOverrides(manualOverrideStorageKey, {
      manualOverrides,
      ignoredEntryIds
    });
  });

  $effect(() => {
    const activeState = state;
    const databaseVersion = animeData.databaseVersion;

    if (activeState.status !== 'active') {
      return;
    }

    databaseVersion;
    const nextOverrides = pruneUnavailableListOverrides(
      { manualOverrides, ignoredEntryIds },
      activeState.entryIdsByView.all,
      (databaseId) => animeData.getDatabaseEntry(databaseId) !== null
    );

    if (
      nextOverrides.manualOverrides === manualOverrides &&
      nextOverrides.ignoredEntryIds === ignoredEntryIds
    ) {
      return;
    }

    replaceManualState(
      nextOverrides.manualOverrides,
      nextOverrides.ignoredEntryIds
    );
  });

  function activate(next: WorkspaceActivation) {
    const nextManualOverrideStorageKey = storageKeyForManualOverrides(
      next.manualOverrideScopeKey
    );
    const storedListOverrides =
      nextManualOverrideStorageKey === manualOverrideStorageKey
        ? { manualOverrides, ignoredEntryIds }
        : readListOverrides(nextManualOverrideStorageKey);
    const nextListOverrides = pruneUnavailableListOverrides(
      storedListOverrides,
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
    replaceManualState(
      nextListOverrides.manualOverrides,
      nextListOverrides.ignoredEntryIds,
      false
    );
    matchSelectorQueries = {};
    exportState = { status: 'idle' };
  }

  async function selectEntry(entryId: WireNumber) {
    if (state.status !== 'active') {
      return;
    }

    if (
      (selectedEntryId !== null &&
        wireNumberEquals(entryId, selectedEntryId)) ||
      (pendingSelectionEntryId !== null &&
        wireNumberEquals(entryId, pendingSelectionEntryId))
    ) {
      return;
    }

    if (!state.entryIdsByView.all.some((id) => wireNumberEquals(id, entryId))) {
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
    const nextInitialMatchSearch = await loadInitialMatchSelectorSearch(
      selectedEntry,
      matchSelectorQueries[wireNumberKey(entryId)] ?? ''
    );

    if (currentSelectionRequestId !== selectionRequestId) {
      return;
    }

    pendingSelectionEntryId = null;
    initialMatchSearch = nextInitialMatchSearch;
    selectedEntryId = entryId;
  }

  function clearSelectionIfMissing() {
    const currentSelectedEntryId = selectedEntryId;

    if (
      state.status === 'active' &&
      currentSelectedEntryId !== null &&
      !state.entryIdsByView.all.some((entryId) =>
        wireNumberEquals(entryId, currentSelectedEntryId)
      )
    ) {
      selectionRequestId += 1;
      pendingSelectionEntryId = null;
      selectedEntryId = null;
      initialMatchSearch = null;
    }
  }

  function setManualOverride(shindenId: WireNumber, databaseId: WireNumber) {
    const automaticWinnerId = automaticWinnerIdForEntry(shindenId, matchResult);
    if (
      automaticWinnerId !== null &&
      wireNumberEquals(databaseId, automaticWinnerId)
    ) {
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
      [wireNumberKey(shindenId)]: databaseId
    };
    const nextIgnoredEntryIds = omitRecordKey(ignoredEntryIds, shindenId);

    for (const ownerId of duplicateOwnerIds) {
      const ownerKey = wireNumberKey(ownerId);
      if (
        nextOverrides[ownerKey] !== undefined &&
        wireNumberEquals(nextOverrides[ownerKey], databaseId)
      ) {
        delete nextOverrides[ownerKey];
      }
    }

    replaceManualState({ ...nextOverrides }, nextIgnoredEntryIds);
  }

  function clearManualOverride(shindenId: WireNumber) {
    if (automaticWinnerIdForEntry(shindenId, matchResult) !== null) {
      restoreAutomaticWinner(shindenId);
      return;
    }

    replaceManualState(
      omitRecordKey(manualOverrides, shindenId),
      omitRecordKey(ignoredEntryIds, shindenId)
    );
  }

  function setIgnored(shindenId: WireNumber) {
    if (ignoredEntryIds[wireNumberKey(shindenId)] === true) {
      replaceManualState(
        manualOverrides,
        omitRecordKey(ignoredEntryIds, shindenId)
      );
      return;
    }

    replaceManualState(omitRecordKey(manualOverrides, shindenId), {
      ...ignoredEntryIds,
      [wireNumberKey(shindenId)]: true
    });
  }

  function setMatchSelectorQuery(shindenId: WireNumber, query: string) {
    if (query.length === 0) {
      resetMatchSelectorQuery(shindenId);
      return;
    }

    matchSelectorQueries = {
      ...matchSelectorQueries,
      [wireNumberKey(shindenId)]: query
    };
  }

  function resetMatchSelectorQuery(shindenId: WireNumber) {
    matchSelectorQueries = omitRecordKey(matchSelectorQueries, shindenId);
  }

  function restoreAutomaticWinner(shindenId: WireNumber) {
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
      const ownerKey = wireNumberKey(ownerId);
      if (
        nextOverrides[ownerKey] !== undefined &&
        wireNumberEquals(nextOverrides[ownerKey], automaticWinnerId)
      ) {
        delete nextOverrides[ownerKey];
      }
    }

    replaceManualState({ ...nextOverrides }, nextIgnoredEntryIds);
  }

  function clearManualOverrides() {
    replaceManualState({}, {});
  }

  function replaceManualState(
    nextManualOverrides: WireNumberRecord<WireNumber>,
    nextIgnoredEntryIds: WireNumberRecord<true>,
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
  entryId: WireNumber,
  matchEntry: ShindenMatchResult | null,
  manualOverrides: WireNumberRecord<WireNumber>,
  ignoredEntryIds: WireNumberRecord<true>,
  displacedAutomaticEntryIds: WireNumberRecord<true>
) {
  const entryKey = wireNumberKey(entryId);

  if (ignoredEntryIds[entryKey] === true) {
    return null;
  }

  if (manualOverrides[entryKey] !== undefined) {
    return manualOverrides[entryKey];
  }

  if (displacedAutomaticEntryIds[entryKey] === true) {
    return null;
  }

  return matchEntry?.result.winner?.id ?? null;
}

function buildEffectiveSelections(
  matchResult: MatchListResult | null,
  manualOverrides: WireNumberRecord<WireNumber>,
  ignoredEntryIds: WireNumberRecord<true>,
  displacedAutomaticEntryIds: WireNumberRecord<true>
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
        sourceId: entry.sourceId ?? entry.shindenId,
        shindenId: entry.shindenId,
        databaseId
      });
    }
  }

  return selections;
}

function buildWinnerClaimsByDatabaseId(
  matchResult: MatchListResult | null,
  manualOverrides: WireNumberRecord<WireNumber>,
  ignoredEntryIds: WireNumberRecord<true>,
  displacedAutomaticEntryIds: WireNumberRecord<true>
) {
  const claims = new Map<WireNumber, WireNumber[]>();

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
  manualOverrides: WireNumberRecord<WireNumber>,
  ignoredEntryIds: WireNumberRecord<true>,
  displacedAutomaticEntryIds: WireNumberRecord<true>
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
    (entryId) => ignoredEntryIds[wireNumberKey(entryId)] === true
  );
  const baseManualIds = state.entryIdsByView.manual;
  const baseManualIdSet = new Set(baseManualIds.map(wireNumberKey));
  const automaticManualIds = state.entryIdsByView.automatic.filter(
    (entryId) =>
      displacedAutomaticEntryIds[wireNumberKey(entryId)] === true ||
      manualOverrides[wireNumberKey(entryId)] !== undefined
  );

  return {
    manual: [
      ...baseManualIds,
      ...automaticManualIds.filter(
        (entryId) => !baseManualIdSet.has(wireNumberKey(entryId))
      )
    ],
    automatic: state.entryIdsByView.automatic,
    ignored: ignoredIds,
    all: state.entryIdsByView.all
  };
}

function activeWinnerOwnerIdsForDatabase(
  databaseId: WireNumber,
  excludedShindenId: WireNumber,
  matchResult: MatchListResult | null,
  manualOverrides: WireNumberRecord<WireNumber>,
  ignoredEntryIds: WireNumberRecord<true>,
  displacedAutomaticEntryIds: WireNumberRecord<true>
) {
  const ownerIds: WireNumber[] = [];

  for (const entry of matchResult?.entries ?? []) {
    if (wireNumberEquals(entry.shindenId, excludedShindenId)) {
      continue;
    }

    const winnerId = winnerIdForEntry(
      entry.shindenId,
      entry,
      manualOverrides,
      ignoredEntryIds,
      displacedAutomaticEntryIds
    );

    if (winnerId !== null && wireNumberEquals(winnerId, databaseId)) {
      ownerIds.push(entry.shindenId);
    }
  }

  return ownerIds;
}

function buildDisplacedAutomaticEntryIds(
  matchResult: MatchListResult | null,
  manualOverrides: WireNumberRecord<WireNumber>,
  ignoredEntryIds: WireNumberRecord<true>
) {
  const displacedEntryIds: WireNumberRecord<true> = {};

  for (const [rawShindenId, databaseId] of Object.entries(manualOverrides)) {
    const shindenId = parseWireNumber(rawShindenId);
    if (shindenId === null) {
      continue;
    }

    for (const ownerId of activeWinnerOwnerIdsForDatabase(
      databaseId,
      shindenId,
      matchResult,
      manualOverrides,
      ignoredEntryIds,
      {}
    )) {
      const automaticWinnerId = automaticWinnerIdForEntry(ownerId, matchResult);
      if (
        automaticWinnerId !== null &&
        wireNumberEquals(automaticWinnerId, databaseId)
      ) {
        displacedEntryIds[wireNumberKey(ownerId)] = true;
      }
    }
  }

  return displacedEntryIds;
}

function automaticWinnerIdForEntry(
  shindenId: WireNumber,
  matchResult: MatchListResult | null
) {
  return (
    matchResult?.entries.find((entry) =>
      wireNumberEquals(entry.shindenId, shindenId)
    )?.result.winner?.id ?? null
  );
}

function omitRecordKey<T>(record: WireNumberRecord<T>, key: WireNumber) {
  const { [wireNumberKey(key)]: _removed, ...nextRecord } = record;

  return nextRecord;
}

function pruneUnavailableListOverrides(
  listOverrides: PersistedListOverrides,
  availableShindenIds: readonly WireNumber[],
  hasDatabaseEntry: (databaseId: WireNumber) => boolean
) {
  const availableShindenIdSet = new Set(availableShindenIds.map(wireNumberKey));
  const manualOverrides = listOverrides.manualOverrides;
  const ignoredEntryIds = listOverrides.ignoredEntryIds;
  const nextManualOverrides: WireNumberRecord<WireNumber> = {};
  const nextIgnoredEntryIds: WireNumberRecord<true> = {};
  let manualOverridesChanged = false;
  let ignoredEntryIdsChanged = false;

  for (const [rawShindenId, databaseId] of Object.entries(manualOverrides)) {
    if (
      !availableShindenIdSet.has(rawShindenId) ||
      !hasDatabaseEntry(databaseId)
    ) {
      manualOverridesChanged = true;
      continue;
    }

    nextManualOverrides[rawShindenId] = databaseId;
  }

  for (const rawShindenId of Object.keys(ignoredEntryIds)) {
    if (!availableShindenIdSet.has(rawShindenId)) {
      ignoredEntryIdsChanged = true;
      continue;
    }

    nextIgnoredEntryIds[rawShindenId] = true;
  }

  return {
    manualOverrides: manualOverridesChanged
      ? nextManualOverrides
      : manualOverrides,
    ignoredEntryIds: ignoredEntryIdsChanged
      ? nextIgnoredEntryIds
      : ignoredEntryIds
  };
}

const manualOverrideStoragePrefix = 'shinden-to-anilist:manual-overrides:v1:';

function storageKeyForManualOverrides(scopeKey: string) {
  return `${manualOverrideStoragePrefix}${scopeKey}`;
}

function readListOverrides(storageKey: string): PersistedListOverrides {
  const storage = getLocalStorage();
  if (storage === null) {
    return emptyListOverrides();
  }

  try {
    return normalizeListOverrides(
      JSON.parse(storage.getItem(storageKey) ?? 'null')
    );
  } catch {
    return emptyListOverrides();
  }
}

function writeListOverrides(
  storageKey: string,
  listOverrides: PersistedListOverrides
) {
  const storage = getLocalStorage();
  if (storage === null) {
    return;
  }

  try {
    if (
      Object.keys(listOverrides.manualOverrides).length === 0 &&
      Object.keys(listOverrides.ignoredEntryIds).length === 0
    ) {
      storage.removeItem(storageKey);
      return;
    }

    storage.setItem(
      storageKey,
      JSON.stringify({
        overrides: stringifyWireNumberRecord(listOverrides.manualOverrides),
        ignoredEntryIds: listOverrides.ignoredEntryIds
      })
    );
  } catch {
    // Storage is best-effort; the in-memory workspace should keep working.
  }
}

function normalizeListOverrides(value: unknown): PersistedListOverrides {
  if (!isRecord(value)) {
    return emptyListOverrides();
  }

  return {
    manualOverrides: normalizeManualOverrides(value),
    ignoredEntryIds: normalizeTrueRecord(value.ignoredEntryIds)
  };
}

function normalizeManualOverrides(
  value: unknown
): WireNumberRecord<WireNumber> {
  const overrides =
    isRecord(value) && isRecord(value.overrides) ? value.overrides : value;
  if (!isRecord(overrides)) {
    return {};
  }

  const normalized: WireNumberRecord<WireNumber> = {};
  for (const [rawShindenId, rawDatabaseId] of Object.entries(overrides)) {
    const shindenId = parseWireNumber(rawShindenId);
    const databaseId = parseWireNumber(rawDatabaseId);

    if (shindenId !== null && databaseId !== null) {
      normalized[wireNumberKey(shindenId)] = databaseId;
    }
  }

  return normalized;
}

function normalizeTrueRecord(value: unknown): WireNumberRecord<true> {
  if (!isRecord(value)) {
    return {};
  }

  const normalized: WireNumberRecord<true> = {};
  for (const [rawEntryId, flag] of Object.entries(value)) {
    const entryId = parseWireNumber(rawEntryId);

    if (flag === true && entryId !== null) {
      normalized[wireNumberKey(entryId)] = true;
    }
  }

  return normalized;
}

function emptyListOverrides(): PersistedListOverrides {
  return {
    manualOverrides: {},
    ignoredEntryIds: {}
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function getLocalStorage() {
  return typeof localStorage === 'undefined' ? null : localStorage;
}

function recordsEqual<T>(
  left: WireNumberRecord<T>,
  right: WireNumberRecord<T>
) {
  const leftKeys = Object.keys(left);
  const rightKeys = Object.keys(right);

  if (leftKeys.length !== rightKeys.length) {
    return false;
  }

  return leftKeys.every((key) => left[key] === right[key]);
}

function stringifyWireNumberRecord(record: WireNumberRecord<WireNumber>) {
  return Object.fromEntries(
    Object.entries(record).map(([key, value]) => [key, value.toString()])
  );
}

function errorMessage(error: unknown) {
  return toUserFacingErrorMessage(
    error,
    'Nie udało się wyeksportować dopasowań'
  );
}
