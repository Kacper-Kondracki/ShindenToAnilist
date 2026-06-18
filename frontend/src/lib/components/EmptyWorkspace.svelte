<script lang="ts">
  import type { ProviderOption } from '../config/providers';
  import type { UserListRequestState } from '../domain/anime';
  import AnimatedGridPanel from './AnimatedGridPanel.svelte';
  import SourceImportProgress from './SourceImportProgress.svelte';

  let {
    provider,
    canLoadProvider,
    userListRequestState
  }: {
    provider: ProviderOption;
    canLoadProvider: boolean;
    userListRequestState: UserListRequestState;
  } = $props();
</script>

<section class="grid flex-1 p-4">
  <AnimatedGridPanel
    class="grid place-items-center overflow-hidden surface-panel"
  >
    {#if userListRequestState.status === 'loading'}
      <SourceImportProgress
        providerLabel={provider.label}
        progress={userListRequestState.progress}
      />
    {:else}
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
        <p class="text-base font-medium text-muted md:text-xl">
          {#if canLoadProvider}
            Aktywny import z {provider.label}, pozostałe źródła w budowie
          {:else}
            Możesz wybrać to źródło, ale import listy nie jest jeszcze dostępny
          {/if}
        </p>
      </div>
    {/if}
  </AnimatedGridPanel>
</section>
