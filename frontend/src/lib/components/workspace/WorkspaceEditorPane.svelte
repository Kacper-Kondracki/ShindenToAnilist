<script lang="ts">
  import type { DatabaseEntry } from "../../domain/anime";
  import DatabaseEntryPreview from "./DatabaseEntryPreview.svelte";

  let {
    selectedEntryId,
    selectedWinner,
  }: {
    selectedEntryId: number | null;
    selectedWinner: DatabaseEntry | null;
  } = $props();
</script>

<section class="workspace-pane" aria-label="Editor">
  <div class="workspace-pane__body">
    {#if selectedEntryId === null}
      <p class="workspace-empty text-sm font-medium text-muted">
        Wybierz wpis z listy
      </p>
    {:else if selectedWinner === null}
      <p class="workspace-empty text-sm font-medium text-muted">
        Brak automatycznego dopasowania
      </p>
    {:else}
      <DatabaseEntryPreview entry={selectedWinner} />
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
    overflow: auto;
    padding: 0;
  }

  .workspace-empty {
    padding: calc(var(--spacing) * 4);
  }
</style>
