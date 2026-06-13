<script lang="ts">
  import type { DatabaseEntry } from '../../domain/anime';
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
  import type { EntryRowTone } from './EntryRow.svelte';

  let {
    entry,
    scoreLabel,
    isSelected,
    onSelect,
    showIndicator = true,
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
    rounded?: boolean;
    compact?: boolean;
    softWarning?: boolean;
    tone?: EntryRowTone | null;
  } = $props();

  let tone = $derived<EntryRowTone>(
    explicitTone ?? (isSelected ? 'matched' : 'neutral')
  );
  let metadataItems = $derived<RowMetadataBadge[]>([
    { label: formatYear(entry.year), tone: 'year' },
    { label: translateSeason(entry.season), tone: 'season' },
    { label: translateAnimeType(entry.animeType), tone: 'anime-type' },
    {
      label: translateAnimeStatus(entry.status),
      tone: 'status',
      animeStatus: entry.status
    }
  ]);
</script>

<EntryRow
  {tone}
  {isSelected}
  ariaLabel={`${entry.title}: ${scoreLabel}`}
  title={entry.title}
  {showIndicator}
  {rounded}
  {compact}
  {softWarning}
  {onSelect}
>
  <h3 class="truncate text-sm font-semibold">{entry.title}</h3>
  <RowMetadataBadges items={metadataItems} />

  {#snippet meta()}
    <span class="text-xs font-semibold">{scoreLabel}</span>
    <span class="text-xs text-muted">
      {formatEpisodeCount(entry.episodes)} odc.
    </span>
  {/snippet}
</EntryRow>
