<script lang="ts">
  import type { DatabaseEntry } from "../../domain/anime";
  import { createDatabaseEntryPreviewController } from "../../features/workspace/databaseEntryPreviewController.svelte";

  let { entry }: { entry: DatabaseEntry } = $props();

  const preview = createDatabaseEntryPreviewController({
    getEntry: () => entry,
  });
</script>

<div class="database-entry-preview">
  <div
    class="database-entry-cover"
    style:--database-entry-cover-height={preview.coverHeight}
    style:--database-entry-cover-width={preview.coverWidth}
    aria-label="Okładka dopasowanego anime"
  >
    {#if preview.coverUrl && !preview.hasCoverError}
      {#if !preview.isCoverLoaded}
        <span class="database-entry-cover__skeleton skeleton" aria-hidden="true"
        ></span>
      {/if}
      <img
        class:database-entry-cover__image--loaded={preview.isCoverLoaded}
        class="database-entry-cover__image"
        src={preview.coverUrl}
        alt={`Okładka: ${entry.title}`}
        loading="lazy"
        decoding="async"
        onload={preview.handleCoverLoad}
        onerror={preview.handleCoverError}
      />
    {:else}
      <span
        class="database-entry-cover__placeholder text-xs font-medium text-muted"
      >
        Brak okładki
      </span>
    {/if}
  </div>

  <div class="database-entry-details" bind:clientHeight={preview.detailsHeight}>
    <div class="database-entry-title">
      <p class="text-xs font-medium text-muted">Tytuł</p>
      <h2 class="text-lg font-semibold">{entry.title}</h2>
    </div>

    <dl class="database-entry-metadata">
      {#each preview.metadataItems as item}
        <div class="database-entry-metadata__item">
          <dt class="text-xs font-medium text-muted">{item.label}</dt>
          <dd class="text-sm font-semibold">{item.value}</dd>
        </div>
      {/each}
    </dl>
  </div>
</div>

<style>
  .database-entry-preview {
    display: flex;
    min-width: 0;
    align-items: flex-start;
    flex-wrap: nowrap;
    gap: calc(var(--spacing) * 4);
    padding: calc(var(--spacing) * 4);
    white-space: nowrap;
  }

  .database-entry-cover {
    display: grid;
    position: relative;
    flex: 0 0 var(--database-entry-cover-width);
    width: var(--database-entry-cover-width);
    height: var(--database-entry-cover-height);
    aspect-ratio: 3 / 4;
    place-items: center;
    overflow: hidden;
    border-radius: var(--radius-box);
    background-color: var(--color-base-200);
  }

  .database-entry-cover::after {
    content: "";
    position: absolute;
    z-index: 3;
    inset: -2px;
    border-radius: calc(var(--radius-box) - 1px);
    background:
      linear-gradient(
        90deg,
        rgb(0 0 0 / 30%) 0%,
        transparent 18%,
        transparent 82%,
        rgb(0 0 0 / 34%) 100%
      ),
      linear-gradient(
        180deg,
        rgb(255 255 255 / 10%) 0%,
        transparent 24%,
        transparent 68%,
        rgb(0 0 0 / 42%) 100%
      ),
      radial-gradient(ellipse at center, transparent 44%, rgb(0 0 0 / 22%) 100%);
    pointer-events: none;
  }

  .database-entry-cover__image,
  .database-entry-cover__skeleton {
    grid-area: 1 / 1;
    width: 100%;
    height: 100%;
  }

  .database-entry-cover__image {
    position: relative;
    z-index: 1;
    object-fit: cover;
    opacity: 0;
    transition: opacity 200ms cubic-bezier(0.445, 0.05, 0.55, 0.95);
  }

  .database-entry-cover__image--loaded {
    opacity: 1;
  }

  .database-entry-cover__skeleton {
    border-radius: inherit;
  }

  .database-entry-cover__placeholder {
    padding: calc(var(--spacing) * 2);
    text-align: center;
  }

  .database-entry-details {
    display: flex;
    min-width: 0;
    flex: 1 1 auto;
    flex-direction: column;
    gap: calc(var(--spacing) * 4);
  }

  .database-entry-title {
    min-width: 0;
  }

  .database-entry-title p {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .database-entry-title h2 {
    display: -webkit-box;
    min-height: 4.05em;
    max-height: 4.05em;
    overflow: hidden;
    overflow-wrap: anywhere;
    white-space: normal;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    line-height: 1.35;
  }

  .database-entry-metadata {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: calc(var(--spacing) * 3);
  }

  .database-entry-metadata__item {
    min-width: 0;
    border-top: var(--border) solid
      color-mix(in oklab, var(--color-base-content) 10%, transparent);
    padding-top: calc(var(--spacing) * 2);
  }

  .database-entry-metadata__item dd {
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .database-entry-metadata__item dt {
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
