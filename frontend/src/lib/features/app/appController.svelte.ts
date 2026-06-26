import {
  isShindenCloudflareChallengeError,
  openShindenCloudflareAutoCloseTest,
  openShindenCloudflareVerification,
  setShindenCloudflareClearance
} from '../../api/appService';
import { providerById, providers, type Provider } from '../../config/providers';
import { createLoadedAnimeData } from '../../data/loadedAnimeData.svelte';
import type { DatabaseState } from '../../domain/anime';
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
import { createShindenCloudflareController } from '../shinden/cloudflareController.svelte';
import { parseSourceDevCommand } from '../source/devCommands';
import { createSourceImportController } from '../source/sourceImportController.svelte';
import { providerFromInput } from '../source/userInput';
import { createWorkspaceController } from '../workspace/workspaceController.svelte';

export type AppController = ReturnType<typeof createAppController>;

export function createAppController() {
  const animeData = createLoadedAnimeData();
  const workspace = createWorkspaceController(animeData);
  const notifications = createNotificationController();
  let sourceImport: ReturnType<typeof createSourceImportController>;

  const shindenCloudflare = createShindenCloudflareController({
    openVerification: openShindenCloudflareVerification,
    applyClearance: setShindenCloudflareClearance,
    saveClearance: writeShindenCloudflareClearance,
    retry: async (request) => {
      await sourceImport.loadParsedSourceUser(
        request.provider,
        request.query,
        request.sourceUser,
        { throwErrors: true, handleProviderErrors: false }
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
  let databaseInitializationPromise: Promise<DatabaseState> | null = null;
  let lastDatabaseNotificationMessage = '';

  sourceImport = createSourceImportController({
    animeData,
    workspace,
    notifications,
    waitForReadyDatabase,
    providerHooks: {
      shinden: {
        beforeFetch: applySavedShindenCloudflareClearance,
        canHandleError: isShindenCloudflareChallengeError,
        handleBlockedImport: (request) => {
          clearShindenCloudflareClearance();
          shindenCloudflare.block(request);
        },
        retryBlockedMessage:
          'Shinden nadal wymaga weryfikacji Cloudflare. Spróbuj ponownie przejść weryfikację.'
      }
    }
  });

  let trimmedQuery = $derived(userQuery.trim());
  let userListRequestState = $derived(sourceImport.state);
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
  let isUserListLoading = $derived(sourceImport.isLoading);
  let userListRequestProviderDetails = $derived(
    userListRequestState.status === 'loading' ||
      userListRequestState.status === 'error'
      ? providerById(userListRequestState.provider)
      : selectedProviderDetails
  );
  let isProviderSupported = $derived(selectedProviderDetails.supportsUserList);
  let shouldShowSourceImportProgress = $derived(
    sourceImport.shouldShowProgress
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
    sourceImport.clearError();
  }

  async function submitUserList() {
    sourceImport.stopPreview();
    clearUserListError();

    if (!trimmedQuery || isUserListLoading || !isProviderSupported) {
      return;
    }

    const provider = selectedProvider;
    const query = trimmedQuery;
    const devCommand = parseSourceDevCommand(query);

    if (devCommand?.kind === 'sourceImportPreview') {
      sourceImport.startPreview(query);
      return;
    }

    if (devCommand?.kind === 'shindenCloudflareAutoCloseTest') {
      await runShindenCloudflareAutoCloseTest();
      return;
    }

    if (devCommand?.kind === 'showMockNotifications') {
      showMockNotifications(devCommand.count);
      return;
    }

    await sourceImport.submitQuery(provider, query);
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
    sourceImport.cancelLoad();
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

  async function runShindenCloudflareAutoCloseTest() {
    try {
      notifications.info(
        'Test okna Shindena',
        'Otwieram okno i czekam na ciasteczko sess_shinden.'
      );
      await openShindenCloudflareAutoCloseTest();
      notifications.success(
        'Test zakończony',
        'Okno Shindena zamknęło się po wykryciu ciasteczka sess_shinden.'
      );
    } catch (error) {
      const message = errorMessage(error);
      sourceImport.setError('shinden', trimmedQuery, message);
      notifications.error('Test okna Shindena nie powiódł się', message);
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
    get isSourceImportProgressDebounced() {
      return sourceImport.isProgressDebounced;
    },
    initializeDatabase,
    setSelectedProvider,
    setUserQuery,
    clearUserListError,
    submitUserList,
    cancelUserListLoad,
    startSourceImportPreview: sourceImport.startPreview,
    stopSourceImportPreview: sourceImport.stopPreview
  };
}
