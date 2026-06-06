<script lang="ts">
  import type { ShindenEntry } from "../../domain/anime";

  let {
    selectedEntryId,
    selectedEntry,
  }: {
    selectedEntryId: number | null;
    selectedEntry: ShindenEntry | null;
  } = $props();
</script>

<section class="workspace-pane" aria-label="Editor">
  <div class="workspace-pane__body">
    {#if selectedEntry !== null}
      <div class="editor-selection">
        <p class="text-xs font-medium text-muted">Wybrany wpis</p>
        <h2 class="truncate text-base font-semibold">{selectedEntry.title}</h2>
        <p class="text-xs text-muted">ID Shinden: {selectedEntryId}</p>
      </div>
    {:else}
      <p class="workspace-empty text-sm font-medium text-muted">
        Wybierz wpis z listy
      </p>
    {/if}
  </div>
</section>

<style>
  .workspace-pane {
    display: flex;
    min-width: 0;
    flex-direction: column;
    overflow: hidden;
    background-color: var(--color-base-300);
  }

  .workspace-pane__body {
    display: block;
    flex: 1;
    min-height: 0;
    overflow: hidden;
    padding: 0;
  }

  .editor-selection,
  .workspace-empty {
    padding: calc(var(--spacing) * 4);
  }
</style>
