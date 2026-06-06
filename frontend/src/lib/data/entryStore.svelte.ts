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

type CacheKind = "shinden" | "database";

const shindenReleasedCapacity = 200;
const databaseReleasedCapacity = 50;

export type EntryStore = ReturnType<typeof createEntryStore>;

export function createEntryStore() {
  let shindenEntries = $state<Record<number, ShindenEntry>>({});
  let databaseEntries = $state<Record<number, DatabaseEntry>>({});
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
  }

  function getShindenEntry(entryId: number) {
    touch(shindenCache, entryId);
    return shindenEntries[entryId] ?? null;
  }

  function getDatabaseEntry(entryId: number | null) {
    if (entryId === null) {
      return null;
    }

    touch(databaseCache, entryId);
    return databaseEntries[entryId] ?? null;
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

    requestDatabaseEntries([entryId]);

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

      pendingShindenIds.add(entryId);
    }

    scheduleShindenBatch();
  }

  function requestDatabaseEntries(entryIds: number[]) {
    for (const entryId of entryIds) {
      if (databaseCache.has(entryId) || inFlightDatabaseIds.has(entryId)) {
        continue;
      }

      pendingDatabaseIds.add(entryId);
    }

    scheduleDatabaseBatch();
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
      for (const entry of entries) {
        setCacheEntry(shindenCache, entry.id, entry);
      }
      publishShindenEntries();
      evictReleased("shinden");
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
      for (const entry of entries) {
        setCacheEntry(databaseCache, entry.id, entry);
      }
      publishDatabaseEntries();
      evictReleased("database");
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

  return {
    get shindenEntries() {
      return shindenEntries;
    },
    get databaseEntries() {
      return databaseEntries;
    },
    reset,
    getShindenEntry,
    getDatabaseEntry,
    retainShindenEntry,
    pinDatabaseEntry,
    requestShindenEntries,
    requestDatabaseEntries,
  };
}
