<script lang="ts">
  import { onMount } from "svelte";

  import { AppService } from "../bindings/shindentoanilist";
  import AnimatedGridPanel from "./lib/components/AnimatedGridPanel.svelte";
  import WorkspaceView from "./lib/components/WorkspaceView.svelte";

  type ProviderOption = {
    id: string;
    label: string;
    site: string;
    accent: string;
    disabled?: boolean;
  };

  const providers = [
    {
      id: "shinden",
      label: "Shinden",
      site: "shinden.pl",
      accent: "var(--color-purple-300)",
      disabled: false,
    },
    {
      id: "ogladaj-anime",
      label: "Oglądaj Anime",
      site: "ogladajanime.pl",
      accent: "var(--color-cyan-300)",
      disabled: false,
    },
    {
      id: "anime-zone",
      label: "AnimeZone",
      site: "animezone.pl",
      accent: "var(--color-rose-300)",
      disabled: false,
    },
  ] as const satisfies readonly ProviderOption[];

  type Provider = (typeof providers)[number]["id"];
  type DatabaseInfo = {
    lastUpdate: string;
    release: string;
    sha256: string;
    path: string;
    updated: boolean;
  };
  type ShindenEntry = {
    id: number;
    coverId: number | null;
    title: string;
    animeStatus: string;
    animeType: string;
    premiereDate: string | null;
    finishDate: string | null;
    episodes: number | null;
    isFavourite: boolean;
    watchStatus: string;
    watchedEpisodes: number;
    score: number | null;
    note: string | null;
    description: string | null;
  };
  type ShindenList = {
    entries: ShindenEntry[];
  };
  type LoadedUserList = {
    provider: Provider;
    query: string;
    entries: ShindenEntry[];
  };
  type DatabaseState =
    | { status: "loading" }
    | { status: "ready"; info: DatabaseInfo }
    | { status: "error"; message: string };
  type UserListRequestState =
    | { status: "idle" }
    | { status: "loading"; provider: Provider; query: string }
    | ({ status: "loaded" } & LoadedUserList)
    | { status: "error"; provider: Provider; query: string; message: string };
  type WorkspaceState =
    | { status: "empty" }
    | ({ status: "active" } & LoadedUserList);

  const databaseRetryDelays = [10000, 500, 1500] as const;

  let selectedProvider = $state<Provider>("shinden");
  let userQuery = $state("");
  let databaseState = $state<DatabaseState>({ status: "loading" });
  let userListRequestState = $state<UserListRequestState>({ status: "idle" });
  let workspaceState = $state<WorkspaceState>({ status: "empty" });
  let trimmedQuery = $derived(userQuery.trim());
  let parsedShindenUserId = $derived(parseShindenUserId(userQuery));
  let isShindenProfileInput = $derived(hasShindenProfileHost(userQuery));
  let selectedProviderDetails = $derived(
    providers.find(({ id }) => id === selectedProvider) ?? providers[0],
  );
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
      return providers.find(({ id }) => id === state.provider) ?? providers[0];
    }

    return selectedProviderDetails;
  });
  let isUserListLoading = $derived(userListRequestState.status === "loading");
  let isWaitingForDatabase = $derived(
    userListRequestState.status === "loaded" &&
      databaseState.status !== "ready",
  );
  let isLoadButtonBusy = $derived(isUserListLoading || isWaitingForDatabase);
  let hasUserListError = $derived(userListRequestState.status === "error");
  let canSubmit = $derived(Boolean(trimmedQuery) && !isUserListLoading);

  onMount(() => {
    void initializeDatabase();
  });

  $effect(() => {
    if (isShindenProfileInput && selectedProvider !== "shinden") {
      selectedProvider = "shinden";
    }
  });

  $effect(() => {
    if (
      databaseState.status === "ready" &&
      userListRequestState.status === "loaded"
    ) {
      workspaceState = {
        status: "active",
        provider: userListRequestState.provider,
        query: userListRequestState.query,
        entries: userListRequestState.entries,
      };
    }
  });

  async function initializeDatabase() {
    databaseState = { status: "loading" };

    let lastError: unknown = null;

    for (const [attempt, delayMs] of databaseRetryDelays.entries()) {
      if (delayMs > 0) {
        await delay(delayMs);
      }

      try {
        const info = (await AppService.EnsureDatabase()) as DatabaseInfo;
        databaseState = { status: "ready", info };
        return;
      } catch (error) {
        lastError = error;
        if (attempt === databaseRetryDelays.length - 1) {
          break;
        }
      }
    }

    databaseState = { status: "error", message: errorMessage(lastError) };
  }

  function delay(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  function errorMessage(error: unknown) {
    if (error instanceof Error) {
      return error.message;
    }

    if (typeof error === "string") {
      return error;
    }

    return "Nie udało się wczytać bazy danych";
  }

  function parseShindenUserId(value: string) {
    const query = value.trim();
    if (!query) return null;

    const match = query.match(
      /^(?:https?:\/\/)?(?:www\.)?(?:shinden\.pl\/user\/)?(\d+)(?:-[A-Za-z0-9_-]+)?\/?$/,
    );

    if (!match) return null;

    const userId = Number(match[1]);
    return Number.isSafeInteger(userId) && userId > 0 ? userId : null;
  }

  function hasShindenProfileHost(value: string) {
    return /^(?:https?:\/\/)?(?:www\.)?shinden\.pl\/user\/\d+(?:-[A-Za-z0-9_-]+)?\/?$/.test(
      value.trim(),
    );
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

    userListRequestState = { status: "loading", provider, query };

    try {
      const list = (await AppService.LoadShindenList(
        parsedShindenUserId,
      )) as ShindenList;
      userListRequestState = {
        status: "loaded",
        provider,
        query,
        entries: list.entries,
      };
    } catch (error) {
      console.error("Unable to load Shinden user list", error);
      userListRequestState = {
        status: "error",
        provider,
        query,
        message: errorMessage(error),
      };
    }
  }
</script>

<main
  class="app-shell"
  style:--provider-accent={selectedProviderDetails.accent}
>
  <header class="app-header">
    <div class="app-header-body">
      <div class="app-header-primary">
        <div class="min-w-52">
          <h1 class="text-xl font-bold">ShindenToAnilist</h1>

          <div
            class="database-status flex items-center gap-1 text-xs font-medium"
            class:database-status--loaded={databaseState.status === "ready"}
            class:database-status--error={databaseState.status === "error"}
            aria-live="polite"
            title={databaseState.status === "error"
              ? databaseState.message
              : databaseState.status === "ready"
                ? databaseState.info.path
                : undefined}
          >
            {#if databaseState.status === "loading"}
              <span
                class="loading loading-xs loading-spinner"
                aria-hidden="true"
              ></span>
            {:else if databaseState.status === "ready"}
              <span
                class="database-status__icon database-status__icon--loaded"
                aria-hidden="true"
              ></span>
            {:else}
              <span
                class="database-status__icon database-status__icon--error"
                aria-hidden="true"
              ></span>
            {/if}
            <span>{databaseStatusText}</span>
          </div>
        </div>

        <div class="join shrink-0">
          {#each providers as provider}
            <button
              type="button"
              class:provider-button--selected={selectedProvider === provider.id}
              class:btn-ghost={selectedProvider !== provider.id}
              class="provider-button btn join-item border-0 btn-soft"
              style:--provider-button-accent={provider.accent}
              disabled={provider.disabled}
              aria-pressed={selectedProvider === provider.id}
              title={provider.site}
              onclick={() => (selectedProvider = provider.id)}
            >
              {provider.label}
            </button>
          {/each}
        </div>
      </div>

      <form class="join flex-1" onsubmit={handleSubmit}>
        <label
          class="input join-item flex-1 user-list-input"
          class:input-error={hasUserListError}
          title={userListRequestState.status === "error"
            ? userListRequestState.message
            : undefined}
        >
          <span class="sr-only">ID lub nazwa użytkownika</span>
          <input
            bind:value={userQuery}
            type="text"
            placeholder="ID, profil Shinden lub nazwa użytkownika"
            autocomplete="off"
            oninput={clearUserListError}
            aria-invalid={hasUserListError}
          />
        </label>
        <button
          class:load-button--active={isLoadButtonBusy}
          class="load-button btn join-item btn-info"
          type="submit"
          disabled={!canSubmit}
        >
          <span class="load-button__text">Wczytaj</span>
        </button>
      </form>
    </div>
  </header>

  <div class="view-stage">
    {#if workspaceState.status === "empty"}
      <div class="view-frame">
        <section class="app-content">
          <AnimatedGridPanel
            class="grid place-items-center overflow-hidden surface-panel"
          >
            <div
              class="isolate grid max-w-3xl justify-items-center gap-2 px-6 text-center"
            >
              <p class="text-2xl font-bold md:text-4xl">
                Wczytaj listę, żeby rozpocząć dopasowywanie
              </p>
              <p class="text-base font-medium text-muted md:text-xl">
                Aktywny import z {selectedProviderDetails.label}, pozostałe
                źródła w budowie
              </p>
            </div>
          </AnimatedGridPanel>
        </section>
      </div>
    {:else}
      <div class="view-frame view-frame--workspace-enter">
        <WorkspaceView
          providerLabel={activeProviderDetails.label}
          entries={workspaceState.entries}
        />
      </div>
    {/if}
  </div>
</main>

<style>
  .user-list-input {
    transition: border-color 150ms ease;
  }

  .provider-button {
    --provider-button-color: var(
      --provider-button-accent,
      var(--color-primary)
    );
    --btn-color: color-mix(
      in oklab,
      var(--provider-button-color) 70%,
      transparent
    );
  }

  .provider-button--selected {
    border-color: var(--provider-button-color);
    background-color: var(--provider-button-color);
    color: var(--color-primary-content);
  }

  .load-button {
    --load-button-highlight: color-mix(in oklab, white 44%, transparent);
    position: relative;
    isolation: isolate;
    overflow: hidden;
    min-width: 6rem;
    transition:
      transform 160ms ease,
      background-color 180ms ease,
      box-shadow 180ms ease,
      color 180ms ease;
  }

  .load-button::before,
  .load-button::after {
    position: absolute;
    border-radius: inherit;
    content: "";
    pointer-events: none;
  }

  .load-button::before {
    inset: -35% -70%;
    z-index: 0;
    background: linear-gradient(
      110deg,
      transparent 34%,
      var(--load-button-highlight) 46%,
      transparent 58%
    );
    opacity: 0;
    transform: translateX(-45%) rotate(5deg);
  }

  .load-button::after {
    inset: 1px;
    z-index: 0;
    background:
      radial-gradient(
        circle at 50% 20%,
        color-mix(in oklab, white 32%, transparent),
        transparent 75%
      ),
      linear-gradient(
        to bottom,
        color-mix(in oklab, white 14%, transparent),
        transparent 58%
      );
    opacity: 0;
    transition: opacity 180ms ease;
  }

  .load-button--active {
    filter: saturate(2) hue-rotate(0deg);
    --load-button-accent: color-mix(
      in oklab,
      var(--provider-accent) 75%,
      black
    );
    --btn-color: var(--load-button-accent);
    --load-button-highlight: color-mix(in oklab, white 62%, transparent);

    cursor: wait;
    border-color: color-mix(in oklab, var(--load-button-accent) 78%, white);
    background-color: var(--load-button-accent);
    color: var(--color-primary-content);
    box-shadow:
      0 0 0 1px color-mix(in oklab, var(--load-button-accent) 42%, transparent),
      0 0.45rem 1.1rem
        color-mix(in oklab, var(--load-button-accent) 34%, transparent),
      0 0 1.75rem color-mix(in oklab, var(--color-info) 26%, transparent);
    animation: load-button-halo 2.4s ease-in-out infinite;
  }

  .load-button--active::before {
    opacity: 0.9;
    animation: load-button-sheen 0.9s linear infinite;
  }

  .load-button--active::after {
    opacity: 0.72;
  }

  .load-button__text {
    position: relative;
    z-index: 1;
  }

  .load-button--active .load-button__text {
    animation: load-button-text 2.4s ease-in-out infinite;
  }

  @keyframes load-button-halo {
    0%,
    100% {
      background-color: color-mix(
        in oklab,
        var(--load-button-accent) 86%,
        var(--color-info)
      );
      box-shadow:
        0 0 0 1px
          color-mix(in oklab, var(--load-button-accent) 36%, transparent),
        0 0.4rem 1rem
          color-mix(in oklab, var(--load-button-accent) 28%, transparent),
        0 0 1.45rem color-mix(in oklab, var(--color-info) 20%, transparent);
    }

    50% {
      background-color: color-mix(
        in oklab,
        var(--load-button-accent) 78%,
        white
      );
      box-shadow:
        0 0 0 1px
          color-mix(in oklab, var(--load-button-accent) 56%, transparent),
        0 0.55rem 1.35rem
          color-mix(in oklab, var(--load-button-accent) 42%, transparent),
        0 0 1.95rem color-mix(in oklab, var(--color-info) 30%, transparent);
    }
  }

  @keyframes load-button-sheen {
    0% {
      transform: translateX(-45%) rotate(5deg);
    }

    58%,
    100% {
      transform: translateX(45%) rotate(5deg);
    }
  }

  @keyframes load-button-text {
    0%,
    100% {
      opacity: 0.92;
      text-shadow: 0 0 0 color-mix(in oklab, white 0%, transparent);
    }

    50% {
      opacity: 1;
      text-shadow: 0 0 0.75rem color-mix(in oklab, white 42%, transparent);
    }
  }

  .database-status {
    color: color-mix(in oklab, var(--color-base-content) 64%, transparent);
  }

  .database-status--loaded {
    color: var(--color-success);
  }

  .database-status--error {
    color: var(--color-error);
  }

  .database-status__icon {
    display: inline-grid;
    position: relative;
    width: 1rem;
    height: 1rem;
    flex: 0 0 1rem;
    place-items: center;

    border-radius: 999px;
    background-color: currentColor;
  }

  .database-status__icon--loaded::before {
    position: absolute;
    width: 0.32rem;
    height: 0.58rem;
    border-right: 2px solid var(--color-base-300);
    border-bottom: 2px solid var(--color-base-300);
    content: "";
    transform: rotate(45deg) translate(-0.02rem, -0.08rem);
  }

  .database-status__icon--error::before,
  .database-status__icon--error::after {
    position: absolute;
    border-radius: 999px;
    background-color: var(--color-base-300);
    content: "";
  }

  .database-status__icon--error::before {
    width: 0.12rem;
    height: 0.5rem;
    transform: translateY(-0.12rem);
  }

  .database-status__icon--error::after {
    width: 0.14rem;
    height: 0.14rem;
    transform: translateY(0.3rem);
  }

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
