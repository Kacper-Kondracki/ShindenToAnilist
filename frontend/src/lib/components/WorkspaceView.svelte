<script lang="ts">
  import type { LoadedAnimeData } from '../data/loadedAnimeData.svelte';
  import type { ShindenListViews } from '../domain/anime';
  import type { WorkspaceController } from '../features/workspace/workspaceController.svelte';
  import AnimeListPane from './workspace/AnimeListPane.svelte';
  import WorkspaceEditorPane from './workspace/WorkspaceEditorPane.svelte';
  import WorkspaceStatusBar from './workspace/WorkspaceStatusBar.svelte';

  let {
    providerLabel,
    entryIdsByView,
    animeData,
    workspace
  }: {
    providerLabel: string;
    entryIdsByView: ShindenListViews;
    animeData: LoadedAnimeData;
    workspace: WorkspaceController;
  } = $props();
</script>

<section class="grid min-h-0 flex-1 items-stretch">
  <div class="workspace-layout">
    <AnimeListPane
      {providerLabel}
      {entryIdsByView}
      {animeData}
      matchResult={workspace.matchResult}
      selectedEntryId={workspace.selectedEntryId}
      onSelectEntry={workspace.selectEntry}
    />
    <WorkspaceEditorPane
      {animeData}
      selectedEntryId={workspace.selectedEntryId}
      selectedWinnerState={workspace.selectedWinnerState}
      onSetManualOverride={workspace.setManualOverride}
      onClearManualOverride={workspace.clearManualOverride}
    />
  </div>
</section>

<WorkspaceStatusBar
  entryIds={entryIdsByView.all}
  matchResult={workspace.matchResult}
  matchErrorMessage={workspace.matchErrorMessage}
  isMatching={false}
  fetchDurationMs={workspace.fetchDurationMs}
  matchDurationMs={workspace.matchDurationMs}
  manualSelections={workspace.manualOverrides}
  exportState={workspace.exportState}
  canExport={workspace.canExport}
  onExport={workspace.exportCurrentSelections}
/>

<style>
  .workspace-layout {
    display: grid;
    height: 100%;
    min-height: 0;
    gap: 2px;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    grid-template-rows: minmax(0, 1fr);
    background-color: color-mix(
      in oklab,
      var(--color-base-content) 10%,
      transparent
    );
  }

  @media (width <= 48rem) {
    .workspace-layout {
      grid-template-columns: minmax(0, 1fr);
      grid-template-rows: repeat(2, minmax(0, 1fr));
    }
  }
</style>
