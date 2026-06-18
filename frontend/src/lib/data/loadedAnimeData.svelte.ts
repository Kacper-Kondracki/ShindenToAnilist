import type { DatabaseEntry, SourceEntry } from '../domain/anime';

type DatabaseFullLoad = {
  databaseVersion: number;
  entries: DatabaseEntry[];
};

type ShindenFullLoad = {
  sourceVersion: number;
  entries: SourceEntry[];
};

export type LoadedAnimeData = ReturnType<typeof createLoadedAnimeData>;

export function createLoadedAnimeData() {
  let databaseVersion = $state(0);
  let sourceVersion = $state(0);
  let databaseEntriesById = $state<Map<number, DatabaseEntry>>(new Map());
  let sourceEntriesById = $state<Map<number, SourceEntry>>(new Map());

  function replaceDatabaseFull(load: DatabaseFullLoad) {
    databaseEntriesById = entriesById(load.entries);
    databaseVersion = load.databaseVersion;
  }

  function replaceSourceFull(load: ShindenFullLoad) {
    sourceEntriesById = entriesById(load.entries);
    sourceVersion = load.sourceVersion;
  }

  function replaceShindenFull(load: { shindenVersion: number; entries: SourceEntry[] }) {
    replaceSourceFull({
      sourceVersion: load.shindenVersion,
      entries: load.entries
    });
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

    return sourceEntriesById.get(entryId) ?? null;
  }

  return {
    get databaseVersion() {
      return databaseVersion;
    },
    get sourceVersion() {
      return sourceVersion;
    },
    get shindenVersion() {
      return sourceVersion;
    },
    get databaseEntryCount() {
      return databaseEntriesById.size;
    },
    get shindenEntryCount() {
      return sourceEntriesById.size;
    },
    replaceDatabaseFull,
    replaceSourceFull,
    replaceShindenFull,
    getDatabaseEntry,
    getShindenEntry
  };
}

function entriesById<T extends { id: number }>(entries: T[]) {
  return new Map(entries.map((entry) => [entry.id, entry]));
}
