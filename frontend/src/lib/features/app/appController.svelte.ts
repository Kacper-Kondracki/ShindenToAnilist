import {
  cancelSourceListFetch,
  fetchSourceList,
  getSourceFull,
  getSourceIds,
  isShindenCloudflareChallengeError,
  matchSourceList,
  openShindenCloudflareVerification,
  setShindenCloudflareClearance
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
import { createNotificationController } from '../notifications/notificationController.svelte';
import {
  clearShindenCloudflareClearance,
  readShindenCloudflareClearance,
  writeShindenCloudflareClearance
} from '../shinden/cloudflareClearanceStorage';
import {
  isSourceImportPreviewInput,
  parseMockNotificationCount,
  parseSourceUser,
  providerFromInput,
  type ParsedSourceUser
} from '../source/userInput';
import { createShindenCloudflareController } from '../shinden/cloudflareController.svelte';
import { createWorkspaceController } from '../workspace/workspaceController.svelte';

export type AppController = ReturnType<typeof createAppController>;

export function createAppController() {
  const animeData = createLoadedAnimeData();
  const workspace = createWorkspaceController(animeData);
  const notifications = createNotificationController();
  const shindenCloudflare = createShindenCloudflareController({
    openVerification: openShindenCloudflareVerification,
    applyClearance: setShindenCloudflareClearance,
    saveClearance: writeShindenCloudflareClearance,
    retry: async (request) => {
      await loadParsedSourceUser(
        request.provider,
        request.query,
        request.sourceUser,
        { throwErrors: true, handleCloudflare: false }
      );
    },
    onResolved: () => {
      notifications.success(
        'Weryfikacja Shindena zakończona',
        'Import został wznowiony po weryfikacji Cloudflare.'
      );
    }
  });

  let selectedProvider = $state<Provider>('shinden');
  let userQuery = $state('');
  let databaseState = $state<DatabaseState>({ status: 'loading' });
  let userListRequestState = $state<UserListRequestState>({ status: 'idle' });
  let activeRequestId = 0;
  let activeUserListAbortController: AbortController | null = null;
  let databaseInitializationPromise: Promise<DatabaseState> | null = null;
  let sourceImportPreviewInterval: number | null = null;
  let lastDatabaseNotificationMessage = '';

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
      providerById(userListRequestState.provider)
        .supportsSourceImportProgress &&
      userListRequestState.progress !== null &&
      (userListRequestState.progress.total > 0 ||
        userListRequestState.progress.phase !== SourceFetchPhase.FETCHING_LIST)
  );
  let isWaitingForDatabase = $derived(
    userListRequestState.status === 'loading' &&
      databaseState.status !== 'ready'
  );
  let isLoadButtonBusy = $derived(
    isUserListLoading || isWaitingForDatabase || shindenCloudflare.isBusy
  );
  let hasUserListError = $derived(userListRequestState.status === 'error');
  let userListErrorMessage = $derived(
    userListRequestState.status === 'error'
      ? userListRequestState.message
      : null
  );
  let canSubmit = $derived(
    Boolean(trimmedQuery) &&
      !isUserListLoading &&
      !shindenCloudflare.isOpen &&
      isProviderSupported
  );
  async function initializeDatabase() {
    databaseState = { status: 'loading' };
    databaseInitializationPromise = initializeDatabaseState(animeData);
    databaseState = await databaseInitializationPromise;
    if (
      databaseState.status === 'error' &&
      databaseState.message !== lastDatabaseNotificationMessage
    ) {
      lastDatabaseNotificationMessage = databaseState.message;
      notifications.error(
        'Nie udało się wczytać bazy danych',
        databaseState.message
      );
    }

    return databaseState;
  }

  function setSelectedProvider(provider: Provider) {
    if (isUserListLoading) {
      return;
    }

    shindenCloudflare.cancel();
    selectedProvider = provider;
    clearUserListError();
  }

  function setUserQuery(value: string) {
    if (isUserListLoading) {
      return;
    }

    shindenCloudflare.cancel();
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

    const mockNotificationCount = parseMockNotificationCount(query);
    if (mockNotificationCount !== null) {
      showMockNotifications(mockNotificationCount);
      return;
    }

    const sourceUser = parseSourceUser(provider, query);

    if (sourceUser === null) {
      const message = `Nie udało się rozpoznać użytkownika ${selectedProviderDetails.label}`;
      userListRequestState = {
        status: 'error',
        provider,
        query,
        message
      };
      notifications.error('Nieprawidłowy użytkownik', message);
      return;
    }

    await loadParsedSourceUser(provider, query, sourceUser);
  }

  async function loadParsedSourceUser(
    provider: Provider,
    query: string,
    sourceUser: ParsedSourceUser,
    options: { throwErrors?: boolean; handleCloudflare?: boolean } = {}
  ) {
    const throwErrors = options.throwErrors ?? false;
    const handleCloudflare = options.handleCloudflare ?? true;
    const requestId = activeRequestId + 1;
    activeRequestId = requestId;
    activeUserListAbortController?.abort();
    const abortController = new AbortController();
    activeUserListAbortController = abortController;
    const startedAt = performance.now();
    userListRequestState = {
      status: 'loading',
      provider,
      query,
      progress: null
    };

    try {
      if (provider === 'shinden') {
        await applySavedShindenCloudflareClearance();
        if (activeRequestId !== requestId) {
          return false;
        }
      }

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
      if (provider === 'shinden' && isShindenCloudflareChallengeError(error)) {
        clearShindenCloudflareClearance();

        if (!handleCloudflare) {
          throw new Error(
            'Shinden nadal wymaga weryfikacji Cloudflare. Spróbuj ponownie przejść weryfikację.'
          );
        }

        userListRequestState = { status: 'idle' };
        shindenCloudflare.block({
          provider,
          query,
          sourceUser,
          message
        });
        return false;
      }

      if (throwErrors) {
        throw error;
      }

      userListRequestState = {
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
        activeUserListAbortController = null;
      }
    }
  }

  async function applySavedShindenCloudflareClearance() {
    const clearance = readShindenCloudflareClearance();
    if (clearance === null) {
      return;
    }

    try {
      const applied = await setShindenCloudflareClearance(clearance);
      if (!applied.accepted) {
        clearShindenCloudflareClearance();
      }
    } catch (error) {
      clearShindenCloudflareClearance();
      console.warn(
        'Unable to apply persisted Shinden Cloudflare clearance',
        error
      );
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

  function showMockNotifications(count: number) {
    const tones = ['info', 'success', 'warning', 'error'] as const;

    for (let index = 0; index < count; index += 1) {
      const notificationNumber = index + 1;
      const tone = tones[index % tones.length] ?? 'info';
      notifications.notify({
        tone,
        title: `Testowe powiadomienie ${notificationNumber}`,
        message:
          notificationNumber % 2 === 0
            ? 'To powiadomienie pozwala sprawdzić bufor i kolejkę wyświetlania.'
            : 'Kliknij powiadomienie, żeby zamknąć je ręcznie.',
        timeoutMs: 10000
      });
    }
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
    notifications,
    shindenCloudflare,
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
