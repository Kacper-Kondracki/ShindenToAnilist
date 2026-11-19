import type {
  MatchListResult,
  ShindenListViews,
  WireNumber
} from '../../domain/anime';

export function buildMatchViews(
  matchResult: MatchListResult,
  allEntryIds: WireNumber[]
): ShindenListViews {
  const unmatched = new Set<WireNumber>();
  const review = new Set<WireNumber>();
  const automatic = new Set<WireNumber>();

  for (const entry of matchResult.entries) {
    if (entry.result.winner !== null) {
      automatic.add(entry.shindenId);
    } else if (entry.result.top.length > 0) {
      review.add(entry.shindenId);
    } else {
      unmatched.add(entry.shindenId);
    }
  }

  const unmatchedIds = allEntryIds.filter((entryId) => unmatched.has(entryId));
  const reviewIds = allEntryIds.filter((entryId) => review.has(entryId));
  const automaticIds = allEntryIds.filter((entryId) => automatic.has(entryId));

  return {
    manual: [...unmatchedIds, ...reviewIds],
    automatic: automaticIds,
    ignored: [],
    all: [...unmatchedIds, ...reviewIds, ...automaticIds]
  };
}
