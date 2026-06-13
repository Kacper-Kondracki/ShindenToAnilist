<script lang="ts">
  import { AnimeStatus } from '../../gen/shinden_to_anilist/v1/common_pb';

  export type RowMetadataBadge = {
    label: string;
    tone: 'year' | 'season' | 'anime-type' | 'status';
    animeStatus?: AnimeStatus;
  };

  let {
    items,
    ariaLabel = 'Metadane anime'
  }: {
    items: RowMetadataBadge[];
    ariaLabel?: string;
  } = $props();

  function badgeTone(item: RowMetadataBadge) {
    if (item.tone !== 'status') {
      return item.tone;
    }

    switch (item.animeStatus) {
      case AnimeStatus.FINISHED:
        return 'status-finished';
      case AnimeStatus.ONGOING:
        return 'status-ongoing';
      case AnimeStatus.UPCOMING:
        return 'status-upcoming';
      default:
        return 'status-unknown';
    }
  }
</script>

<div class="row-metadata-badges" aria-label={ariaLabel}>
  {#each items as item}
    <span
      class={`row-metadata-badge row-metadata-badge--${badgeTone(item)}`}
      title={item.label}
    >
      {item.label}
    </span>
  {/each}
</div>

<style>
  .row-metadata-badges {
    display: flex;
    min-width: 0;
    max-width: 100%;
    flex-wrap: wrap;
    align-items: center;
    gap: calc(var(--spacing) * 1);
    padding-top: calc(var(--spacing) * 0.75);
  }

  .row-metadata-badge {
    --row-metadata-accent: var(--color-base-content);

    display: inline-flex;
    min-width: 0;
    max-width: 100%;
    align-items: center;
    border: var(--border) solid
      color-mix(in oklab, var(--row-metadata-accent) 46%, transparent);
    border-radius: var(--radius-field);
    background:
      linear-gradient(
        135deg,
        color-mix(in oklab, var(--row-metadata-accent) 20%, transparent),
        color-mix(in oklab, var(--row-metadata-accent) 8%, transparent)
      ),
      var(--color-base-100);
    color: color-mix(in oklab, var(--row-metadata-accent) 78%, white 22%);
    font-size: 0.6875rem;
    font-weight: 650;
    line-height: 1;
    overflow: hidden;
    padding: calc(var(--spacing) * 0.75) calc(var(--spacing) * 1.25);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row-metadata-badge--year {
    --row-metadata-accent: var(--ctp-mocha-blue);
  }

  .row-metadata-badge--season {
    --row-metadata-accent: var(--ctp-mocha-sky);
  }

  .row-metadata-badge--anime-type {
    --row-metadata-accent: var(--ctp-mocha-mauve);
  }

  .row-metadata-badge--status-finished {
    --row-metadata-accent: var(--color-success);
  }

  .row-metadata-badge--status-ongoing {
    --row-metadata-accent: var(--color-warning);
  }

  .row-metadata-badge--status-upcoming {
    --row-metadata-accent: var(--color-info);
  }

  .row-metadata-badge--status-unknown {
    --row-metadata-accent: var(--color-base-content);
  }
</style>
