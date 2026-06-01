<script lang="ts">
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

  let selectedProvider = $state<Provider>('shinden');
  let userQuery = $state('');
  let trimmedQuery = $derived(userQuery.trim());
  let selectedProviderDetails = $derived(
    providers.find(({ id }) => id === selectedProvider) ?? providers[0]
  );

  function handleSubmit() {
    if (!trimmedQuery) return;

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
          <p class="text-sm text-muted">Konwerter listy anime</p>
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
        <button class="btn join-item btn-primary" type="submit" disabled={!trimmedQuery}
          >Wczytaj</button
        >
      </form>
    </div>
  </header>

  <section class="app-content">
    <div class="empty-state grid place-items-center overflow-hidden surface-panel">
      <div class="empty-state__grid" aria-hidden="true"></div>
      <div class="isolate grid max-w-3xl justify-items-center gap-2 px-6 text-center">
        <p class="text-2xl font-bold md:text-4xl">Wczytaj listę, żeby rozpocząć dopasowywanie</p>
        <p class="text-base font-medium text-muted md:text-xl">
          Aktywny import z {selectedProviderDetails.label}, pozostałe źródła w budowie
        </p>
      </div>
    </div>
  </section>
</main>

<style>
  @property --empty-state-accent {
    syntax: '<color>';
    inherits: true;
    initial-value: transparent;
  }
  @property --empty-state-grid-line {
    syntax: '<color>';
    inherits: true;
    initial-value: transparent;
  }
  @property --empty-state-glow {
    syntax: '<color>';
    inherits: true;
    initial-value: transparent;
  }

  .empty-state {
    --empty-state-accent: var(--provider-accent, var(--color-primary));
    --empty-state-glow: color-mix(in oklab, var(--empty-state-accent) 36%, transparent);
    --empty-state-grid-line: color-mix(in oklab, var(--empty-state-accent) 40%, transparent);

    transition:
      --empty-state-accent 100ms ease,
      --empty-state-grid-line 100ms ease,
      --empty-state-glow 100ms ease,
      box-shadow 100ms ease;

    position: relative;
    box-shadow: inset 0 0 2rem var(--empty-state-glow);
  }

  .provider-button {
    --provider-button-color: var(--provider-button-accent, var(--color-primary));
    --btn-color: color-mix(in oklab, var(--provider-button-color) 70%, transparent);
  }

  .provider-button--selected {
    border-color: var(--provider-button-color);
    background-color: var(--provider-button-color);
    color: var(--color-primary-content);
  }

  .empty-state::before,
  .empty-state::after,
  .empty-state__grid {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }

  .empty-state::before,
  .empty-state::after {
    z-index: 1;
    content: '';
  }

  .empty-state::before {
    background: linear-gradient(
      180deg,
      var(--color-base-300),
      color-mix(in oklab, var(--color-base-300) 70%, transparent) 22%,
      transparent 56%
    );
  }

  .empty-state::after {
    background: radial-gradient(
      ellipse at center,
      transparent 44%,
      color-mix(in oklab, var(--color-base-300) 84%, transparent)
    );
  }

  .empty-state__grid {
    background-image:
      linear-gradient(var(--empty-state-grid-line) 2px, transparent 2px),
      linear-gradient(90deg, var(--empty-state-grid-line) 2px, transparent 2px);
    background-size: 4rem 4rem;
    animation: move-grid 6s linear infinite;
    filter: blur(1px);
  }

  @keyframes move-grid {
    to {
      background-position: 8rem 12rem;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    @keyframes move-grid {
    }
  }
</style>
