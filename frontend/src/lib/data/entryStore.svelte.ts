import {
  getAnimeDatabaseEntries,
  getLoadedShindenEntries,
} from "../api/appService";
import type { DatabaseEntry, ShindenEntry } from "../domain/anime";

type CacheEntry<T> = {
  value: T;
  retainCount: number;
  pinned: boolean;
  lastAccess: number;
};

export type EntryLoadState<T> =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "ready"; entry: T }
  | { status: "missing" }
  | { status: "error"; message: string };

type CacheKind = "shinden" | "database";

const shindenReleasedCapacity = 200;
const databaseReleasedCapacity = 50;

export type EntryStore = ReturnType<typeof createEntryStore>;

export function createEntryStore() {
  let shindenEntries = $state<Record<number, ShindenEntry>>({});
  let databaseEntries = $state<Record<number, DatabaseEntry>>({});
  let shindenEntryStates = $state<Record<number, EntryLoadState<ShindenEntry>>>(
    {},
  );
  let databaseEntryStates = $state<
    Record<number, EntryLoadState<DatabaseEntry>>
  >({});
  let tick = 0;
  let generation = 0;

  const shindenCache = new Map<number, CacheEntry<ShindenEntry>>();
  const databaseCache = new Map<number, CacheEntry<DatabaseEntry>>();
  const pendingShindenIds = new Set<number>();
  const pendingDatabaseIds = new Set<number>();
  const inFlightShindenIds = new Set<number>();
  const inFlightDatabaseIds = new Set<number>();
  const shindenRetainCounts = new Map<number, number>();
  const pinnedDatabaseIds = new Set<number>();
  let shindenBatchScheduled = false;
  let databaseBatchScheduled = false;

  function reset() {
    generation += 1;
    shindenCache.clear();
    databaseCache.clear();
    pendingShindenIds.clear();
    pendingDatabaseIds.clear();
    inFlightShindenIds.clear();
    inFlightDatabaseIds.clear();
    shindenRetainCounts.clear();
    pinnedDatabaseIds.clear();
    shindenBatchScheduled = false;
    databaseBatchScheduled = false;
    shindenEntries = {};
    databaseEntries = {};
    shindenEntryStates = {};
    databaseEntryStates = {};
  }

  function getShindenEntryState(entryId: number) {
    touch(shindenCache, entryId);
    return shindenEntryStates[entryId] ?? idleEntryState<ShindenEntry>();
  }

  function getReadyShindenEntry(entryId: number) {
    const state = getShindenEntryState(entryId);
    return state.status === "ready" ? state.entry : null;
  }

  function getDatabaseEntryState(entryId: number | null) {
    if (entryId === null) {
      return idleEntryState<DatabaseEntry>();
    }

    touch(databaseCache, entryId);
    return databaseEntryStates[entryId] ?? idleEntryState<DatabaseEntry>();
  }

  function getReadyDatabaseEntry(entryId: number | null) {
    const state = getDatabaseEntryState(entryId);
    return state.status === "ready" ? state.entry : null;
  }

  function retainShindenEntry(entryId: number) {
    retain(shindenCache, entryId);
    requestShindenEntries([entryId]);

    return () => {
      release(shindenCache, entryId, "shinden");
    };
  }

  function pinDatabaseEntry(entryId: number | null) {
    if (entryId === null) {
      return () => {};
    }

    const cacheEntry = databaseCache.get(entryId);
    if (cacheEntry !== undefined) {
      cacheEntry.pinned = true;
      cacheEntry.lastAccess = nextTick();
    }
    pinnedDatabaseIds.add(entryId);

    const state =
      databaseEntryStates[entryId] ?? idleEntryState<DatabaseEntry>();
    if (state.status === "idle") {
      requestDatabaseEntries([entryId]);
    }

    return () => {
      const latest = databaseCache.get(entryId);
      if (latest !== undefined) {
        latest.pinned = false;
      }
      pinnedDatabaseIds.delete(entryId);
      evictReleased("database");
    };
  }

  function requestShindenEntries(entryIds: number[]) {
    for (const entryId of entryIds) {
      if (shindenCache.has(entryId) || inFlightShindenIds.has(entryId)) {
        continue;
      }

      setShindenEntryState(entryId, { status: "loading" });
      pendingShindenIds.add(entryId);
    }

    scheduleShindenBatch();
  }

  function requestDatabaseEntries(entryIds: number[]) {
    for (const entryId of entryIds) {
      if (databaseCache.has(entryId) || inFlightDatabaseIds.has(entryId)) {
        continue;
      }

      setDatabaseEntryState(entryId, { status: "loading" });
      pendingDatabaseIds.add(entryId);
    }

    scheduleDatabaseBatch();
  }

  async function ensureReadyShindenEntry(entryId: number) {
    const readyEntry = getReadyShindenEntry(entryId);
    if (readyEntry !== null) {
      return readyEntry;
    }

    const requestGeneration = generation;
    setShindenEntryState(entryId, { status: "loading" });

    try {
      const entries = await getLoadedShindenEntries([entryId]);
      if (requestGeneration !== generation) {
        return null;
      }

      const entry = entries.find((value) => value.id === entryId) ?? null;
      if (entry === null) {
        setShindenEntryState(entryId, { status: "missing" });
        return null;
      }

      setCacheEntry(shindenCache, entry.id, entry);
      publishShindenEntries();
      evictReleased("shinden");
      return entry;
    } catch (error) {
      if (requestGeneration === generation) {
        setShindenEntryState(entryId, {
          status: "error",
          message: errorMessage(error),
        });
      }
      return null;
    }
  }

  async function ensureReadyDatabaseEntry(entryId: number) {
    const readyEntry = getReadyDatabaseEntry(entryId);
    if (readyEntry !== null) {
      return readyEntry;
    }

    // Use one bounded backend call instead of waiting on the batched loader.
    // A generation change means the caller selected from stale workspace state.
    const requestGeneration = generation;
    setDatabaseEntryState(entryId, { status: "loading" });

    try {
      const entries = await getAnimeDatabaseEntries([entryId]);
      if (requestGeneration !== generation) {
        return null;
      }

      const entry = entries.find((value) => value.id === entryId) ?? null;
      if (entry === null) {
        setDatabaseEntryState(entryId, { status: "missing" });
        return null;
      }

      setCacheEntry(databaseCache, entry.id, entry);
      publishDatabaseEntries();
      evictReleased("database");
      return entry;
    } catch (error) {
      if (requestGeneration === generation) {
        setDatabaseEntryState(entryId, {
          status: "error",
          message: errorMessage(error),
        });
      }
      return null;
    }
  }

  function scheduleShindenBatch() {
    if (shindenBatchScheduled || pendingShindenIds.size === 0) {
      return;
    }

    shindenBatchScheduled = true;
    queueMicrotask(loadPendingShindenEntries);
  }

  function scheduleDatabaseBatch() {
    if (databaseBatchScheduled || pendingDatabaseIds.size === 0) {
      return;
    }

    databaseBatchScheduled = true;
    queueMicrotask(loadPendingDatabaseEntries);
  }

  async function loadPendingShindenEntries() {
    shindenBatchScheduled = false;

    const ids = drainPendingIds(pendingShindenIds, inFlightShindenIds);
    if (ids.length === 0) {
      return;
    }
    const requestGeneration = generation;

    try {
      const entries = await getLoadedShindenEntries(ids);
      if (requestGeneration !== generation) {
        return;
      }
      const loadedIds = new Set<number>();
      for (const entry of entries) {
        loadedIds.add(entry.id);
        setCacheEntry(shindenCache, entry.id, entry);
      }
      for (const id of ids) {
        if (!loadedIds.has(id)) {
          setShindenEntryState(id, { status: "missing" });
        }
      }
      publishShindenEntries();
      evictReleased("shinden");
    } catch (error) {
      if (requestGeneration === generation) {
        for (const id of ids) {
          setShindenEntryState(id, {
            status: "error",
            message: errorMessage(error),
          });
        }
      }
    } finally {
      for (const id of ids) {
        inFlightShindenIds.delete(id);
      }
      scheduleShindenBatch();
    }
  }

  async function loadPendingDatabaseEntries() {
    databaseBatchScheduled = false;

    const ids = drainPendingIds(pendingDatabaseIds, inFlightDatabaseIds);
    if (ids.length === 0) {
      return;
    }
    const requestGeneration = generation;

    try {
      const entries = await getAnimeDatabaseEntries(ids);
      if (requestGeneration !== generation) {
        return;
      }
      const loadedIds = new Set<number>();
      for (const entry of entries) {
        loadedIds.add(entry.id);
        setCacheEntry(databaseCache, entry.id, entry);
      }
      for (const id of ids) {
        if (!loadedIds.has(id)) {
          setDatabaseEntryState(id, { status: "missing" });
        }
      }
      publishDatabaseEntries();
      evictReleased("database");
    } catch (error) {
      if (requestGeneration === generation) {
        for (const id of ids) {
          setDatabaseEntryState(id, {
            status: "error",
            message: errorMessage(error),
          });
        }
      }
    } finally {
      for (const id of ids) {
        inFlightDatabaseIds.delete(id);
      }
      scheduleDatabaseBatch();
    }
  }

  function drainPendingIds(pending: Set<number>, inFlight: Set<number>) {
    const ids = [...pending].filter((id) => !inFlight.has(id));
    pending.clear();

    for (const id of ids) {
      inFlight.add(id);
    }

    return ids;
  }

  function retain<T>(cache: Map<number, CacheEntry<T>>, entryId: number) {
    if (cache === shindenCache) {
      shindenRetainCounts.set(
        entryId,
        (shindenRetainCounts.get(entryId) ?? 0) + 1,
      );
    }

    const cacheEntry = cache.get(entryId);
    if (cacheEntry !== undefined) {
      cacheEntry.retainCount += 1;
      cacheEntry.lastAccess = nextTick();
    }
  }

  function release<T>(
    cache: Map<number, CacheEntry<T>>,
    entryId: number,
    kind: CacheKind,
  ) {
    if (cache === shindenCache) {
      const retainCount = Math.max(
        0,
        (shindenRetainCounts.get(entryId) ?? 0) - 1,
      );
      if (retainCount === 0) {
        shindenRetainCounts.delete(entryId);
      } else {
        shindenRetainCounts.set(entryId, retainCount);
      }
    }

    const cacheEntry = cache.get(entryId);
    if (cacheEntry !== undefined) {
      cacheEntry.retainCount = Math.max(0, cacheEntry.retainCount - 1);
      cacheEntry.lastAccess = nextTick();
    }

    evictReleased(kind);
  }

  function setCacheEntry<T>(
    cache: Map<number, CacheEntry<T>>,
    entryId: number,
    value: T,
  ) {
    const existing = cache.get(entryId);
    cache.set(entryId, {
      value,
      retainCount:
        existing?.retainCount ??
        (cache === shindenCache ? (shindenRetainCounts.get(entryId) ?? 0) : 0),
      pinned:
        existing?.pinned ??
        (cache === databaseCache ? pinnedDatabaseIds.has(entryId) : false),
      lastAccess: nextTick(),
    });

    if (cache === shindenCache) {
      setShindenEntryState(entryId, {
        status: "ready",
        entry: value as ShindenEntry,
      });
    } else {
      setDatabaseEntryState(entryId, {
        status: "ready",
        entry: value as DatabaseEntry,
      });
    }
  }

  function touch<T>(cache: Map<number, CacheEntry<T>>, entryId: number) {
    const cacheEntry = cache.get(entryId);
    if (cacheEntry !== undefined) {
      cacheEntry.lastAccess = nextTick();
    }
  }

  function evictReleased(kind: CacheKind) {
    const cache = kind === "shinden" ? shindenCache : databaseCache;
    const capacity =
      kind === "shinden" ? shindenReleasedCapacity : databaseReleasedCapacity;
    const releasedEntries = [...cache.entries()]
      .filter(([, entry]) => entry.retainCount === 0 && !entry.pinned)
      .sort((left, right) => left[1].lastAccess - right[1].lastAccess);
    const evictCount = releasedEntries.length - capacity;

    if (evictCount <= 0) {
      return;
    }

    for (const [entryId] of releasedEntries.slice(0, evictCount)) {
      cache.delete(entryId);
      if (kind === "shinden") {
        removeShindenEntryState(entryId);
      } else {
        removeDatabaseEntryState(entryId);
      }
    }

    if (kind === "shinden") {
      publishShindenEntries();
    } else {
      publishDatabaseEntries();
    }
  }

  function publishShindenEntries() {
    shindenEntries = Object.fromEntries(
      [...shindenCache.entries()].map(([entryId, entry]) => [
        entryId,
        entry.value,
      ]),
    );
  }

  function publishDatabaseEntries() {
    databaseEntries = Object.fromEntries(
      [...databaseCache.entries()].map(([entryId, entry]) => [
        entryId,
        entry.value,
      ]),
    );
  }

  function nextTick() {
    tick += 1;
    return tick;
  }

  function setShindenEntryState(
    entryId: number,
    state: EntryLoadState<ShindenEntry>,
  ) {
    shindenEntryStates = {
      ...shindenEntryStates,
      [entryId]: state,
    };
  }

  function setDatabaseEntryState(
    entryId: number,
    state: EntryLoadState<DatabaseEntry>,
  ) {
    databaseEntryStates = {
      ...databaseEntryStates,
      [entryId]: state,
    };
  }

  function removeShindenEntryState(entryId: number) {
    const { [entryId]: _removed, ...nextStates } = shindenEntryStates;
    shindenEntryStates = nextStates;
  }

  function removeDatabaseEntryState(entryId: number) {
    const { [entryId]: _removed, ...nextStates } = databaseEntryStates;
    databaseEntryStates = nextStates;
  }

  return {
    get shindenEntries() {
      return shindenEntries;
    },
    get databaseEntries() {
      return databaseEntries;
    },
    get shindenEntryStates() {
      return shindenEntryStates;
    },
    get databaseEntryStates() {
      return databaseEntryStates;
    },
    reset,
    getShindenEntryState,
    getReadyShindenEntry,
    getDatabaseEntryState,
    getReadyDatabaseEntry,
    ensureReadyShindenEntry,
    ensureReadyDatabaseEntry,
    retainShindenEntry,
    pinDatabaseEntry,
    requestShindenEntries,
    requestDatabaseEntries,
  };
}

function idleEntryState<T>(): EntryLoadState<T> {
  return { status: "idle" };
}

function errorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === "string") {
    return error;
  }

  return "Nie udało się wczytać danych wpisu";
}
