<script lang="ts">
  import type { LoadedAnimeData } from '../../data/loadedAnimeData.svelte';
  import type { DatabaseEntry } from '../../domain/anime';
  import type { MatchSelectorInitialSearch } from '../../features/workspace/matchSelectorController.svelte';
  import type { SelectedWinnerState } from '../../features/workspace/workspaceController.svelte';
  import {
    AnimeStatus,
    AnimeType,
    Season
  } from '../../gen/shinden_to_anilist/v1/common_pb';
  import DatabaseEntryPreview from './DatabaseEntryPreview.svelte';
  import MatchSelector from './MatchSelector.svelte';

  const placeholderDatabaseEntry = {
    id: 0,
    sources: [],
    title: 'Placeholder database entry',
    animeType: AnimeType.UNSPECIFIED,
    episodes: 0,
    status: AnimeStatus.UNSPECIFIED,
    season: Season.UNSPECIFIED,
    year: null,
    picture: '',
    thumbnail: '',
    duration: null,
    synonyms: []
  } satisfies DatabaseEntry;

  let {
    animeData,
    selectedEntryId,
    onSetManualOverride,
    onClearManualOverride,
    manualOverrides,
    initialMatchSearch,
    selectedWinnerState
  }: {
    animeData: LoadedAnimeData;
    selectedEntryId: number | null;
    selectedWinnerState: SelectedWinnerState;
    manualOverrides: Record<number, number>;
    initialMatchSearch: MatchSelectorInitialSearch | null;
    onSetManualOverride: (shindenId: number, databaseId: number) => void;
    onClearManualOverride: (shindenId: number) => void;
  } = $props();

  let selectedShindenEntry = $derived(
    selectedEntryId === null ? null : animeData.getShindenEntry(selectedEntryId)
  );
  let selectedDatabaseEntryId = $derived.by(() => {
    if (selectedWinnerState.status === 'ready') {
      return selectedWinnerState.entry.id;
    }

    if (selectedWinnerState.status === 'missing') {
      return selectedWinnerState.databaseEntryId;
    }

    return null;
  });
  let manualOverrideId = $derived(
    selectedEntryId === null ? null : (manualOverrides[selectedEntryId] ?? null)
  );
  let previewEntry = $derived(
    selectedWinnerState.status === 'ready'
      ? selectedWinnerState.entry
      : placeholderDatabaseEntry
  );
  let isPreviewPlaceholder = $derived(selectedWinnerState.status !== 'ready');
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
      <div class="editor-layout">
        <div class="editor-layout__selector">
          {#key selectedShindenEntry.id}
            <MatchSelector
              selectedEntry={selectedShindenEntry}
              {selectedDatabaseEntryId}
              {manualOverrideId}
              initialSearch={initialMatchSearch}
              getDatabaseEntry={animeData.getDatabaseEntry}
              {onSetManualOverride}
              {onClearManualOverride}
            />
          {/key}
        </div>
        <div class="editor-layout__preview" aria-hidden={isPreviewPlaceholder}>
          <DatabaseEntryPreview
            entry={previewEntry}
            placeholder={isPreviewPlaceholder}
          />
        </div>
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
    overflow: hidden;
    padding: 0;
  }

  .editor-layout {
    display: grid;
    width: 100%;
    max-width: 100%;
    height: 100%;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    grid-template-rows:
      minmax(0, 1fr)
      auto;
  }

  .editor-layout__selector {
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .editor-layout__preview {
    min-width: 0;
    overflow: hidden;
  }

  .workspace-empty {
    padding: calc(var(--spacing) * 4);
  }
</style>
