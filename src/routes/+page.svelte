<script lang="ts">
  import { onMount } from 'svelte';

  import AnimatedGridPanel from '$lib/components/AnimatedGridPanel.svelte';
  import WorkspaceView from '$lib/components/WorkspaceView.svelte';
  import { getAppStatus, loadDatabase, updateDatabase } from '$lib/tauri/pipeline';

  type ProviderOption = {
    id: string;
    label: string;
    site: string;
    accent: string;
    disabled?: boolean;
  };

  const providers = [
    {
      id: 'shinden',
      label: 'Shinden',
      site: 'shinden.pl',
      accent: 'var(--color-purple-300)',
      disabled: false
    },
    {
      id: 'ogladaj-anime',
      label: 'Oglądaj Anime',
      site: 'ogladajanime.pl',
      accent: 'var(--color-cyan-300)',
      disabled: false
    },
    {
      id: 'anime-zone',
      label: 'AnimeZone',
      site: 'animezone.pl',
      accent: 'var(--color-rose-300)',
      disabled: false
    }
  ] as const satisfies readonly ProviderOption[];

  type Provider = (typeof providers)[number]['id'];
  type DatabaseLoadState = 'loading' | 'loaded' | 'error';
  type AppView = 'start' | 'workspace';

  let selectedProvider = $state<Provider>('shinden');
  let userQuery = $state('');
  let appView = $state<AppView>('start');
  let databaseLoadState = $state<DatabaseLoadState>('loading');
  let databaseLastUpdate = $state<string | null>(null);
  let databaseError = $state<string | null>(null);
  let trimmedQuery = $derived(userQuery.trim());
  let selectedProviderDetails = $derived(
    providers.find(({ id }) => id === selectedProvider) ?? providers[0]
  );
  let databaseStatusText = $derived.by(() => {
    if (databaseLoadState === 'loaded') {
      return databaseLastUpdate ? `Baza danych: ${databaseLastUpdate}` : 'Baza danych załadowana';
    }

    if (databaseLoadState === 'error') {
      return 'Baza danych niedostępna';
    }

    return 'Ładowanie bazy danych';
  });

  onMount(() => {
    void initializeDatabase();
  });

  async function initializeDatabase() {
    databaseLoadState = 'loading';
    databaseError = null;

    try {
      const status = await getAppStatus();

      if (status.databaseLoaded && status.databaseLastUpdate) {
        databaseLastUpdate = status.databaseLastUpdate;
      } else {
        if (status.databaseExists) {
          try {
            await updateDatabase();
          } catch (error) {
            console.warn('Database update failed, loading local database', error);
          }
        } else {
          await updateDatabase();
        }

        const database = await loadDatabase();
        databaseLastUpdate = database.lastUpdate;
      }

      databaseLoadState = 'loaded';
    } catch (error) {
      databaseError = getErrorMessage(error);
      databaseLoadState = 'error';
    }
  }

  function getErrorMessage(error: unknown) {
    if (error instanceof Error) return error.message;
    if (typeof error === 'object' && error && 'message' in error) {
      return String(error.message);
    }

    return String(error);
  }

  function handleSubmit(event: SubmitEvent) {
    event.preventDefault();

    if (!trimmedQuery) return;

    appView = 'workspace';

    console.log('Load user list', {
      provider: selectedProvider,
      query: trimmedQuery
    });
  }
</script>

<main class="app-shell" style:--provider-accent={selectedProviderDetails.accent}>
  <header class="app-header">
    <div class="app-header-body">
      <div class="app-header-primary">
        <div class="min-w-52">
          <h1 class="text-xl font-bold">ShindenToAnilist</h1>

          <div
            class="database-status flex items-center gap-1 text-xs font-medium"
            class:database-status--loaded={databaseLoadState === 'loaded'}
            class:database-status--error={databaseLoadState === 'error'}
            aria-live="polite"
            title={databaseError ?? undefined}
          >
            {#if databaseLoadState === 'loading'}
              <span class="loading loading-xs loading-spinner" aria-hidden="true"></span>
            {:else if databaseLoadState === 'loaded'}
              <span class="database-status__icon database-status__icon--loaded" aria-hidden="true"
              ></span>
            {:else}
              <span class="database-status__icon database-status__icon--error" aria-hidden="true"
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
        <label class="input join-item flex-1">
          <span class="sr-only">ID lub nazwa użytkownika</span>
          <input
            bind:value={userQuery}
            type="text"
            placeholder="ID lub nazwa użytkownika"
            autocomplete="off"
          />
        </label>
        <button class="btn join-item btn-info" type="submit" disabled={!trimmedQuery}
          >Wczytaj</button
        >
      </form>
    </div>
  </header>

  <div class="view-stage">
    {#if appView === 'start'}
      <div class="view-frame">
        <section class="app-content">
          <AnimatedGridPanel class="grid place-items-center overflow-hidden surface-panel">
            <div class="isolate grid max-w-3xl justify-items-center gap-2 px-6 text-center">
              <p class="text-2xl font-bold md:text-4xl">
                Wczytaj listę, żeby rozpocząć dopasowywanie
              </p>
              <p class="text-base font-medium text-muted md:text-xl">
                Aktywny import z {selectedProviderDetails.label}, pozostałe źródła w budowie
              </p>
            </div>
          </AnimatedGridPanel>
        </section>
      </div>
    {:else}
      <div class="view-frame view-frame--workspace-enter">
        <WorkspaceView providerLabel={selectedProviderDetails.label} />
      </div>
    {/if}
  </div>
</main>

<style>
  .provider-button {
    --provider-button-color: var(--provider-button-accent, var(--color-primary));
    --btn-color: color-mix(in oklab, var(--provider-button-color) 70%, transparent);
  }

  .provider-button--selected {
    border-color: var(--provider-button-color);
    background-color: var(--provider-button-color);
    color: var(--color-primary-content);
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
    content: '';
    transform: rotate(45deg) translate(-0.02rem, -0.08rem);
  }

  .database-status__icon--error::before,
  .database-status__icon--error::after {
    position: absolute;
    border-radius: 999px;
    background-color: var(--color-base-300);
    content: '';
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
