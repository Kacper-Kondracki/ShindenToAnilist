<script lang="ts">
  import type { DatabaseEntry } from '../../domain/anime';
  import {
    AnimeStatus,
    AnimeType,
    Season
  } from '../../gen/shinden_to_anilist/v1/common_pb';
  import {
    formatEpisodeCount,
    formatYear,
    translateAnimeStatus,
    translateAnimeType,
    translateSeason
  } from '../../domain/animeView';
  import EntryRow from './EntryRow.svelte';
  import RowMetadataBadges from './RowMetadataBadges.svelte';
  import type { RowMetadataBadge } from './RowMetadataBadges.svelte';
  import type {
    EntryRowIndicator,
    EntryRowTone
  } from './EntryRow.svelte';

  let {
    entry,
    scoreLabel,
    isSelected,
    onSelect,
    showIndicator = true,
    indicator = 'bar',
    rounded = false,
    compact = false,
    softWarning = false,
    tone: explicitTone = null
  }: {
    entry: DatabaseEntry;
    scoreLabel: string;
    isSelected: boolean;
    onSelect: () => void;
    showIndicator?: boolean;
    indicator?: EntryRowIndicator;
    rounded?: boolean;
    compact?: boolean;
    softWarning?: boolean;
    tone?: EntryRowTone | null;
  } = $props();

  let tone = $derived<EntryRowTone>(
    explicitTone ?? (isSelected ? 'matched' : 'neutral')
  );
  let metadataItems = $derived.by<RowMetadataBadge[]>(() => {
    const items: RowMetadataBadge[] = [];

    if (entry.year !== null) {
      items.push({
        label: formatYear(entry.year),
        tone: 'year'
      });
    }

    if (
      entry.season !== Season.UNSPECIFIED &&
      entry.season !== Season.UNKNOWN
    ) {
      items.push({
        label: translateSeason(entry.season),
        tone: 'season'
      });
    }

    if (
      entry.animeType !== AnimeType.UNSPECIFIED &&
      entry.animeType !== AnimeType.UNKNOWN
    ) {
      items.push({
        label: translateAnimeType(entry.animeType),
        tone: 'anime-type'
      });
    }

    if (
      entry.status !== AnimeStatus.UNSPECIFIED &&
      entry.status !== AnimeStatus.UNKNOWN
    ) {
      items.push({
        label: translateAnimeStatus(entry.status),
        tone: 'status',
        animeStatus: entry.status
      });
    }

    return items;
  });
</script>

<EntryRow
  {tone}
  {isSelected}
  ariaLabel={`${entry.title}: ${scoreLabel}`}
  title={entry.title}
  {showIndicator}
  {indicator}
  {rounded}
  {compact}
  {softWarning}
  {onSelect}
>
  <h3 class="truncate text-sm font-semibold">{entry.title}</h3>
  {#if metadataItems.length > 0}
    <RowMetadataBadges items={metadataItems} />
  {/if}

  {#snippet meta()}
    <span class="text-xs font-semibold">{scoreLabel}</span>
    <span class="text-xs text-muted">
      {formatEpisodeCount(entry.episodes)} odc.
    </span>
  {/snippet}
</EntryRow>
