<script lang="ts">
  import type { LoadedAnimeData } from '../data/loadedAnimeData.svelte';
  import { createAnimeListPaneController } from '../features/workspace/animeListPaneController.svelte';
  import type { WorkspaceController } from '../features/workspace/workspaceController.svelte';
  import AnimeListPane from './workspace/AnimeListPane.svelte';
  import AnimeListTabs from './workspace/AnimeListTabs.svelte';
  import ContextMenuLayer from './workspace/ContextMenuLayer.svelte';
  import WorkspaceEditorPane from './workspace/WorkspaceEditorPane.svelte';
  import WorkspaceStatusBar from './workspace/WorkspaceStatusBar.svelte';
  import { closeContextMenu } from './workspace/contextMenuState.svelte';
  import type { AnimeListTabId } from './workspace/tabs';

  const specificTabIds = [
    'ignored',
    'manual',
    'automatic'
  ] as const satisfies readonly AnimeListTabId[];

  let {
    providerLabel,
    animeData,
    workspace
  }: {
    providerLabel: string;
    animeData: LoadedAnimeData;
    workspace: WorkspaceController;
  } = $props();

  const listPane = createAnimeListPaneController({
    getEntryIdsByView: () => workspace.entryIdsByView,
    getMatchResult: () => workspace.matchResult,
    getManualOverrides: () => workspace.manualOverrides,
    getIgnoredEntryIds: () => workspace.ignoredEntryIds,
    getDisplacedAutomaticEntryIds: () => workspace.displacedAutomaticEntryIds,
    getSelectedEntryId: () => workspace.selectedEntryId
  });
  let selectedEntryTabIds = $derived.by((): ReadonlySet<AnimeListTabId> => {
    if (workspace.selectedEntryId === null) {
      return new Set<AnimeListTabId>();
    }

    const containingSpecificTabs = specificTabIds.filter((tabId) =>
      workspace.entryIdsByView[tabId].some(
        (entryId) => entryId === workspace.selectedEntryId
      )
    );

    if (listPane.activeTab === 'all') {
      return new Set<AnimeListTabId>(containingSpecificTabs);
    }

    if (
      listPane.visibleEntryIds.some(
        (entryId) => entryId === workspace.selectedEntryId
      )
    ) {
      return new Set<AnimeListTabId>();
    }

    const priorityTabId = containingSpecificTabs[0];
    const containingTabIds: AnimeListTabId[] =
      priorityTabId !== undefined
        ? [priorityTabId]
        : workspace.entryIdsByView.all.some(
              (entryId) => entryId === workspace.selectedEntryId
            )
          ? ['all']
          : [];

    return new Set<AnimeListTabId>(containingTabIds);
  });
  let listOverrideCount = $derived(
    Object.keys(workspace.manualOverrides).length +
      Object.keys(workspace.ignoredEntryIds).length
  );
  let observedSelectedEntryId = $state<number | null | undefined>(undefined);

  $effect(() => {
    const selectedEntryId = workspace.selectedEntryId;

    if (observedSelectedEntryId === undefined) {
      observedSelectedEntryId = selectedEntryId;
      return;
    }

    if (selectedEntryId === observedSelectedEntryId) {
      return;
    }

    observedSelectedEntryId = selectedEntryId;
    closeContextMenu();
  });

  function resetEntry(entryId: number) {
    workspace.resetMatchSelectorQuery(entryId);
    workspace.clearManualOverride(entryId);
  }

  function canResetEntry(entryId: number) {
    return (
      workspace.matchSelectorQueries[entryId] !== undefined ||
      workspace.manualOverrides[entryId] !== undefined ||
      workspace.ignoredEntryIds[entryId] === true ||
      workspace.displacedAutomaticEntryIds[entryId] === true
    );
  }
</script>

<section class="grid min-h-0 flex-1 items-stretch">
  <div class="workspace-shell">
    <div class="workspace-header">
      <AnimeListTabs
        activeTab={listPane.activeTab}
        {selectedEntryTabIds}
        {listOverrideCount}
        onSelectTab={listPane.selectTab}
        onClearListOverrides={workspace.clearManualOverrides}
      />
    </div>
    <div class="workspace-layout">
      <AnimeListPane
        {providerLabel}
        {animeData}
        {listPane}
        selectedEntryId={workspace.selectedEntryId}
        onSelectEntry={workspace.selectEntry}
        onResetEntry={resetEntry}
        {canResetEntry}
        onToggleIgnored={workspace.setIgnored}
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
        onSelectEntry={workspace.selectEntry}
      />
    </div>
  </div>
</section>

<ContextMenuLayer />

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
  .workspace-shell {
    display: flex;
    height: 100%;
    min-width: 0;
    min-height: 0;
    flex-direction: column;
    background-color: color-mix(
      in oklab,
      var(--color-base-content) 10%,
      transparent
    );
  }

  .workspace-header {
    min-width: 0;
    flex: 0 0 auto;
    border-bottom: calc(var(--border) * 2) solid
      color-mix(in oklab, var(--color-base-content) 10%, transparent);
    background-color: var(--color-base-300);
    padding-top: calc(var(--spacing) * 1);
  }

  .workspace-layout {
    display: grid;
    flex: 1 1 auto;
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
