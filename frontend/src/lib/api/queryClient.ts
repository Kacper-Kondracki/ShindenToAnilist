import { QueryClient } from '@tanstack/query-core';

const tightLocalCacheMs = 15_000;

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 0,
      gcTime: tightLocalCacheMs,
      retry: false,
      refetchOnMount: true,
      refetchOnReconnect: false,
      refetchOnWindowFocus: false
    }
  }
});

export const queryKeys = {
  shinden: ['shinden'] as const,
  database: ['database'] as const,
  match: ['match'] as const,
  search: ['search'] as const,
  shindenEntry: (entryId: number) =>
    [...queryKeys.shinden, 'entry', entryId] as const,
  databaseEntry: (entryId: number) =>
    [...queryKeys.database, 'entry', entryId] as const,
  shindenIds: () => [...queryKeys.shinden, 'ids'] as const,
  matchList: () => [...queryKeys.match, 'list'] as const
};

export function clearShindenQueries() {
  queryClient.removeQueries({ queryKey: queryKeys.shinden });
}

export function clearDatabaseQueries() {
  queryClient.removeQueries({ queryKey: queryKeys.database });
  queryClient.removeQueries({ queryKey: queryKeys.search });
}

export function clearMatchQueries() {
  queryClient.removeQueries({ queryKey: queryKeys.match });
}
