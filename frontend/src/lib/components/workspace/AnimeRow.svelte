<script lang="ts">
  import type { ShindenEntry } from '../../domain/anime';
  import {
    AnimeStatus,
    AnimeType
  } from '../../gen/shinden_to_anilist/v1/common_pb';
  import {
    formatEpisodeCount,
    formatPremiereDate,
    translateAnimeStatus,
    translateAnimeType,
    translateWatchStatus
  } from '../../domain/animeView';
  import EntryRow from './EntryRow.svelte';
  import RowMetadataBadges from './RowMetadataBadges.svelte';
  import type { EntryRowTone } from './EntryRow.svelte';
  import type { RowMetadataBadge } from './RowMetadataBadges.svelte';

  export type AnimeMatchStatus =
    | 'matched'
    | 'review'
    | 'unmatched'
    | 'manual'
    | 'suppressed'
    | 'ignored';

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
    unmatched: 'Nie znaleziono kandydatów',
    manual: 'Dobrano ręcznie',
    suppressed: 'Automatyczne dopasowanie zostało zastąpione',
    ignored: 'Ignorowany'
  };
  let rowTone = $derived<EntryRowTone>(
    matchStatus === 'manual' ? 'info' : matchStatus
  );

  let episodeCountText = $derived(formatEpisodeCount(entry.episodes));
  let episodeProgressText = $derived(
    episodeCountText === 'Brak'
      ? 'Brak odc.'
      : `${entry.watchedEpisodes} / ${episodeCountText} odc.`
  );
  let metadataItems = $derived.by<RowMetadataBadge[]>(() => {
    const items: RowMetadataBadge[] = [];

    if (entry.premiereDate) {
      items.push({
        label: formatPremiereDate(entry.premiereDate),
        tone: 'year'
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
      entry.animeStatus !== AnimeStatus.UNSPECIFIED &&
      entry.animeStatus !== AnimeStatus.UNKNOWN
    ) {
      items.push({
        label: translateAnimeStatus(entry.animeStatus),
        tone: 'status',
        animeStatus: entry.animeStatus
      });
    }

    return items;
  });
</script>

<EntryRow
  tone={rowTone}
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
  {#if metadataItems.length > 0}
    <RowMetadataBadges items={metadataItems} />
  {/if}

  {#snippet meta()}
    <span class="text-xs font-semibold">
      {translateWatchStatus(entry.watchStatus)}
    </span>
    <span class="text-xs text-muted">{episodeProgressText}</span>
  {/snippet}
</EntryRow>
