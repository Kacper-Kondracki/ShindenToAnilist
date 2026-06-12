import type { DatabaseEntry, ShindenEntry } from '../domain/anime';

type DatabaseFullLoad = {
  databaseVersion: number;
  entries: DatabaseEntry[];
};

type ShindenFullLoad = {
  shindenVersion: number;
  entries: ShindenEntry[];
};

export type LoadedAnimeData = ReturnType<typeof createLoadedAnimeData>;

export function createLoadedAnimeData() {
  let databaseVersion = $state(0);
  let shindenVersion = $state(0);
  let databaseEntriesById = $state<Map<number, DatabaseEntry>>(new Map());
  let shindenEntriesById = $state<Map<number, ShindenEntry>>(new Map());

  function replaceDatabaseFull(load: DatabaseFullLoad) {
    databaseEntriesById = entriesById(load.entries);
    databaseVersion = load.databaseVersion;
  }

  function replaceShindenFull(load: ShindenFullLoad) {
    shindenEntriesById = entriesById(load.entries);
    shindenVersion = load.shindenVersion;
  }

  function getDatabaseEntry(entryId: number | null) {
    if (entryId === null) {
      return null;
    }

    return databaseEntriesById.get(entryId) ?? null;
  }

  function getShindenEntry(entryId: number | null) {
    if (entryId === null) {
      return null;
    }

    return shindenEntriesById.get(entryId) ?? null;
  }

  return {
    get databaseVersion() {
      return databaseVersion;
    },
    get shindenVersion() {
      return shindenVersion;
    },
    get databaseEntryCount() {
      return databaseEntriesById.size;
    },
    get shindenEntryCount() {
      return shindenEntriesById.size;
    },
    replaceDatabaseFull,
    replaceShindenFull,
    getDatabaseEntry,
    getShindenEntry
  };
}

function entriesById<T extends { id: number }>(entries: T[]) {
  return new Map(entries.map((entry) => [entry.id, entry]));
}
