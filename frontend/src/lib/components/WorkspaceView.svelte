<script lang="ts">
  import type { LoadedAnimeData } from '../data/loadedAnimeData.svelte';
  import type { WorkspaceController } from '../features/workspace/workspaceController.svelte';
  import AnimeListPane from './workspace/AnimeListPane.svelte';
  import WorkspaceEditorPane from './workspace/WorkspaceEditorPane.svelte';
  import WorkspaceStatusBar from './workspace/WorkspaceStatusBar.svelte';

  let {
    providerLabel,
    animeData,
    workspace
  }: {
    providerLabel: string;
    animeData: LoadedAnimeData;
    workspace: WorkspaceController;
  } = $props();
</script>

<section class="grid min-h-0 flex-1 items-stretch">
  <div class="workspace-layout">
    <AnimeListPane
      {providerLabel}
      entryIdsByView={workspace.entryIdsByView}
      {animeData}
      matchResult={workspace.matchResult}
      manualOverrides={workspace.manualOverrides}
      ignoredEntryIds={workspace.ignoredEntryIds}
      displacedAutomaticEntryIds={workspace.displacedAutomaticEntryIds}
      selectedEntryId={workspace.selectedEntryId}
      onSelectEntry={workspace.selectEntry}
    />
    <WorkspaceEditorPane
      {animeData}
      selectedEntryId={workspace.selectedEntryId}
      selectedMatchEntry={workspace.selectedMatchEntry}
      selectedWinnerState={workspace.selectedWinnerState}
      manualOverrides={workspace.manualOverrides}
      ignoredEntryIds={workspace.ignoredEntryIds}
      displacedAutomaticEntryIds={workspace.displacedAutomaticEntryIds}
      matchSelectorQueries={workspace.matchSelectorQueries}
      winnerClaimsByDatabaseId={workspace.winnerClaimsByDatabaseId}
      initialMatchSearch={workspace.initialMatchSearch}
      onSetManualOverride={workspace.setManualOverride}
      onSetIgnored={workspace.setIgnored}
      onClearManualOverride={workspace.clearManualOverride}
      onSetMatchSelectorQuery={workspace.setMatchSelectorQuery}
      onResetMatchSelectorQuery={workspace.resetMatchSelectorQuery}
    />
  </div>
</section>

<WorkspaceStatusBar
  entryIds={workspace.entryIdsByView.all}
  matchResult={workspace.matchResult}
  matchErrorMessage={workspace.matchErrorMessage}
  isMatching={false}
  fetchDurationMs={workspace.fetchDurationMs}
  matchDurationMs={workspace.matchDurationMs}
  manualSelections={workspace.manualOverrides}
  ignoredEntryIds={workspace.ignoredEntryIds}
  displacedAutomaticEntryIds={workspace.displacedAutomaticEntryIds}
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
