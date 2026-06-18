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
  import ContextMenu from './ContextMenu.svelte';
  import EntryRow from './EntryRow.svelte';
  import RowMetadataBadges from './RowMetadataBadges.svelte';
  import {
    copyText,
    openExternalUrl,
    shindenEntryUrl
  } from './contextMenuActions';
  import type { ContextMenuItem } from './contextMenuState.svelte';
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
    onReset,
    canReset,
    onToggleIgnored,
    showIndicator = true,
    rounded = false,
    compact = false,
    striped = false,
    separated = true
  }: {
    entry: ShindenEntry;
    matchStatus: AnimeMatchStatus;
    isSelected: boolean;
    onSelect: () => void;
    onReset: () => void;
    canReset: boolean;
    onToggleIgnored: () => void;
    showIndicator?: boolean;
    rounded?: boolean;
    compact?: boolean;
    striped?: boolean;
    separated?: boolean;
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
  let contextMenuItems = $derived<ContextMenuItem[]>([
    {
      id: 'copy-title',
      label: 'Kopiuj tytuł',
      icon: 'icon-[lucide--copy]',
      onSelect: () => copyText(entry.title)
    },
    {
      id: 'open-shinden',
      label: 'Otwórz stronę Shinden',
      icon: 'icon-[lucide--external-link]',
      onSelect: () => openExternalUrl(shindenEntryUrl(entry.id))
    },
    {
      id: 'reset-entry',
      label: 'Resetuj wpis',
      icon: 'icon-[lucide--rotate-ccw]',
      dividerBefore: true,
      disabled: !canReset,
      onSelect: onReset
    },
    {
      id: 'toggle-ignored',
      label:
        matchStatus === 'ignored' ? 'Przestań ignorować wpis' : 'Ignoruj wpis',
      icon: 'icon-[lucide--eye-off]',
      checked: matchStatus === 'ignored',
      onSelect: onToggleIgnored
    }
  ]);
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

<ContextMenu items={contextMenuItems}>
  <EntryRow
    tone={rowTone}
    {isSelected}
    ariaLabel={`${entry.title}: ${matchStatusLabels[matchStatus]}`}
    title={entry.title}
    class="h-0"
    {showIndicator}
    {rounded}
    {compact}
    {striped}
    {separated}
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
</ContextMenu>
