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
  import type { EntryRowTone } from './EntryRow.svelte';

  let {
    entry,
    scoreLabel,
    isSelected,
    onSelect,
    showIndicator = true,
    rounded = false,
    compact = false
  }: {
    entry: DatabaseEntry;
    scoreLabel: string;
    isSelected: boolean;
    onSelect: () => void;
    showIndicator?: boolean;
    rounded?: boolean;
    compact?: boolean;
  } = $props();

  let tone = $derived<EntryRowTone>(isSelected ? 'matched' : 'neutral');
</script>

<EntryRow
  {tone}
  {isSelected}
  ariaLabel={`${entry.title}: ${scoreLabel}`}
  title={entry.title}
  {showIndicator}
  {rounded}
  {compact}
  {onSelect}
>
  <h3 class="truncate text-sm font-semibold">{entry.title}</h3>
  <p class="truncate text-xs text-muted">
    {formatYear(entry.year)} · {translateSeason(entry.season)} · {translateAnimeType(
      entry.animeType
    )}
    · {translateAnimeStatus(entry.status)}
  </p>

  {#snippet meta()}
    <span class="text-xs font-semibold">{scoreLabel}</span>
    <span class="text-xs text-muted">
      {formatEpisodeCount(entry.episodes)} odc.
    </span>
  {/snippet}
</EntryRow>
