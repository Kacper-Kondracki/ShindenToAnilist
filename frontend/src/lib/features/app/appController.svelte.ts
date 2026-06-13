import {
  fetchShindenList,
  getShindenFull,
  getShindenIds,
  matchShindenList
} from '../../api/appService';
import { providerById, providers, type Provider } from '../../config/providers';
import { createLoadedAnimeData } from '../../data/loadedAnimeData.svelte';
import type {
  DatabaseState,
  MatchListResult,
  ShindenListViews,
  UserListRequestState
} from '../../domain/anime';
import {
  errorMessage,
  initializeDatabaseState
} from '../database/initializeDatabase';
import {
  hasShindenProfileHost,
  parseShindenUserId
} from '../shinden/userInput';
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
  let databaseInitializationPromise: Promise<DatabaseState> | null = null;

  let trimmedQuery = $derived(userQuery.trim());
  let parsedShindenUserId = $derived(parseShindenUserId(userQuery));
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
  let isProviderSupported = $derived(selectedProviderDetails.supportsUserList);
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
    selectedProvider = provider;
    clearUserListError();
  }

  function setUserQuery(value: string) {
    userQuery = value;
    clearUserListError();

    if (hasShindenProfileHost(value)) {
      selectedProvider = 'shinden';
    }
  }

  function clearUserListError() {
    if (userListRequestState.status === 'error') {
      userListRequestState = { status: 'idle' };
    }
  }

  async function submitUserList() {
    clearUserListError();

    if (!trimmedQuery || isUserListLoading || !isProviderSupported) {
      return;
    }

    const provider = selectedProvider;
    const query = trimmedQuery;

    if (provider !== 'shinden' || parsedShindenUserId === null) {
      userListRequestState = {
        status: 'error',
        provider,
        query,
        message: 'Nie udało się rozpoznać użytkownika Shinden'
      };
      return;
    }

    const requestId = activeRequestId + 1;
    activeRequestId = requestId;
    userListRequestState = { status: 'loading', provider, query };

    try {
      const fetchStartedAt = performance.now();
      const fetchedList = await fetchShindenList(parsedShindenUserId);

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

      const shindenFull = await getShindenFull();
      const nextFetchDurationMs = performance.now() - fetchStartedAt;

      if (activeRequestId !== requestId) {
        return;
      }

      const matchStartedAt = performance.now();
      const nextMatchResult = await matchShindenList();
      const allIds = await getShindenIds();
      const nextMatchDurationMs = performance.now() - matchStartedAt;

      if (activeRequestId !== requestId) {
        return;
      }

      assertConsistentWorkspaceVersions(
        fetchedList.shindenVersion,
        shindenFull.shindenVersion,
        readyDatabaseState.info.databaseVersion,
        nextMatchResult,
        allIds.shindenVersion
      );

      const entryIdsByView = buildEntryIdsByView(
        nextMatchResult,
        allIds.entryIds
      );

      animeData.replaceShindenFull(shindenFull);

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
        matchDurationMs: nextMatchDurationMs
      });
    } catch (error) {
      if (activeRequestId !== requestId) {
        return;
      }

      console.error('Unable to load Shinden user list', error);
      userListRequestState = {
        status: 'error',
        provider,
        query,
        message: errorMessage(error)
      };
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
    get activeProviderDetails() {
      return activeProviderDetails;
    },
    get databaseStatusText() {
      return databaseStatusText;
    },
    get isLoadButtonBusy() {
      return isLoadButtonBusy;
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
    initializeDatabase,
    setSelectedProvider,
    setUserQuery,
    clearUserListError,
    submitUserList
  };
}

function assertConsistentWorkspaceVersions(
  fetchedShindenVersion: number,
  fullShindenVersion: number,
  readyDatabaseVersion: number,
  matchResult: MatchListResult,
  shindenIdsVersion: number
) {
  if (
    fetchedShindenVersion !== matchResult.shindenVersion ||
    fullShindenVersion !== matchResult.shindenVersion ||
    shindenIdsVersion !== matchResult.shindenVersion
  ) {
    throw new Error(
      'Lista Shinden zmieniła się podczas dopasowywania. Spróbuj ponownie.'
    );
  }

  if (readyDatabaseVersion !== matchResult.databaseVersion) {
    throw new Error(
      'Baza danych zmieniła się podczas dopasowywania. Spróbuj ponownie.'
    );
  }
}

function buildEntryIdsByView(
  matchResult: MatchListResult,
  allEntryIds: number[]
): ShindenListViews {
  const unmatched = new Set<number>();
  const review = new Set<number>();
  const automatic = new Set<number>();

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
