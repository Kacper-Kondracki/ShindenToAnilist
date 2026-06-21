<script lang="ts">
  import type { ShindenCloudflareState } from '../features/shinden/cloudflareController.svelte';

  let {
    state,
    busy,
    onCancel,
    onOpenVerification
  }: {
    state: ShindenCloudflareState;
    busy: boolean;
    onCancel: () => void;
    onOpenVerification: () => void;
  } = $props();

  let statusText = $derived.by(() => {
    switch (state.status) {
      case 'openingWindow':
        return 'Otwieranie okna Shindena';
      case 'applyingClearance':
        return 'Zapisywanie weryfikacji';
      case 'retrying':
        return 'Ponawianie importu';
      case 'failed':
        return state.message;
      default:
        return 'Shinden wymaga weryfikacji Cloudflare';
    }
  });
</script>

{#if state.status !== 'idle'}
  <div class="modal modal-open" role="dialog" aria-modal="true">
    <div class="modal-box max-w-lg">
      <div class="grid gap-4">
        <div class="flex items-start gap-3">
          <span
            class="icon-[lucide--shield-check] text-warning mt-1 size-6 shrink-0"
            aria-hidden="true"
          ></span>
          <div class="min-w-0">
            <h2 class="text-lg font-bold">Weryfikacja Shindena</h2>
            <p class="text-muted mt-1 text-sm">{statusText}</p>
          </div>
        </div>

        <p class="text-sm leading-6">
          Otwórz okno Shindena, przejdź weryfikację, a potem zamknij to okno.
          Import zostanie wznowiony automatycznie po odczytaniu weryfikacji.
        </p>

        <div class="modal-action">
          <button
            class="btn btn-ghost"
            type="button"
            disabled={busy}
            onclick={onCancel}
          >
            Anuluj
          </button>
          <button
            class="btn btn-warning"
            type="button"
            disabled={busy}
            onclick={onOpenVerification}
          >
            {#if busy}
              <span class="loading loading-spinner loading-sm"></span>
            {:else}
              <span
                class="icon-[lucide--external-link] size-4"
                aria-hidden="true"
              ></span>
            {/if}
            <span>Otwórz weryfikację</span>
          </button>
        </div>
      </div>
    </div>
    <button
      class="modal-backdrop"
      type="button"
      aria-label="Anuluj weryfikację Shindena"
      disabled={busy}
      onclick={onCancel}
    ></button>
  </div>
{/if}
