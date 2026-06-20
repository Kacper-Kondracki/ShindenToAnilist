<script lang="ts">
  import type { DatabaseEntry } from '../../domain/anime';
  import { createDatabaseEntryPreviewController } from '../../features/workspace/databaseEntryPreviewController.svelte';

  let {
    entry,
    placeholder = false,
    compact = false
  }: {
    entry: DatabaseEntry;
    placeholder?: boolean;
    compact?: boolean;
  } = $props();

  const preview = createDatabaseEntryPreviewController({
    getEntry: () => entry,
    getCompact: () => compact
  });
</script>

<div
  class:database-entry-preview--placeholder={placeholder}
  class:database-entry-preview--compact={compact}
  class="database-entry-preview"
  style:--database-entry-cover-height={preview.coverHeight}
  style:--database-entry-cover-width={preview.coverWidth}
  aria-hidden={placeholder}
>
  <div class="database-entry-cover" aria-label="Okładka dopasowanego anime">
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
        class="database-entry-cover__placeholder text-muted text-xs font-medium"
      >
        Brak okładki
      </span>
    {/if}
  </div>

  <div class="database-entry-details" bind:clientHeight={preview.detailsHeight}>
    <div class="database-entry-title">
      <p class="text-muted text-xs font-medium">Tytuł</p>
      <h2 class="text-lg font-semibold">{entry.title}</h2>
    </div>

    <dl
      class="database-entry-metadata"
      bind:clientHeight={preview.metadataHeight}
    >
      {#each preview.metadataItems as item}
        <div
          class={`database-entry-metadata__item database-entry-metadata__item--${item.tone}`}
        >
          <dt>{item.label}</dt>
          <dd>{item.value}</dd>
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
    contain: layout;
    white-space: nowrap;
  }

  .database-entry-preview--compact {
    display: grid;
    grid-template-columns: var(--database-entry-cover-width) minmax(0, 1fr);
    grid-template-areas:
      'title title'
      'cover metadata';
    gap: calc(var(--spacing) * 2) calc(var(--spacing) * 4);
  }

  .database-entry-preview--placeholder {
    visibility: hidden;
    pointer-events: none;
  }

  .database-entry-cover {
    display: grid;
    position: relative;
    grid-area: cover;
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
    content: '';
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

  .database-entry-preview--compact .database-entry-details {
    display: contents;
  }

  .database-entry-title {
    grid-area: title;
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

  .database-entry-preview--compact .database-entry-title h2 {
    min-height: 2.7em;
    max-height: 2.7em;
    -webkit-line-clamp: 2;
    line-clamp: 2;
  }

  .database-entry-metadata {
    display: grid;
    grid-area: metadata;
    width: min(100%, 28rem);
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: calc(var(--spacing) * 1.5);
    justify-self: start;
  }

  .database-entry-metadata__item {
    --database-entry-metadata-accent: var(--color-base-content);
    --database-entry-metadata-glow: color-mix(
      in oklab,
      var(--database-entry-metadata-accent) 18%,
      transparent
    );

    min-width: 0;
    position: relative;
    overflow: hidden;
    border: var(--border) solid
      color-mix(
        in oklab,
        var(--database-entry-metadata-accent) 26%,
        transparent
      );
    border-radius: var(--radius-field);
    background:
      linear-gradient(
        90deg,
        color-mix(
            in oklab,
            var(--database-entry-metadata-accent) 18%,
            transparent
          )
          0%,
        color-mix(
            in oklab,
            var(--database-entry-metadata-accent) 13%,
            transparent
          )
          18%,
        color-mix(
            in oklab,
            var(--database-entry-metadata-accent) 7%,
            transparent
          )
          44%,
        transparent 82%
      ),
      color-mix(in oklab, var(--color-base-content) 4%, transparent);
    box-shadow: 0 0.5rem 1rem -1rem var(--database-entry-metadata-glow);
    padding: calc(var(--spacing) * 1.5) calc(var(--spacing) * 2)
      calc(var(--spacing) * 1.5) calc(var(--spacing) * 2.5);
  }

  .database-entry-metadata__item::before {
    position: absolute;
    inset-block: 0;
    left: 0;
    width: 0.25rem;
    background-color: var(--database-entry-metadata-accent);
    content: '';
  }

  .database-entry-metadata__item dd {
    margin: 0;
    color: color-mix(
      in oklab,
      var(--database-entry-metadata-accent) 72%,
      white 28%
    );
    font-size: 0.8125rem;
    font-weight: 750;
    line-height: 1.2;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .database-entry-metadata__item dt {
    margin: 0;
    color: color-mix(
      in oklab,
      var(--database-entry-metadata-accent) 46%,
      var(--color-base-content) 54%
    );
    font-size: 0.65625rem;
    font-weight: 650;
    line-height: 1.1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .database-entry-metadata__item--year {
    --database-entry-metadata-accent: var(--ctp-mocha-blue);
  }

  .database-entry-metadata__item--anime-type {
    --database-entry-metadata-accent: var(--ctp-mocha-mauve);
  }

  .database-entry-metadata__item--status-finished {
    --database-entry-metadata-accent: var(--color-success);
  }

  .database-entry-metadata__item--status-ongoing {
    --database-entry-metadata-accent: var(--color-warning);
  }

  .database-entry-metadata__item--status-upcoming {
    --database-entry-metadata-accent: var(--color-info);
  }

  .database-entry-metadata__item--status-unknown {
    --database-entry-metadata-accent: var(--color-base-content);
  }

  .database-entry-metadata__item--season {
    --database-entry-metadata-accent: var(--ctp-mocha-sky);
  }

  .database-entry-metadata__item--episodes {
    --database-entry-metadata-accent: var(--ctp-mocha-maroon);
  }

  .database-entry-metadata__item--duration {
    --database-entry-metadata-accent: var(--ctp-mocha-flamingo);
  }
</style>
