import { reportFailure } from '../../api/diagnostics';
import {
  cancelSourceListFetch,
  fetchSourceList,
  getSourceFull,
  getSourceIds,
  matchSourceList
} from '../../api/appService';
import { providerById, type Provider } from '../../config/providers';
import type { LoadedAnimeData } from '../../data/loadedAnimeData.svelte';
import type {
  DatabaseState,
  MatchListResult,
  UserListRequestState,
  WireNumber
} from '../../domain/anime';
import { wireNumberEquals } from '../../domain/anime';
import { SourceFetchPhase } from '../../gen/shinden_to_anilist/v1/source_pb';
import {
  AppFailure,
  ErrorKind,
  isFailure,
  userErrorMessage
} from '../../api/errors';
import type { NotificationController } from '../notifications/notificationController.svelte';
import type { ShindenCloudflareRequest } from '../shinden/cloudflareController.svelte';
import type { WorkspaceController } from '../workspace/workspaceController.svelte';
import { buildMatchViews } from '../workspace/matchViews';
import {
  parseSourceUser,
  sourceImportProgressDebounceMs,
  sourceProviderBehaviorById,
  shouldDebounceSourceImportProgress,
  type ParsedSourceUser
} from './providers';

type SourceImportControllerInput = {
  animeData: LoadedAnimeData;
  workspace: WorkspaceController;
  notifications: NotificationController;
  waitForReadyDatabase: () => Promise<DatabaseState>;
  applySavedShindenVerification: () => Promise<void>;
  onShindenChallenge: (request: ShindenCloudflareRequest) => void;
};

export type SourceImportController = ReturnType<
  typeof createSourceImportController
>;

