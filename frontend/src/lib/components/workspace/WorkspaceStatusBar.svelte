<footer class="app-status-bar">
  <div class="app-status-bar__body">
    <span class="text-sm font-medium text-muted">Gotowe do eksportu</span>
    <button class="btn btn-error md:btn-wide" type="button">Eksport</button>
  </div>
</footer>

<style>
  .app-status-bar {
    border-top: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 10%, transparent);
    background-color: var(--color-base-200);
  }

  .app-status-bar__body {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: calc(var(--spacing) * 4);
    padding-inline: calc(var(--spacing) * 4);
    padding-block: calc(var(--spacing) * 3);
  }
</style>
