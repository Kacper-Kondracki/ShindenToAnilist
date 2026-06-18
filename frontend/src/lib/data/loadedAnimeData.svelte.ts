import type { DatabaseEntry, SourceEntry, WireNumber } from '../domain/anime';

type DatabaseFullLoad = {
  databaseVersion: WireNumber;
  entries: DatabaseEntry[];
};

type ShindenFullLoad = {
  sourceVersion: WireNumber;
  entries: SourceEntry[];
};

export type LoadedAnimeData = ReturnType<typeof createLoadedAnimeData>;

export function createLoadedAnimeData() {
  let databaseVersion = $state<WireNumber>(0n);
  let sourceVersion = $state<WireNumber>(0n);
  let databaseEntriesById = $state<Map<WireNumber, DatabaseEntry>>(new Map());
  let sourceEntriesById = $state<Map<WireNumber, SourceEntry>>(new Map());

  function replaceDatabaseFull(load: DatabaseFullLoad) {
    databaseEntriesById = entriesById(load.entries);
    databaseVersion = load.databaseVersion;
  }

  function replaceSourceFull(load: ShindenFullLoad) {
    sourceEntriesById = entriesById(load.entries);
    sourceVersion = load.sourceVersion;
  }

  function replaceShindenFull(load: {
    shindenVersion: WireNumber;
    entries: SourceEntry[];
  }) {
    replaceSourceFull({
      sourceVersion: load.shindenVersion,
      entries: load.entries
    });
  }

  function getDatabaseEntry(entryId: WireNumber | null) {
    if (entryId === null) {
      return null;
    }

    return databaseEntriesById.get(entryId) ?? null;
  }

  function getShindenEntry(entryId: WireNumber | null) {
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

function entriesById<T extends { id: WireNumber }>(entries: T[]) {
  return new Map(entries.map((entry) => [entry.id, entry]));
}
