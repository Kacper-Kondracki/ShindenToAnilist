<script lang="ts">
  import type { Snippet } from 'svelte';

  type DialogTone = 'error' | 'warning';

  let {
    open,
    titleId,
    title,
    tone = 'warning',
    confirmTone = tone,
    cancelLabel = 'Anuluj',
    confirmLabel,
    onCancel,
    onConfirm,
    children
  }: {
    open: boolean;
    titleId: string;
    title: string;
    tone?: DialogTone;
    confirmTone?: DialogTone;
    cancelLabel?: string;
    confirmLabel: string;
    onCancel: () => void;
    onConfirm: () => void;
    children?: Snippet;
  } = $props();

  function handleKeydown(event: KeyboardEvent) {
    if (!open) {
      return;
    }

    if (event.key === 'Escape') {
      event.preventDefault();
      onCancel();
      return;
    }

    if (event.key === 'Enter') {
      event.preventDefault();
      onConfirm();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div
    class="modal modal-open"
    role="dialog"
    aria-modal="true"
    aria-labelledby={titleId}
  >
    <div class="modal-box max-w-md">
      <div class="flex items-start gap-3">
        <span
          aria-hidden="true"
          class:text-error={tone === 'error'}
          class:text-warning={tone === 'warning'}
          class="icon-[lucide--triangle-alert] mt-1 size-5 shrink-0"
        ></span>
        <div class="min-w-0">
          <h2 id={titleId} class="text-lg font-bold">{title}</h2>
          {#if children}
            <div class="text-muted mt-2 text-sm leading-6">
              {@render children()}
            </div>
          {/if}
        </div>
      </div>
      <div class="modal-action">
        <button class="btn btn-ghost" type="button" onclick={onCancel}>
          {cancelLabel}
        </button>
        <button
          class:btn-error={confirmTone === 'error'}
          class:btn-warning={confirmTone === 'warning'}
          class="btn"
          type="button"
          onclick={onConfirm}
        >
          {confirmLabel}
        </button>
      </div>
    </div>
    <button
      class="modal-backdrop"
      type="button"
      aria-label="Zamknij"
      onclick={onCancel}
    ></button>
  </div>
{/if}
