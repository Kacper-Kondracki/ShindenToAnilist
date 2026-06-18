import {
  cancelSourceListFetch,
  fetchSourceList,
  getSourceFull,
  getSourceIds,
  matchSourceList
} from '../../api/appService';
import { providerById, providers, type Provider } from '../../config/providers';
import { createLoadedAnimeData } from '../../data/loadedAnimeData.svelte';
import type {
  DatabaseState,
  MatchListResult,
  ShindenListViews,
  UserListRequestState,
  WireNumber
} from '../../domain/anime';
import { wireNumberEquals } from '../../domain/anime';
import { SourceFetchPhase } from '../../gen/shinden_to_anilist/v1/source_pb';
import {
  errorMessage,
  initializeDatabaseState
} from '../database/initializeDatabase';
import {
  isSourceImportPreviewInput,
  parseSourceUser,
  providerFromInput
} from '../source/userInput';
import { createWorkspaceController } from '../workspace/workspaceController.svelte';

export type AppController = ReturnType<typeof createAppController>;

export function createAppController() {
  const animeData = createLoadedAnimeData();
  const workspace = createWorkspaceController(animeData);

  let selectedProvider = $state<Provider>('shinden');
  let userQuery = $state('');
  let databaseState = $state<DatabaseState>({ status: 'loading' });
  let userListRequestState = $state<UserListRequestState>({ status: 'idle' });
  let activeRequestId = 0;
  let activeUserListAbortController: AbortController | null = null;
  let databaseInitializationPromise: Promise<DatabaseState> | null = null;
  let sourceImportPreviewInterval: number | null = null;

  let trimmedQuery = $derived(userQuery.trim());
  let selectedProviderDetails = $derived(providerById(selectedProvider));
  let activeProviderDetails = $derived.by(() => {
    const workspaceState = workspace.state;

    if (workspaceState.status === 'active') {
      return providerById(workspaceState.provider);
    }

    return selectedProviderDetails;
  });
  let databaseStatusText = $derived.by(() => {
    if (databaseState.status === 'ready') {
      return databaseState.info.lastUpdate
        ? `Baza danych: ${databaseState.info.lastUpdate}`
        : 'Baza danych załadowana';
    }

    if (databaseState.status === 'error') {
      return 'Baza danych niedostępna';
    }

    return 'Ładowanie bazy danych';
  });
  let isUserListLoading = $derived(userListRequestState.status === 'loading');
  let userListRequestProviderDetails = $derived(
    userListRequestState.status === 'loading' ||
      userListRequestState.status === 'error'
      ? providerById(userListRequestState.provider)
      : selectedProviderDetails
  );
  let isProviderSupported = $derived(selectedProviderDetails.supportsUserList);
  let shouldShowSourceImportProgress = $derived(
    userListRequestState.status === 'loading' &&
      providerById(userListRequestState.provider).supportsSourceImportProgress
  );
  let isWaitingForDatabase = $derived(
    userListRequestState.status === 'loading' &&
      databaseState.status !== 'ready'
  );
  let isLoadButtonBusy = $derived(isUserListLoading || isWaitingForDatabase);
  let hasUserListError = $derived(userListRequestState.status === 'error');
  let userListErrorMessage = $derived(
    userListRequestState.status === 'error'
      ? userListRequestState.message
      : null
  );
  let canSubmit = $derived(
    Boolean(trimmedQuery) && !isUserListLoading && isProviderSupported
  );
  async function initializeDatabase() {
    databaseState = { status: 'loading' };
    databaseInitializationPromise = initializeDatabaseState(animeData);
    databaseState = await databaseInitializationPromise;
    return databaseState;
  }

  function setSelectedProvider(provider: Provider) {
    if (isUserListLoading) {
      return;
    }

    selectedProvider = provider;
    clearUserListError();
  }

  function setUserQuery(value: string) {
    if (isUserListLoading) {
      return;
    }

    userQuery = value;
    clearUserListError();

    const detectedProvider = providerFromInput(value);
    if (detectedProvider !== null) {
      selectedProvider = detectedProvider;
    }
  }

  function clearUserListError() {
    if (userListRequestState.status === 'error') {
      userListRequestState = { status: 'idle' };
    }
  }

  async function submitUserList() {
    stopSourceImportPreview();
    clearUserListError();

    if (!trimmedQuery || isUserListLoading || !isProviderSupported) {
      return;
    }

    const provider = selectedProvider;
    const query = trimmedQuery;

    if (isSourceImportPreviewInput(userQuery)) {
      startSourceImportPreview(query);
      return;
    }

    const sourceUser = parseSourceUser(provider, query);

    if (sourceUser === null) {
      userListRequestState = {
        status: 'error',
        provider,
        query,
        message: `Nie udało się rozpoznać użytkownika ${selectedProviderDetails.label}`
      };
      return;
    }

    const requestId = activeRequestId + 1;
    activeRequestId = requestId;
    activeUserListAbortController?.abort();
    const abortController = new AbortController();
    activeUserListAbortController = abortController;
    const startedAt = performance.now();
    userListRequestState = { status: 'loading', provider, query, progress: null };

    try {
      const fetchStartedAt = performance.now();
      const fetchedList = await fetchSourceList(
        sourceUser.provider,
        sourceUser.user,
        (progress) => {
          if (activeRequestId !== requestId) {
            return;
          }

          userListRequestState = {
            status: 'loading',
            provider,
            query,
            progress: {
              phase: progress.phase,
              current: progress.current,
              total: progress.total,
              latestTitle: progress.latestTitle,
              startedAt
            }
          };
        },
        {
          requestId,
          signal: abortController.signal
        }
      );

      if (activeRequestId !== requestId) {
        return;
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
        return;
      }

      const sourceFull = await getSourceFull();
      const nextFetchDurationMs = performance.now() - fetchStartedAt;

      if (activeRequestId !== requestId) {
        return;
      }

      const matchStartedAt = performance.now();
      const nextMatchResult = await matchSourceList();
      const allIds = await getSourceIds();
      const nextMatchDurationMs = performance.now() - matchStartedAt;

      if (activeRequestId !== requestId) {
        return;
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

      userListRequestState = {
        status: 'loaded',
        provider,
        query,
        entryIdsByView
      };
      workspace.activate({
        provider,
        query,
        entryIdsByView,
        matchResult: nextMatchResult,
        fetchDurationMs: nextFetchDurationMs,
        matchDurationMs: nextMatchDurationMs,
        manualOverrideScopeKey: sourceUser.manualOverrideScopeKey
      });
    } catch (error) {
      if (activeRequestId !== requestId) {
        return;
      }

      console.error('Unable to load source user list', error);
      userListRequestState = {
        status: 'error',
        provider,
        query,
        message: errorMessage(error)
      };
    } finally {
      if (activeRequestId === requestId) {
        activeUserListAbortController = null;
      }
    }
  }

  function cancelUserListLoad() {
    if (userListRequestState.status !== 'loading') {
      return;
    }

    if (sourceImportPreviewInterval !== null) {
      stopSourceImportPreview();
      userListRequestState = { status: 'idle' };
      return;
    }

    const requestId = activeRequestId;
    activeRequestId += 1;
    activeUserListAbortController?.abort();
    activeUserListAbortController = null;
    void cancelSourceListFetch(requestId).catch((error) => {
      console.warn('Unable to cancel source list fetch', error);
    });
    userListRequestState = { status: 'idle' };
  }

  function startSourceImportPreview(query = 'aurora-preview') {
    stopSourceImportPreview();
    activeRequestId += 1;
    activeUserListAbortController?.abort();
    activeUserListAbortController = null;

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

      userListRequestState = {
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
    sourceImportPreviewInterval = window.setInterval(
      updatePreviewProgress,
      900
    );
  }

  function stopSourceImportPreview() {
    if (sourceImportPreviewInterval === null) {
      return;
    }

    window.clearInterval(sourceImportPreviewInterval);
    sourceImportPreviewInterval = null;
  }

  async function waitForReadyDatabase() {
    if (databaseState.status === 'ready' || databaseState.status === 'error') {
      return databaseState;
    }

    if (databaseInitializationPromise === null) {
      return await initializeDatabase();
    }

    return await databaseInitializationPromise;
  }

  return {
    providers,
    animeData,
    workspace,
    get selectedProvider() {
      return selectedProvider;
    },
    get userQuery() {
      return userQuery;
    },
    get databaseState() {
      return databaseState;
    },
    get userListRequestState() {
      return userListRequestState;
    },
    get selectedProviderDetails() {
      return selectedProviderDetails;
    },
    get userListRequestProviderDetails() {
      return userListRequestProviderDetails;
    },
    get activeProviderDetails() {
      return activeProviderDetails;
    },
    get databaseStatusText() {
      return databaseStatusText;
    },
    get isLoadButtonBusy() {
      return isLoadButtonBusy;
    },
    get isUserListLoading() {
      return isUserListLoading;
    },
    get hasUserListError() {
      return hasUserListError;
    },
    get userListErrorMessage() {
      return userListErrorMessage;
    },
    get canSubmit() {
      return canSubmit;
    },
    get isProviderSupported() {
      return isProviderSupported;
    },
    get shouldShowSourceImportProgress() {
      return shouldShowSourceImportProgress;
    },
    initializeDatabase,
    setSelectedProvider,
    setUserQuery,
    clearUserListError,
    submitUserList,
    cancelUserListLoad,
    startSourceImportPreview,
    stopSourceImportPreview
  };
}

function assertConsistentWorkspaceVersions(
  fetchedSourceVersion: WireNumber,
  fullSourceVersion: WireNumber,
  readyDatabaseVersion: WireNumber,
  matchResult: MatchListResult,
  sourceIdsVersion: WireNumber
) {
  if (
    !wireNumberEquals(fetchedSourceVersion, matchResult.shindenVersion) ||
    !wireNumberEquals(fullSourceVersion, matchResult.shindenVersion) ||
    !wireNumberEquals(sourceIdsVersion, matchResult.shindenVersion)
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
