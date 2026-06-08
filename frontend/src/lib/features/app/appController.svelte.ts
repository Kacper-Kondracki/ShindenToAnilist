import {
  getLoadedShindenEntryIds,
  loadShindenList,
  matchLoadedShindenList
} from '../../api/appService';
import { providerById, providers, type Provider } from '../../config/providers';
import { createEntryStore } from '../../data/entryStore.svelte';
import type { DatabaseState, UserListRequestState } from '../../domain/anime';
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
  const entryStore = createEntryStore();
  const workspace = createWorkspaceController(entryStore);

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
    databaseInitializationPromise = initializeDatabaseState();
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
      const list = await loadShindenList(parsedShindenUserId);
      const nextFetchDurationMs = performance.now() - fetchStartedAt;
      const readyDatabaseState = await waitForReadyDatabase();

      if (readyDatabaseState.status !== 'ready') {
        throw new Error(
          readyDatabaseState.status === 'error'
            ? readyDatabaseState.message
            : 'Baza danych nie jest gotowa'
        );
      }

      const matchStartedAt = performance.now();
      const nextMatchResult = await matchLoadedShindenList();
      const [manualIds, automaticIds, allIds] = await Promise.all([
        getLoadedShindenEntryIds('manual'),
        getLoadedShindenEntryIds('automatic'),
        getLoadedShindenEntryIds('all')
      ]);
      const nextMatchDurationMs = performance.now() - matchStartedAt;

      if (activeRequestId !== requestId) {
        return;
      }

      const entryIdsByView = {
        manual: manualIds.entryIds,
        automatic: automaticIds.entryIds,
        all: allIds.entryIds.length > 0 ? allIds.entryIds : list.entryIds
      };

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
    entryStore,
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
