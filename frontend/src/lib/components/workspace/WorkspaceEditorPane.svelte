<script lang="ts">
  import type { LoadedAnimeData } from '../../data/loadedAnimeData.svelte';
  import type { SelectedWinnerState } from '../../features/workspace/workspaceController.svelte';
  import DatabaseEntryPreview from './DatabaseEntryPreview.svelte';
  import MatchSelector from './MatchSelector.svelte';

  let {
    animeData,
    selectedEntryId,
    onSetManualOverride,
    onClearManualOverride,
    selectedWinnerState
  }: {
    animeData: LoadedAnimeData;
    selectedEntryId: number | null;
    selectedWinnerState: SelectedWinnerState;
    onSetManualOverride: (shindenId: number, databaseId: number) => void;
    onClearManualOverride: (shindenId: number) => void;
  } = $props();

  let selectedShindenEntry = $derived(
    selectedEntryId === null ? null : animeData.getShindenEntry(selectedEntryId)
  );
</script>

<section class="workspace-pane" aria-label="Editor">
  <div class="workspace-pane__body">
    {#if selectedWinnerState.status === 'no-selection'}
      <p class="workspace-empty text-sm font-medium text-muted">
        Wybierz wpis z listy
      </p>
    {:else if selectedShindenEntry === null}
      <p class="workspace-empty text-sm font-medium text-muted">
        Nie znaleziono wpisu Shinden
      </p>
    {:else}
      <div class="flex flex-col h-full">
        <div class="min-h-0 flex-1">
          {#key selectedShindenEntry.id}
            <MatchSelector
              selectedEntry={selectedShindenEntry}
              getDatabaseEntry={animeData.getDatabaseEntry}
              {onSetManualOverride}
              {onClearManualOverride}
            />
          {/key}
        </div>
        {#if selectedWinnerState.status === 'ready'}
          <DatabaseEntryPreview entry={selectedWinnerState.entry} />
        {/if}
      </div>
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
