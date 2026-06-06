<script lang="ts">
  import { onMount } from "svelte";

  import {
    loadShindenList,
    matchLoadedShindenList,
  } from "./lib/api/appService";
  import AppHeader from "./lib/components/AppHeader.svelte";
  import EmptyWorkspace from "./lib/components/EmptyWorkspace.svelte";
  import WorkspaceView from "./lib/components/WorkspaceView.svelte";
  import {
    providerById,
    providers,
    type Provider,
  } from "./lib/config/providers";
  import type {
    DatabaseState,
    MatchListResult,
    UserListRequestState,
    WorkspaceState,
  } from "./lib/domain/anime";
  import {
    errorMessage,
    initializeDatabaseState,
  } from "./lib/features/database/initializeDatabase";
  import {
    hasShindenProfileHost,
    parseShindenUserId,
  } from "./lib/features/shinden/userInput";

  let selectedProvider = $state<Provider>("shinden");
  let userQuery = $state("");
  let databaseState = $state<DatabaseState>({ status: "loading" });
  let userListRequestState = $state<UserListRequestState>({ status: "idle" });
  let workspaceState = $state<WorkspaceState>({ status: "empty" });
  let matchResult = $state<MatchListResult | null>(null);
  let matchErrorMessage = $state<string | null>(null);
  let activeRequestId = 0;
  let databaseInitializationPromise: Promise<DatabaseState> | null = null;
  let fetchDurationMs = $state<number | null>(null);
  let matchDurationMs = $state<number | null>(null);
  let manualSelections = $state<Record<number, number>>({});

  let trimmedQuery = $derived(userQuery.trim());
  let parsedShindenUserId = $derived(parseShindenUserId(userQuery));
  let isShindenProfileInput = $derived(hasShindenProfileHost(userQuery));
  let selectedProviderDetails = $derived(providerById(selectedProvider));
  let databaseStatusText = $derived.by(() => {
    if (databaseState.status === "ready") {
      return databaseState.info.lastUpdate
        ? `Baza danych: ${databaseState.info.lastUpdate}`
        : "Baza danych załadowana";
    }

    if (databaseState.status === "error") {
      return "Baza danych niedostępna";
    }

    return "Ładowanie bazy danych";
  });
  let activeProviderDetails = $derived.by(() => {
    const state = workspaceState;

    if (state.status === "active") {
      return providerById(state.provider);
    }

    return selectedProviderDetails;
  });
  let isUserListLoading = $derived(userListRequestState.status === "loading");
  let isWaitingForDatabase = $derived(
    userListRequestState.status === "loading" &&
      databaseState.status !== "ready",
  );
  let isLoadButtonBusy = $derived(isUserListLoading || isWaitingForDatabase);
  let hasUserListError = $derived(userListRequestState.status === "error");
  let userListErrorMessage = $derived(
    userListRequestState.status === "error"
      ? userListRequestState.message
      : undefined,
  );
  let canSubmit = $derived(Boolean(trimmedQuery) && !isUserListLoading);

  onMount(() => {
    void initializeDatabase();
  });

  $effect(() => {
    if (isShindenProfileInput && selectedProvider !== "shinden") {
      selectedProvider = "shinden";
    }
  });

  async function initializeDatabase() {
    databaseState = { status: "loading" };
    databaseInitializationPromise = initializeDatabaseState();
    databaseState = await databaseInitializationPromise;
    return databaseState;
  }

  function clearUserListError() {
    if (userListRequestState.status === "error") {
      userListRequestState = { status: "idle" };
    }
  }

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    clearUserListError();

    if (!trimmedQuery) return;

    const provider = selectedProvider;
    const query = trimmedQuery;

    if (selectedProvider !== "shinden") {
      console.log("Provider loading is not implemented yet", {
        provider,
        query,
      });
      return;
    }

    if (parsedShindenUserId === null) {
      userListRequestState = {
        status: "error",
        provider,
        query,
        message: "Nie udało się rozpoznać użytkownika Shinden",
      };
      return;
    }

    const requestId = activeRequestId + 1;
    activeRequestId = requestId;
    userListRequestState = { status: "loading", provider, query };

    try {
      const fetchStartedAt = performance.now();
      const list = await loadShindenList(parsedShindenUserId);
      const nextFetchDurationMs = performance.now() - fetchStartedAt;
      const readyDatabaseState = await waitForReadyDatabase();

      if (readyDatabaseState.status !== "ready") {
        throw new Error(
          readyDatabaseState.status === "error"
            ? readyDatabaseState.message
            : "Baza danych nie jest gotowa",
        );
      }

      const matchStartedAt = performance.now();
      const nextMatchResult = await matchLoadedShindenList();
      const nextMatchDurationMs = performance.now() - matchStartedAt;

      if (activeRequestId !== requestId) return;

      userListRequestState = {
        status: "loaded",
        provider,
        query,
        entries: list.entries,
      };
      workspaceState = {
        status: "active",
        provider,
        query,
        entries: list.entries,
      };
      matchResult = nextMatchResult;
      matchErrorMessage = null;
      fetchDurationMs = nextFetchDurationMs;
      matchDurationMs = nextMatchDurationMs;
      manualSelections = {};
    } catch (error) {
      if (activeRequestId !== requestId) return;

      console.error("Unable to load Shinden user list", error);
      userListRequestState = {
        status: "error",
        provider,
        query,
        message: errorMessage(error),
      };
    }
  }

  async function waitForReadyDatabase() {
    if (databaseState.status === "ready" || databaseState.status === "error") {
      return databaseState;
    }

    if (databaseInitializationPromise === null) {
      return await initializeDatabase();
    }

    return await databaseInitializationPromise;
  }
</script>

<main
  class="app-shell"
  style:--provider-accent={selectedProviderDetails.accent}
>
  <AppHeader
    {providers}
    bind:selectedProvider
    bind:userQuery
    {databaseState}
    {databaseStatusText}
    {isLoadButtonBusy}
    {hasUserListError}
    {userListErrorMessage}
    {canSubmit}
    onClearUserListError={clearUserListError}
    onSubmit={handleSubmit}
  />

  <div class="view-stage">
    {#if workspaceState.status === "empty"}
      <div class="view-frame">
        <EmptyWorkspace providerLabel={selectedProviderDetails.label} />
      </div>
    {:else}
      <div class="view-frame view-frame--workspace-enter">
        <WorkspaceView
          providerLabel={activeProviderDetails.label}
          entries={workspaceState.entries}
          {matchResult}
          {matchErrorMessage}
          isMatching={false}
          {fetchDurationMs}
          {matchDurationMs}
          bind:manualSelections
        />
      </div>
    {/if}
  </div>
</main>

<style>
  .view-stage {
    position: relative;
    min-height: 0;
    flex: 1;
    overflow: hidden;
    contain: layout paint;
  }

  .view-frame {
    display: flex;
    position: absolute;
    inset: 0;
    min-height: 0;
    flex-direction: column;
  }

  .view-frame--workspace-enter {
    animation: workspace-enter 600ms cubic-bezier(0.22, 1, 0.36, 1) both;
    backface-visibility: hidden;
    transform: translateZ(0);
    will-change: transform, opacity;
  }

  @keyframes workspace-enter {
    from {
      opacity: 0;
      transform: translate3d(0, 4rem, 0);
    }

    to {
      opacity: 1;
      transform: translate3d(0, 0, 0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .view-frame--workspace-enter {
      animation: none;
    }
  }
</style>
