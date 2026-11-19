<script lang="ts">
  import { untrack } from 'svelte';

  import type { AppUpdate } from '../api/appUpdates';
  import { reportFailure } from '../api/diagnostics';
  import { isTauriRuntime, openDesktopLink } from '../api/runtime';
  import type { ProviderOption } from '../config/providers';
  import type { UserListRequestState } from '../domain/anime';
  import { isReadySourceImportProgress } from '../features/source/sourceImportProgressVisibility';
  import AnimatedGridPanel from './AnimatedGridPanel.svelte';
  import AuroraPanel from './AuroraPanel.svelte';
  import SourceImportProgress from './SourceImportProgress.svelte';

  let {
    provider,
    update,
    animateOnMount = true,
    canLoadProvider,
    sourceImportProgressDebounced = false,
    userListRequestState,
    onCancelLoad
  }: {
    provider: ProviderOption;
    update: AppUpdate | null;
    animateOnMount?: boolean;
    canLoadProvider: boolean;
    sourceImportProgressDebounced?: boolean;
    userListRequestState: UserListRequestState;
    onCancelLoad: () => void;
  } = $props();
  const shouldAnimateOnMount = untrack(() => animateOnMount);
  let readySourceImportProgress = $derived.by(() => {
    if (
      userListRequestState.status !== 'loading' ||
      !provider.supportsSourceImportProgress ||
      userListRequestState.progress === null
    ) {
      return null;
    }

    return isReadySourceImportProgress(
      provider.id,
      userListRequestState.progress,
      {
        listPhaseDebounced: sourceImportProgressDebounced
      }
    )
      ? userListRequestState.progress
      : null;
  });

  function openUpdate(event: MouseEvent, url: string) {
    if (
      globalThis.shindenToAnilist?.openExternalUrl === undefined &&
      !isTauriRuntime()
    )
      return;

    event.preventDefault();
    void openDesktopLink(url).catch((error) => {
      reportFailure('open_app_release', error);
    });
  }
</script>

<section class="grid flex-1 p-4">
  {#if readySourceImportProgress !== null}
    <AuroraPanel>
      <SourceImportProgress
        providerLabel={provider.label}
        progress={readySourceImportProgress}
        onCancel={onCancelLoad}
      />
    </AuroraPanel>
  {:else}
    <AnimatedGridPanel
      class={`empty-workspace-panel surface-panel grid place-items-center overflow-hidden ${
        shouldAnimateOnMount ? 'empty-workspace-panel--enter' : ''
      }`}
    >
      {#if update}
        {@const release = update}
        <a
          class="app-update-link group border-info/25 hover:border-info/60 bg-base-200 shadow-base-300/40 focus-visible:outline-info absolute top-4 left-4 z-10 flex max-w-[calc(100%-2rem)] items-center gap-3 overflow-hidden rounded-xl border p-3 text-sm shadow-md transition-colors focus-visible:outline-2 focus-visible:outline-offset-2"
          href={release.url}
          target="_blank"
          rel="noopener noreferrer"
          onclick={(event) => openUpdate(event, release.url)}
          onauxclick={(event) => {
            if (event.button === 1) openUpdate(event, release.url);
          }}
        >
          <span
            class="bg-info/15 text-info grid size-9 shrink-0 place-items-center rounded-lg"
            aria-hidden="true"
          >
            <span class="icon-[lucide--circle-arrow-up] size-5"></span>
          </span>
          <span class="grid min-w-0 gap-1">
            <span class="font-semibold wrap-anywhere">
              Dostępna wersja
              <strong class="font-bold text-(--ctp-mocha-mauve)">
                {release.version}
              </strong>
            </span>
            <span
              class="text-info inline-flex items-center gap-1 font-medium underline-offset-2 group-hover:underline"
            >
              Sprawdź
              <span
                class="icon-[lucide--arrow-up-right] size-3.5"
                aria-hidden="true"
              ></span>
            </span>
          </span>
        </a>
      {/if}
      <div
        class="isolate grid max-w-3xl justify-items-center gap-2 px-6 text-center"
      >
        <p class="text-2xl font-bold md:text-4xl">
          {#if canLoadProvider}
            Wczytaj listę, żeby rozpocząć dopasowywanie
          {:else}
            Import z {provider.label} jest teraz niedostępny
          {/if}
        </p>
        {#if !canLoadProvider}
          <p class="text-muted text-base font-medium md:text-xl">
            Możesz wybrać inne źródło i wrócić do tego później
          </p>
        {/if}
      </div>
    </AnimatedGridPanel>
  {/if}
</section>

<style>
  .app-update-link {
    animation: app-update-enter 320ms cubic-bezier(0.22, 1, 0.36, 1) both;
  }

  @keyframes app-update-enter {
    from {
      opacity: 0;
      transform: translateY(-6px);
    }

    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .app-update-link::before {
    position: absolute;
    z-index: -1;
    inset: 0;
    background: linear-gradient(
      110deg,
      transparent 25%,
      color-mix(in oklab, var(--color-info) 26%, transparent) 50%,
      transparent 75%
    );
    content: '';
    pointer-events: none;
    animation: app-update-shine 6s ease-in-out infinite;
  }

  @keyframes app-update-shine {
    0%,
    12% {
      transform: translateX(-120%);
    }

    36%,
    100% {
      transform: translateX(120%);
    }
  }

  :global(.empty-workspace-panel--enter) {
    animation: empty-workspace-enter 600ms cubic-bezier(0.22, 1, 0.36, 1) both;
    backface-visibility: hidden;
    transform: translateZ(0);
    will-change: transform, opacity;
  }

  @keyframes empty-workspace-enter {
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
    .app-update-link {
      animation: none;
    }

    .app-update-link::before {
      display: none;
    }

    :global(.empty-workspace-panel--enter) {
      animation: none;
    }
  }
</style>
