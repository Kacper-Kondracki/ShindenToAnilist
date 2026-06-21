<script lang="ts">
  import { untrack } from 'svelte';

  import type { ProviderOption } from '../config/providers';
  import type { UserListRequestState } from '../domain/anime';
  import { isReadySourceImportProgress } from '../features/source/sourceImportProgressVisibility';
  import AnimatedGridPanel from './AnimatedGridPanel.svelte';
  import AuroraPanel from './AuroraPanel.svelte';
  import SourceImportProgress from './SourceImportProgress.svelte';

  let {
    provider,
    animateOnMount = true,
    canLoadProvider,
    sourceImportProgressDebounced = false,
    userListRequestState,
    onCancelLoad
  }: {
    provider: ProviderOption;
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
        animeZoneListProgressDebounced: sourceImportProgressDebounced
      }
    )
      ? userListRequestState.progress
      : null;
  });
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
      <div
        class="isolate grid max-w-3xl justify-items-center gap-2 px-6 text-center"
      >
        <p class="text-2xl font-bold md:text-4xl">
          {#if canLoadProvider}
            Wczytaj listę, żeby rozpocząć dopasowywanie
          {:else}
            {provider.label} jest jeszcze w budowie
          {/if}
        </p>
        <p class="text-muted text-base font-medium md:text-xl">
          {#if canLoadProvider}
            Aktywny import z {provider.label}, pozostałe źródła w budowie
          {:else}
            Możesz wybrać to źródło, ale import listy nie jest jeszcze dostępny
          {/if}
        </p>
      </div>
    </AnimatedGridPanel>
  {/if}
</section>

<style>
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
    :global(.empty-workspace-panel--enter) {
      animation: none;
    }
  }
</style>
