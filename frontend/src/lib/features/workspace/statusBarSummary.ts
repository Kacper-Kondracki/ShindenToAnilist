import type { MatchListResult } from '../../domain/anime';
import { percentageFromRatio } from '../../domain/animeView';
import type { ExportState } from './workspaceController.svelte';

export type WorkspaceStatusSummary = {
  totalCount: number;
  automaticallyMatchedCount: number;
  manuallySelectedCount: number;
  reviewCount: number;
  matchedPercentage: number;
};

export function buildWorkspaceStatusSummary(
  entryIds: number[],
  matchResult: MatchListResult | null,
  manualSelections: Record<number, number>,
  ignoredEntryIds: Record<number, true>,
  displacedAutomaticEntryIds: Record<number, true>
): WorkspaceStatusSummary {
  const automaticallySelectedEntryIds = new Set<number>();
  const resolvedEntryIds = new Set<number>();

  for (const entry of matchResult?.entries ?? []) {
    if (ignoredEntryIds[entry.shindenId] === true) {
      resolvedEntryIds.add(entry.shindenId);
      continue;
    }

    if (manualSelections[entry.shindenId] !== undefined) {
      resolvedEntryIds.add(entry.shindenId);
      continue;
    }

    if (
      displacedAutomaticEntryIds[entry.shindenId] !== true &&
      entry.result.winner !== null
    ) {
      automaticallySelectedEntryIds.add(entry.shindenId);
      resolvedEntryIds.add(entry.shindenId);
    }
  }

  let manuallySelectedCount = 0;

  for (const entryId of entryIds) {
    if (manualSelections[entryId] !== undefined) {
      manuallySelectedCount += 1;
    }
  }

  const totalCount = entryIds.length;
  const automaticallyMatchedCount = automaticallySelectedEntryIds.size;
  const reviewCount = Math.max(0, totalCount - resolvedEntryIds.size);

  return {
    totalCount,
    automaticallyMatchedCount,
    manuallySelectedCount,
    reviewCount,
    matchedPercentage:
      totalCount > 0
        ? percentageFromRatio(
            (automaticallyMatchedCount + manuallySelectedCount) / totalCount
          )
        : 0
  };
}

export function formatDuration(durationMs: number | null) {
  if (durationMs === null) {
    return '--.--s';
  }

  return `${(durationMs / 1000).toFixed(2)}s`;
}

export function exportButtonText(_exportState: ExportState) {
  // if (exportState.status === 'exporting') {
  //   return 'Eksportowanie';
  // }

  // if (exportState.status === 'exported') {
  //   return `Wyeksportowano ${exportState.exportedCount}`;
  // }

  return 'Eksport';
}
