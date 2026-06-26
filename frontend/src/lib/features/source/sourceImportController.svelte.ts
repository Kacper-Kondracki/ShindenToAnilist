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
  ShindenListViews,
  UserListRequestState,
  WireNumber
} from '../../domain/anime';
import { wireNumberEquals } from '../../domain/anime';
import { SourceFetchPhase } from '../../gen/shinden_to_anilist/v1/source_pb';
import { errorMessage } from '../database/initializeDatabase';
import type { NotificationController } from '../notifications/notificationController.svelte';
import type { WorkspaceController } from '../workspace/workspaceController.svelte';
import {
  parseSourceUser,
  sourceImportProgressDebounceMs,
  sourceProviderBehaviorById,
  shouldDebounceSourceImportProgress,
  type ParsedSourceUser
} from './providers';

export type SourceImportBlockedRequest = {
  provider: Provider;
  query: string;
  sourceUser: ParsedSourceUser;
  message: string;
};

export type SourceImportProviderHooks = {
  beforeFetch?: (request: {
    provider: Provider;
    query: string;
    sourceUser: ParsedSourceUser;
  }) => Promise<void> | void;
  canHandleError?: (error: unknown) => boolean;
  handleBlockedImport?: (
    request: SourceImportBlockedRequest
  ) => Promise<void> | void;
  retryBlockedMessage?: string;
};

type SourceImportControllerInput = {
  animeData: LoadedAnimeData;
  workspace: WorkspaceController;
  notifications: NotificationController;
  waitForReadyDatabase: () => Promise<DatabaseState>;
  providerHooks?: Partial<Record<Provider, SourceImportProviderHooks>>;
};

export type SourceImportController = ReturnType<
  typeof createSourceImportController
>;

export function createSourceImportController({
  animeData,
  workspace,
  notifications,
  waitForReadyDatabase,
  providerHooks = {}
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

  function setError(provider: Provider, query: string, message: string) {
    state = {
      status: 'error',
      provider,
      query,
      message
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
      const message = `Nie udało się rozpoznać użytkownika ${providerById(provider).label}`;
      state = {
        status: 'error',
        provider,
        query,
        message
      };
      notifications.error('Nieprawidłowy użytkownik', message);
      return;
    }

    await submitParsedSourceUser(provider, query, sourceUser);
  }

  async function loadParsedSourceUser(
    provider: Provider,
    query: string,
    sourceUser: ParsedSourceUser,
    options: { throwErrors?: boolean; handleProviderErrors?: boolean } = {}
  ) {
    const throwErrors = options.throwErrors ?? false;
    const handleProviderErrors = options.handleProviderErrors ?? true;
    const providerHooksForRequest = providerHooks[provider];
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

    try {
      await providerHooksForRequest?.beforeFetch?.({
        provider,
        query,
        sourceUser
      });
      if (activeRequestId !== requestId) {
        return false;
      }

      const fetchStartedAt = performance.now();
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
        return false;
      }

      const readyDatabaseState = await waitForReadyDatabase();

      if (readyDatabaseState.status !== 'ready') {
        throw new Error(
          readyDatabaseState.status === 'error'
            ? readyDatabaseState.message
            : 'Baza danych nie jest gotowa'
        );
      }

      if (activeRequestId !== requestId) {
        return false;
      }

      const sourceFull = await getSourceFull();
      const nextFetchDurationMs = performance.now() - fetchStartedAt;

      if (activeRequestId !== requestId) {
        return false;
      }

      const matchStartedAt = performance.now();
      const nextMatchResult = await matchSourceList();
      const allIds = await getSourceIds();
      const nextMatchDurationMs = performance.now() - matchStartedAt;

      if (activeRequestId !== requestId) {
        return false;
      }

      assertConsistentWorkspaceVersions(
        fetchedList.sourceVersion,
        sourceFull.sourceVersion,
        readyDatabaseState.info.databaseVersion,
        nextMatchResult,
        allIds.sourceVersion ?? allIds.shindenVersion
      );

      const entryIdsByView = buildEntryIdsByView(
        nextMatchResult,
        allIds.entryIds
      );

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
      return true;
    } catch (error) {
      if (activeRequestId !== requestId) {
        return false;
      }

      console.error('Unable to load source user list', error);
      const message = errorMessage(error);
      if (
        handleProviderErrors &&
        providerHooksForRequest?.canHandleError?.(error) === true
      ) {
        state = { status: 'idle' };
        await providerHooksForRequest.handleBlockedImport?.({
          provider,
          query,
          sourceUser,
          message
        });
        return false;
      }

      resetProgressDebounce();
      if (throwErrors) {
        throw providerHooksForRequest?.retryBlockedMessage === undefined
          ? error
          : new Error(providerHooksForRequest.retryBlockedMessage);
      }

      state = {
        status: 'error',
        provider,
        query,
        message
      };
      notifications.error(
        `Nie udało się wczytać listy z ${providerById(provider).label}`,
        message
      );
      return false;
    } finally {
      if (activeRequestId === requestId) {
        activeAbortController = null;
      }
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
      console.warn('Unable to cancel source list fetch', error);
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
    throw new Error(
      'Lista źródłowa zmieniła się podczas dopasowywania. Spróbuj ponownie.'
    );
  }

  if (!wireNumberEquals(readyDatabaseVersion, matchResult.databaseVersion)) {
    throw new Error(
      'Baza danych zmieniła się podczas dopasowywania. Spróbuj ponownie.'
    );
  }
}

function buildEntryIdsByView(
  matchResult: MatchListResult,
  allEntryIds: WireNumber[]
): ShindenListViews {
  const unmatched = new Set<WireNumber>();
  const review = new Set<WireNumber>();
  const automatic = new Set<WireNumber>();

  for (const entry of matchResult.entries) {
    if (entry.result.winner !== null) {
      automatic.add(entry.shindenId);
    } else if (entry.result.top.length > 0) {
      review.add(entry.shindenId);
    } else {
      unmatched.add(entry.shindenId);
    }
  }

  const unmatchedIds = allEntryIds.filter((entryId) => unmatched.has(entryId));
  const reviewIds = allEntryIds.filter((entryId) => review.has(entryId));
  const automaticIds = allEntryIds.filter((entryId) => automatic.has(entryId));

  return {
    manual: [...unmatchedIds, ...reviewIds],
    automatic: automaticIds,
    ignored: [],
    all: [...unmatchedIds, ...reviewIds, ...automaticIds]
  };
}
