import {
  AnimeListSortedBy,
  SourceProvider
} from '../gen/shinden_to_anilist/v1/common_pb';
import type {
  MatchSelection,
  SearchOptions,
  ShindenCloudflareClearance,
  WireNumber
} from '../domain/anime';
import * as protobufService from './protobufService';
import {
  toDatabaseInfo,
  type AppDatabaseReleaseInfo,
  type SourceFetchProgress
} from './mapping';
import {
  resolveAppPaths,
  isTauriRuntime,
  openShindenCloudflareVerification
} from './runtime';
import * as tauriService from './tauriService';
import { currentVersions } from './versions';

export { currentVersions };
export type { SourceFetchProgress };

const backend = isTauriRuntime() ? tauriService : protobufService;

export async function ensureDatabase() {
  const { database: path } = await resolveAppPaths();
  let release: AppDatabaseReleaseInfo | null;
  let needsUpdate: boolean;

  try {
    const update = await checkDatabaseUpdate(path);
    needsUpdate = update.needsUpdate;
    release = update.local ?? update.remote ?? null;
    if (needsUpdate || release === null) {
      release = await downloadDatabase(path);
    }
  } catch (error) {
    try {
      const info = await loadDatabaseInfo(path, null, false);
      return { info, updateFailure: { error } };
    } catch {
      throw error;
    }
  }

  const info = await loadDatabaseInfo(path, release, needsUpdate);
  return { info, updateFailure: null };
}

async function loadDatabaseInfo(
  path: string,
  release: AppDatabaseReleaseInfo | null,
  needsUpdate: boolean
) {
  const metadata = await getDatabaseMetadata(path);
  const { databaseVersion } = await loadDatabase(path);

  return toDatabaseInfo({
    path,
    release,
    metadata,
    needsUpdate,
    databaseVersion
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

export { openShindenCloudflareVerification };

export function openShindenCloudflareAutoCloseTest() {
  return openShindenCloudflareVerification({ mode: 'autocloseTest' });
}

export function setShindenCloudflareClearance(
  clearance: ShindenCloudflareClearance
) {
  return backend.setShindenCloudflareClearance(clearance);
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
  return backend.checkDatabaseUpdate(
    path ?? (await resolveAppPaths()).database
  );
}

export async function downloadDatabase(path?: string) {
  return backend.downloadDatabase(path ?? (await resolveAppPaths()).database);
}

export async function loadDatabase(path?: string) {
  return backend.loadDatabase(path ?? (await resolveAppPaths()).database);
}

export async function getDatabaseMetadata(path?: string) {
  return backend.getDatabaseMetadata(
    path ?? (await resolveAppPaths()).database
  );
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
  return backend.exportXml(matches, path ?? (await resolveAppPaths()).export);
}
