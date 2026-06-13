<script lang="ts">
  import type { ShindenEntry } from '../../domain/anime';
  import {
    formatEpisodeCount,
    formatPremiereYear,
    translateAnimeStatus,
    translateAnimeType,
    translateWatchStatus
  } from '../../domain/animeView';
  import EntryRow from './EntryRow.svelte';
  import RowMetadataBadges from './RowMetadataBadges.svelte';
  import type { RowMetadataBadge } from './RowMetadataBadges.svelte';

  export type AnimeMatchStatus = 'matched' | 'review' | 'unmatched';

  let {
    entry,
    matchStatus,
    isSelected,
    onSelect,
    showIndicator = true,
    rounded = false,
    compact = false
  }: {
    entry: ShindenEntry;
    matchStatus: AnimeMatchStatus;
    isSelected: boolean;
    onSelect: () => void;
    showIndicator?: boolean;
    rounded?: boolean;
    compact?: boolean;
  } = $props();

  const matchStatusLabels: Record<AnimeMatchStatus, string> = {
    matched: 'Dobrano automatycznie',
    review: 'Znaleziono kandydatów do sprawdzenia',
    unmatched: 'Nie znaleziono kandydatów'
  };

  let episodeCountText = $derived(formatEpisodeCount(entry.episodes));
  let episodeProgressText = $derived(
    episodeCountText === 'Brak'
      ? 'Brak odc.'
      : `${entry.watchedEpisodes} / ${episodeCountText} odc.`
  );
  let metadataItems = $derived<RowMetadataBadge[]>([
    { label: formatPremiereYear(entry.premiereDate), tone: 'year' },
    { label: translateAnimeType(entry.animeType), tone: 'anime-type' },
    {
      label: translateAnimeStatus(entry.animeStatus),
      tone: 'status',
      animeStatus: entry.animeStatus
    }
  ]);
</script>

<EntryRow
  tone={matchStatus}
  {isSelected}
  ariaLabel={`${entry.title}: ${matchStatusLabels[matchStatus]}`}
  title={entry.title}
  class="h-0"
  {showIndicator}
  {rounded}
  {compact}
  {onSelect}
>
  <h2 class="truncate text-sm font-semibold">{entry.title}</h2>
  <RowMetadataBadges items={metadataItems} />

  {#snippet meta()}
    <span class="text-xs font-semibold">
      {translateWatchStatus(entry.watchStatus)}
    </span>
    <span class="text-xs text-muted">{episodeProgressText}</span>
  {/snippet}
</EntryRow>