export function createSourceImportController({
  animeData,
  workspace,
  notifications,
  waitForReadyDatabase,
  applySavedShindenVerification,
  onShindenChallenge
}: SourceImportControllerInput) {
  let state = $state<UserListRequestState>({ status: 'idle' });
  let activeRequestId = 0;
  let activeAbortController: AbortController | null = null;
  let previewInterval: number | null = null;
  let progressDebounceTimeout: number | null = null;
  let isProgressDebounced = $state(false);

  let isLoading = $derived(state.status === 'loading');
  let shouldShowProgress = $derived.by(() => {
    if (state.status !== 'loading' || state.progress === null) {
      return false;
    }

    const provider = providerById(state.provider);
    return (
      provider.supportsSourceImportProgress &&
      sourceProviderBehaviorById(state.provider).progress.isReady?.(
        state.progress,
        { listPhaseDebounced: isProgressDebounced }
      ) === true
    );
  });

  function clearError() {
    if (state.status === 'error') {
      state = { status: 'idle' };
    }
  }

  function setError(
    provider: Provider,
    query: string,
    message: string,
    invalidInput = false
  ) {
    state = {
      status: 'error',
      provider,
      query,
      message,
      invalidInput
    };
  }

  async function submitParsedSourceUser(
    provider: Provider,
    query: string,
    sourceUser: ParsedSourceUser
  ) {
    await loadParsedSourceUser(provider, query, sourceUser);
  }

  async function submitQuery(provider: Provider, query: string) {
    const sourceUser = parseSourceUser(provider, query);

    if (sourceUser === null) {
      const message = `Sprawdź nazwę użytkownika lub adres listy w serwisie ${providerById(provider).label}.`;
      setError(provider, query, message, true);
      notifications.error('Nieprawidłowy użytkownik', message);
      return;
    }

    await submitParsedSourceUser(provider, query, sourceUser);
  }

  async function loadParsedSourceUser(
    provider: Provider,
    query: string,
    sourceUser: ParsedSourceUser,
    options: { afterVerification?: boolean } = {}
  ): Promise<'loaded' | 'cancelled' | 'failed' | 'challenge'> {
    const requestId = activeRequestId + 1;
    activeRequestId = requestId;
    activeAbortController?.abort();
    resetProgressDebounce();
    const abortController = new AbortController();
    activeAbortController = abortController;
    const startedAt = performance.now();
    state = {
      status: 'loading',
      provider,
      query,
      progress: null
    };

    let fetchingSource = false;
    try {
      if (provider === 'shinden' && !options.afterVerification) {
        await applySavedShindenVerification();
      }
      if (activeRequestId !== requestId) {
        return 'cancelled';
      }

      const fetchStartedAt = performance.now();
      fetchingSource = true;
      const fetchedList = await fetchSourceList(
        sourceUser.provider,
        sourceUser.user,
        (progress) => {
          if (activeRequestId !== requestId) {
            return;
          }

          const nextProgress = {
            phase: progress.phase,
            current: progress.current,
            total: progress.total,
            latestTitle: progress.latestTitle,
            startedAt
          };

          state = {
            status: 'loading',
            provider,
            query,
            progress: nextProgress
          };

          if (shouldDebounceSourceImportProgress(provider, nextProgress)) {
            scheduleProgressDebounce(provider, requestId);
          }
        },
        {
          requestId,
          signal: abortController.signal
        }
      );

      if (activeRequestId !== requestId) {
        return 'cancelled';
      }

      fetchingSource = false;
      const readyDatabaseState = await waitForReadyDatabase();

      if (activeRequestId !== requestId) {
        return 'cancelled';
      }

      if (readyDatabaseState.status !== 'ready') {
        return failImport(
          readyDatabaseState.status === 'error'
            ? readyDatabaseState.message
            : userErrorMessage(
                new AppFailure(ErrorKind.DATABASE_NOT_LOADED),
                ''
              )
        );
      }

      const sourceFull = await getSourceFull();
      const nextFetchDurationMs = performance.now() - fetchStartedAt;

      if (activeRequestId !== requestId) {
        return 'cancelled';
      }

      const matchStartedAt = performance.now();
      const nextMatchResult = await matchSourceList();
      const allIds = await getSourceIds();
      const nextMatchDurationMs = performance.now() - matchStartedAt;

      if (activeRequestId !== requestId) {
        return 'cancelled';
      }

      assertConsistentWorkspaceVersions(
        fetchedList.sourceVersion,
        sourceFull.sourceVersion,
        readyDatabaseState.info.databaseVersion,
        nextMatchResult,
        allIds.sourceVersion ?? allIds.shindenVersion
      );

      const entryIdsByView = buildMatchViews(nextMatchResult, allIds.entryIds);

      animeData.replaceSourceFull(sourceFull);

      state = {
        status: 'loaded',
        provider,
        query,
        entryIdsByView
      };
      resetProgressDebounce();
      notifications.success(
        'Lista została wczytana',
        `Zaimportowano ${allIds.entryIds.length} pozycji z ${providerById(provider).label}.`
      );
      workspace.activate({
        provider,
        query,
        entryIdsByView,
        matchResult: nextMatchResult,
        fetchDurationMs: nextFetchDurationMs,
        matchDurationMs: nextMatchDurationMs,
        manualOverrideScopeKey: sourceUser.manualOverrideScopeKey
      });
      return 'loaded';
    } catch (error) {
      if (activeRequestId !== requestId) {
        return 'cancelled';
      }

      if (isFailure(error, ErrorKind.CANCELLED)) {
        resetProgressDebounce();
        state = { status: 'idle' };
        return 'cancelled';
      }

      if (
        fetchingSource &&
        provider === 'shinden' &&
        isFailure(error, ErrorKind.CLOUDFLARE_CHALLENGE)
      ) {
        resetProgressDebounce();
        state = { status: 'idle' };
        if (!options.afterVerification) {
          onShindenChallenge({
            provider,
            query,
            sourceUser,
            message: userErrorMessage(error, '')
          });
        }
        return 'challenge';
      }

      reportFailure('source_import', error);
      return failImport(
        userErrorMessage(error, 'Spróbuj wczytać listę ponownie.'),
        isFailure(error, ErrorKind.INVALID_SOURCE_USER)
      );
    } finally {
      if (activeRequestId === requestId) {
        activeAbortController = null;
      }
    }
    function failImport(message: string, invalidInput = false) {
      resetProgressDebounce();
      setError(provider, query, message, invalidInput);
      notifications.error(
        `Nie udało się wczytać listy z ${providerById(provider).label}`,
        message
      );
      return 'failed' as const;
    }
  }

  function cancelLoad() {
    if (state.status !== 'loading') {
      return;
    }

    if (previewInterval !== null) {
      stopPreview();
      state = { status: 'idle' };
      return;
    }

    const requestId = activeRequestId;
    activeRequestId += 1;
    activeAbortController?.abort();
    activeAbortController = null;
    resetProgressDebounce();
    void cancelSourceListFetch(requestId).catch((error) => {
      reportFailure('cancel_source_import', error);
    });
    state = { status: 'idle' };
  }

  function startPreview(query = 'aurora-preview') {
    stopPreview();
    activeRequestId += 1;
    activeAbortController?.abort();
    activeAbortController = null;
    resetProgressDebounce();

    const provider: Provider = 'anime-zone';
    const total = 180;
    const startedAt = performance.now();
    const previewTitles = [
      'Frieren: Beyond Journey’s End',
      'Dungeon Meshi',
      'Cyberpunk: Edgerunners',
      'Violet Evergarden',
      'Mob Psycho 100',
      'Bocchi the Rock!',
      '86 Eighty-Six',
      'Odd Taxi'
    ];

    function updatePreviewProgress() {
      const elapsedMs = performance.now() - startedAt;
      const loopMs = 28000;
      const loopProgress = (elapsedMs % loopMs) / loopMs;
      const current = Math.max(
        1,
        Math.min(total, Math.round(loopProgress * total))
      );
      const phase =
        loopProgress < 0.14
          ? SourceFetchPhase.FETCHING_LIST
          : loopProgress < 0.82
            ? SourceFetchPhase.FETCHING_DETAILS
            : loopProgress < 0.94
              ? SourceFetchPhase.STORING
              : SourceFetchPhase.DONE;
      const titleIndex =
        Math.floor(loopProgress * previewTitles.length * 2) %
        previewTitles.length;
      const latestTitle = previewTitles[titleIndex] ?? previewTitles[0] ?? '';

      state = {
        status: 'loading',
        provider,
        query,
        progress: {
          phase,
          current,
          total,
          latestTitle,
          startedAt
        }
      };
    }

    updatePreviewProgress();
    previewInterval = window.setInterval(updatePreviewProgress, 900);
  }

  function stopPreview() {
    if (previewInterval === null) {
      return;
    }

    window.clearInterval(previewInterval);
    previewInterval = null;
  }

  function scheduleProgressDebounce(provider: Provider, requestId: number) {
    if (progressDebounceTimeout !== null || isProgressDebounced) {
      return;
    }

    progressDebounceTimeout = window.setTimeout(() => {
      progressDebounceTimeout = null;

      if (
        activeRequestId === requestId &&
        state.status === 'loading' &&
        state.provider === provider
      ) {
        isProgressDebounced = true;
      }
    }, sourceImportProgressDebounceMs(provider));
  }

  function resetProgressDebounce() {
    if (progressDebounceTimeout !== null) {
      window.clearTimeout(progressDebounceTimeout);
      progressDebounceTimeout = null;
    }

    isProgressDebounced = false;
  }

  return {
    get state() {
      return state;
    },
    get isLoading() {
      return isLoading;
    },
    get shouldShowProgress() {
      return shouldShowProgress;
    },
    get isProgressDebounced() {
      return isProgressDebounced;
    },
    clearError,
    setError,
    submitQuery,
    loadParsedSourceUser,
    cancelLoad,
    startPreview,
    stopPreview
  };
}

function assertConsistentWorkspaceVersions(
  fetchedSourceVersion: WireNumber,
  fullSourceVersion: WireNumber,
  readyDatabaseVersion: WireNumber,
  matchResult: MatchListResult,
  sourceIdsVersion: WireNumber
) {
  const matchedSourceVersion =
    matchResult.sourceVersion ?? matchResult.shindenVersion;
  if (
    !wireNumberEquals(fetchedSourceVersion, matchedSourceVersion) ||
    !wireNumberEquals(fullSourceVersion, matchedSourceVersion) ||
    !wireNumberEquals(sourceIdsVersion, matchedSourceVersion)
  ) {
    throw new AppFailure(ErrorKind.SOURCE_LIST_CHANGED);
  }

  if (!wireNumberEquals(readyDatabaseVersion, matchResult.databaseVersion)) {
    throw new AppFailure(ErrorKind.DATABASE_CHANGED);
  }
}
