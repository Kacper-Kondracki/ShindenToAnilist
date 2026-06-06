<script lang="ts">
  import type { ShindenEntry } from "../domain/anime";
  import AnimeListPane from "./workspace/AnimeListPane.svelte";
  import WorkspaceEditorPane from "./workspace/WorkspaceEditorPane.svelte";
  import WorkspaceStatusBar from "./workspace/WorkspaceStatusBar.svelte";

  let {
    providerLabel,
    entries,
  }: {
    providerLabel: string;
    entries: ShindenEntry[];
  } = $props();

</script>

<section class="workspace-content">
  <div class="workspace-layout">
    <AnimeListPane {providerLabel} {entries} />
    <WorkspaceEditorPane />
  </div>
</section>

<WorkspaceStatusBar />

<style>
  .workspace-content {
    display: grid;
    flex: 1;
    min-height: 0;
    align-items: stretch;
  }

  .workspace-layout {
    display: grid;
    min-height: 0;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  }

  @media (width <= 48rem) {
    .workspace-layout {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
