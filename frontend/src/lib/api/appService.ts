import {
  AnimeListSortedBy,
  SourceProvider
} from '../gen/shinden_to_anilist/v1/common_pb';
import type {
  MatchSelection,
  SearchOptions,
  WireNumber
} from '../domain/anime';
import * as grpcService from './grpcService';
import { toDatabaseInfo, type SourceFetchProgress } from './mapping';
import {
  appPathsPromise,
  databasePath,
  exportPath,
  isTauriRuntime
} from './runtime';
import * as tauriService from './tauriService';
import { currentVersions } from './versions';

export { currentVersions, databasePath, exportPath };
export type { SourceFetchProgress };

const backend = isTauriRuntime() ? tauriService : grpcService;

export async function ensureDatabase() {
  const paths = await appPathsPromise;
  const update = await checkDatabaseUpdate(paths.database);

  let release = update.local ?? update.remote ?? null;
  if (update.needsUpdate || release === null) {
    release = await downloadDatabase(paths.database);
  }

  const [{ databaseVersion: loadedVersion }, metadata] = await Promise.all([
    loadDatabase(paths.database),
    getDatabaseMetadata(paths.database)
  ]);

  return toDatabaseInfo({
    path: paths.database,
    release,
    metadata,
    needsUpdate: update.needsUpdate,
    databaseVersion: loadedVersion
  });
}

export function fetchSourceList(
  provider: SourceProvider,
  user: string,
  onProgress?: (progress: SourceFetchProgress) => void,
  options?: { requestId?: number; signal?: AbortSignal }
) {
  return backend.fetchSourceList(provider, user, onProgress, options);
}

export function cancelSourceListFetch(requestId: number) {
  return backend.cancelSourceListFetch(requestId);
}

export function fetchShindenList(userId: number) {
  return backend.fetchShindenList(userId);
}

export function getSourceIds(sortedBy = AnimeListSortedBy.URGENCY) {
  return backend.getSourceIds(sortedBy);
}

export function getShindenIds(sortedBy = AnimeListSortedBy.URGENCY) {
  return backend.getShindenIds(sortedBy);
}

export function getSourceFull() {
  return backend.getSourceFull();
}

export function getShindenFull() {
  return backend.getShindenFull();
}

export async function checkDatabaseUpdate(path?: string) {
  return backend.checkDatabaseUpdate(path ?? (await appPathsPromise).database);
}

export async function downloadDatabase(path?: string) {
  return backend.downloadDatabase(path ?? (await appPathsPromise).database);
}

export async function loadDatabase(path?: string) {
  return backend.loadDatabase(path ?? (await appPathsPromise).database);
}

export async function getDatabaseMetadata(path?: string) {
  return backend.getDatabaseMetadata(path ?? (await appPathsPromise).database);
}

export function getDatabaseFull() {
  return backend.getDatabaseFull();
}

export function fuzzySearch(query: string, options: SearchOptions = {}) {
  return backend.fuzzySearch(query, options);
}

export function fuzzyMatch(
  query: string,
  options: SearchOptions = {},
  sourceId?: WireNumber
) {
  return backend.fuzzyMatch(query, options, sourceId);
}

export function matchShindenList(options: SearchOptions = {}) {
  return backend.matchShindenList(options);
}

export function matchSourceList(options: SearchOptions = {}) {
  return backend.matchSourceList(options);
}

export async function exportXml(matches: MatchSelection[], path?: string) {
  return backend.exportXml(matches, path ?? (await appPathsPromise).export);
}
