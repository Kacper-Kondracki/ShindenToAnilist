<script lang="ts">
  import type { SelectedWinnerState } from "../../features/workspace/workspaceController.svelte";
  import DatabaseEntryPreview from "./DatabaseEntryPreview.svelte";

  let {
    selectedWinnerState,
  }: {
    selectedWinnerState: SelectedWinnerState;
  } = $props();
</script>

<section class="workspace-pane" aria-label="Editor">
  <div class="workspace-pane__body">
    {#if selectedWinnerState.status === "no-selection"}
      <p class="workspace-empty text-sm font-medium text-muted">
        Wybierz wpis z listy
      </p>
    {:else if selectedWinnerState.status === "no-winner"}
      <p class="workspace-empty text-sm font-medium text-muted">
        Brak automatycznego dopasowania
      </p>
    {:else if selectedWinnerState.status === "loading"}
      <p class="workspace-empty text-sm font-medium text-muted">
        Wczytywanie dopasowania
      </p>
    {:else if selectedWinnerState.status === "missing"}
      <p class="workspace-empty text-sm font-medium text-muted">
        Nie znaleziono wpisu w bazie
      </p>
    {:else if selectedWinnerState.status === "error"}
      <p
        class="workspace-empty text-sm font-medium text-error"
        title={selectedWinnerState.message}
      >
        Nie udało się wczytać dopasowania
      </p>
    {:else}
      <DatabaseEntryPreview entry={selectedWinnerState.entry} />
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
