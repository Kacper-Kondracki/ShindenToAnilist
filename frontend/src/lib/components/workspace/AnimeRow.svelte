<script lang="ts">
  import type { ShindenEntry } from "../../domain/anime";

  let { entry }: { entry: ShindenEntry } = $props();
</script>

<article class="anime-row">
  <div class="min-w-0">
    <h2 class="truncate text-sm font-semibold">{entry.title}</h2>
    <p class="truncate text-xs text-muted">
      {entry.animeType} · {entry.watchStatus} · {entry.watchedEpisodes}/{entry.episodes ??
        "?"}
    </p>
  </div>

  {#if entry.score !== null}
    <span class="badge badge-soft badge-info shrink-0">
      {entry.score}/10
    </span>
  {/if}
</article>

<style>
  .anime-row {
    display: flex;
    min-height: 4.5rem;
    align-items: center;
    justify-content: space-between;
    gap: calc(var(--spacing) * 3);
    border-bottom: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 8%, transparent);
    padding-inline: calc(var(--spacing) * 4);
    padding-block: calc(var(--spacing) * 3);
  }
</style>
