import {
  QueryObserver,
  type QueryCacheNotifyEvent,
  type QueryKey
} from '@tanstack/query-core';

import {
  getDatabaseEntries,
  getShindenEntries
} from '../api/appService';
import { queryClient, queryKeys } from '../api/queryClient';
import type { DatabaseEntry, ShindenEntry } from '../domain/anime';

export type EntryLoadState<T> =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'ready'; entry: T }
  | { status: 'missing' }
  | { status: 'error'; message: string };

const idleShindenEntryState: EntryLoadState<ShindenEntry> = { status: 'idle' };
const idleDatabaseEntryState: EntryLoadState<DatabaseEntry> = {
  status: 'idle'
};

export type EntryStore = ReturnType<typeof createEntryStore>;

export function createEntryStore() {
  let revision = $state(0);
  let isRevisionUpdateQueued = false;
  let isDestroyed = false;

  const unsubscribe = queryClient.getQueryCache().subscribe((event) => {
    if (isEntryStateChange(event)) {
      queueRevisionUpdate();
    }
  });

  function reset() {
    queryClient.removeQueries({ queryKey: queryKeys.shinden });
    queryClient.removeQueries({ queryKey: queryKeys.database });
    revision += 1;
  }

  function getShindenEntryState(entryId: number) {
    revision;
    return getEntryState(
      queryKeys.shindenEntry(entryId),
      idleShindenEntryState
    );
  }

  function getReadyShindenEntry(entryId: number) {
    const state = getShindenEntryState(entryId);
    return state.status === 'ready' ? state.entry : null;
  }

  function getDatabaseEntryState(entryId: number | null) {
    revision;
    if (entryId === null) {
      return idleDatabaseEntryState;
    }

    return getEntryState(
      queryKeys.databaseEntry(entryId),
      idleDatabaseEntryState
    );
  }

  function getReadyDatabaseEntry(entryId: number | null) {
    const state = getDatabaseEntryState(entryId);
    return state.status === 'ready' ? state.entry : null;
  }

  async function ensureShindenEntry(entryId: number) {
    return await queryClient.fetchQuery(shindenEntryQuery(entryId));
  }

  async function ensureDatabaseEntry(entryId: number) {
    return await queryClient.fetchQuery(databaseEntryQuery(entryId));
  }

  function prefetchShindenEntry(entryId: number) {
    void queryClient.prefetchQuery(shindenEntryQuery(entryId));
  }

  function prefetchDatabaseEntry(entryId: number | null) {
    if (entryId === null) {
      return;
    }

    void queryClient.prefetchQuery(databaseEntryQuery(entryId));
  }

  function retainShindenEntry(entryId: number) {
    return retainEntry(shindenEntryQuery(entryId));
  }

  function pinDatabaseEntry(entryId: number | null) {
    if (entryId === null) {
      return () => {};
    }

    return retainEntry(databaseEntryQuery(entryId));
  }

  function requestShindenEntries(entryIds: number[]) {
    for (const entryId of entryIds) {
      prefetchShindenEntry(entryId);
    }
  }

  function requestDatabaseEntries(entryIds: number[]) {
    for (const entryId of entryIds) {
      prefetchDatabaseEntry(entryId);
    }
  }

  function destroy() {
    isDestroyed = true;
    unsubscribe();
  }

  function queueRevisionUpdate() {
    if (isRevisionUpdateQueued) {
      return;
    }

    isRevisionUpdateQueued = true;
    queueMicrotask(() => {
      isRevisionUpdateQueued = false;
      if (!isDestroyed) {
        revision += 1;
      }
    });
  }

  return {
    get shindenEntryStates() {
      revision;
      return {};
    },
    get databaseEntryStates() {
      revision;
      return {};
    },
    reset,
    destroy,
    getShindenEntryState,
    getReadyShindenEntry,
    getDatabaseEntryState,
    getReadyDatabaseEntry,
    ensureShindenEntry,
    ensureDatabaseEntry,
    ensureReadyShindenEntry: ensureShindenEntry,
    ensureReadyDatabaseEntry: ensureDatabaseEntry,
    retainShindenEntry,
    pinDatabaseEntry,
    requestShindenEntries,
    requestDatabaseEntries
  };
}

function shindenEntryQuery(entryId: number) {
  return {
    queryKey: queryKeys.shindenEntry(entryId),
    queryFn: () => loadShindenEntry(entryId)
  };
}

function databaseEntryQuery(entryId: number) {
  return {
    queryKey: queryKeys.databaseEntry(entryId),
    queryFn: () => loadDatabaseEntry(entryId)
  };
}

function retainEntry<T>(options: {
  queryKey: QueryKey;
  queryFn: () => Promise<T | null>;
}) {
  const observer = new QueryObserver<T | null>(queryClient, {
    ...options,
    refetchOnMount: false,
    staleTime: Infinity
  });

  return observer.subscribe(() => {});
}

function isEntryStateChange(event: QueryCacheNotifyEvent) {
  return (
    event.type === 'added' ||
    event.type === 'removed' ||
    event.type === 'updated'
  );
}

function getEntryState<T>(
  queryKey: readonly unknown[],
  idleState: EntryLoadState<T>
): EntryLoadState<T> {
  const queryState = queryClient.getQueryState<T | null>(queryKey);

  if (queryState === undefined) {
    return idleState;
  }

  if (queryState.status === 'error') {
    return {
      status: 'error',
      message: errorMessage(queryState.error)
    };
  }

  if (queryState.status === 'success') {
    const data = queryState.data;
    if (data === null || data === undefined) {
      return { status: 'missing' };
    }

    return {
      status: 'ready',
      entry: data
    };
  }

  return queryState.fetchStatus === 'idle' ? idleState : { status: 'loading' };
}

async function loadShindenEntry(entryId: number) {
  const entries = await getShindenEntries([entryId]);
  return entries.find((entry) => entry.id === entryId) ?? null;
}

async function loadDatabaseEntry(entryId: number) {
  const entries = await getDatabaseEntries([entryId]);
  return entries.find((entry) => entry.id === entryId) ?? null;
}

function errorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === 'string') {
    return error;
  }

  return 'Nie udało się wczytać danych wpisu';
}
