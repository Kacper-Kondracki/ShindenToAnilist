<script lang="ts">
  import type { DatabaseState } from '../domain/anime';
  import { databaseStatusTitle } from '../domain/animeView';

  let {
    state,
    text,
    onRetry
  }: { state: DatabaseState; text: string; onRetry: () => void } = $props();

  let usingSavedDatabase = $derived(
    state.status === 'ready' && state.updateWarning !== undefined
  );
  let retryLabel = $derived(
    state.status === 'error'
      ? 'Wczytaj ponownie bazę danych'
      : 'Ponów aktualizację bazy danych'
  );
</script>

<div
  class="database-status flex items-center gap-1 text-xs font-medium"
  class:database-status--loaded={state.status === 'ready' &&
    !usingSavedDatabase}
  class:database-status--warning={usingSavedDatabase}
  class:database-status--error={state.status === 'error'}
  aria-live="polite"
  title={databaseStatusTitle(state)}
>
  {#if state.status === 'error' || usingSavedDatabase}
    <button
      type="button"
      class="flex cursor-pointer items-center gap-1 rounded-sm text-left hover:underline focus-visible:outline focus-visible:outline-offset-2"
      aria-label={usingSavedDatabase
        ? `${text}. ${retryLabel}`
        : `${retryLabel}. ${text}`}
      onclick={onRetry}
    >
      <span
        class={`size-4 shrink-0 ${usingSavedDatabase ? 'icon-[lucide--cloud-off]' : 'icon-[lucide--rotate-cw]'}`}
        aria-hidden="true"
      ></span>
      <span>{usingSavedDatabase ? text : retryLabel}</span>
    </button>
  {:else}
    {#if state.status === 'loading'}
      <span class="loading loading-xs loading-spinner" aria-hidden="true"
      ></span>
    {:else}
      <span class="database-status__loaded-icon" aria-hidden="true"></span>
    {/if}
    <span>{text}</span>
  {/if}
</div>

<style>
  .database-status {
    color: color-mix(in oklab, var(--color-base-content) 64%, transparent);
  }

  .database-status--loaded {
    color: var(--color-success);
  }

  .database-status--error {
    color: var(--color-error);
  }

  .database-status--warning {
    color: var(--color-warning);
  }

  .database-status__loaded-icon {
    display: inline-grid;
    position: relative;
    width: 1rem;
    height: 1rem;
    flex: 0 0 1rem;
    place-items: center;

    border-radius: 999px;
    background-color: currentColor;
  }

  .database-status__loaded-icon::before {
    position: absolute;
    width: 0.32rem;
    height: 0.58rem;
    border-right: 2px solid var(--color-base-300);
    border-bottom: 2px solid var(--color-base-300);
    content: '';
    transform: rotate(45deg) translate(-0.02rem, -0.08rem);
  }
</style>
