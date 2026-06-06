<section class="workspace-pane" aria-label="Editor">
  <div class="workspace-pane__body">
    <p class="text-muted">Widok docelowy będzie tutaj.</p>
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
</style>
