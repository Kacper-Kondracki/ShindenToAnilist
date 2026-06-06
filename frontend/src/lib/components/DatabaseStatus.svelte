<script lang="ts">
  import type { DatabaseState } from "../domain/anime";

  let { state, text }: { state: DatabaseState; text: string } = $props();
</script>

<div
  class="database-status flex items-center gap-1 text-xs font-medium"
  class:database-status--loaded={state.status === "ready"}
  class:database-status--error={state.status === "error"}
  aria-live="polite"
  title={state.status === "error"
    ? state.message
    : state.status === "ready"
      ? state.info.path
      : undefined}
>
  {#if state.status === "loading"}
    <span class="loading loading-xs loading-spinner" aria-hidden="true"></span>
  {:else if state.status === "ready"}
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
  <span>{text}</span>
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
</style>
