import { fuzzyMatch } from '../../api/appService';
import { toUserFacingErrorMessage } from '../../api/runtime';
import type {
  DatabaseEntry,
  MatchResult,
  ScoredCandidate,
  ShindenEntry,
  WireNumber
} from '../../domain/anime';
import { wireNumberEquals } from '../../domain/anime';

type MatchSelectorControllerInput = {
  getSelectedEntry: () => ShindenEntry;
  getRememberedQuery: () => string;
  getDatabaseEntry: (entryId: WireNumber) => DatabaseEntry | null;
  getAutomaticMatchResult: () => MatchResult | null;
  getInitialSearch: () => MatchSelectorInitialSearch | null;
  getWinnerClaimsByDatabaseId: () => ReadonlyMap<
    WireNumber,
    readonly WireNumber[]
  >;
  setRememberedQuery: (shindenId: WireNumber, query: string) => void;
  resetRememberedQuery: (shindenId: WireNumber) => void;
  setManualOverride: (shindenId: WireNumber, databaseId: WireNumber) => void;
  setIgnored: (shindenId: WireNumber) => void;
  clearManualOverride: (shindenId: WireNumber) => void;
};

export type MatchSelectorResult = ScoredCandidate & {
  entry: DatabaseEntry;
};

export type MatchSelectorController = ReturnType<
  typeof createMatchSelectorController
>;

export type MatchSelectorInitialSearch =
  | { status: 'idle'; shindenId: WireNumber; query: string }
  | {
      status: 'ready';
      shindenId: WireNumber;
      query: string;
      result: MatchResult;
    }
  | {
      status: 'error';
      shindenId: WireNumber;
      query: string;
      message: string;
    };

type MatchSearchState =
  | { status: 'idle' }
  | { status: 'ready'; query: string; result: MatchResult }
  | { status: 'error'; query: string; message: string };

export function createMatchSelectorController(
  input: MatchSelectorControllerInput
) {
  const initialSelectedEntry = input.getSelectedEntry();
  const initialQuery = input.getRememberedQuery();
  const initialSearchState = getInitialSearchState(
    initialSelectedEntry,
    initialQuery,
    input.getInitialSearch()
  );

  let query = $state(initialQuery);
  let searchState = $state<MatchSearchState>(initialSearchState);
  const automaticCandidateOrders = new WeakMap<MatchResult, WireNumber[]>();
  const searchCandidateOrders = new WeakMap<MatchResult, WireNumber[]>();
  let requestId = 0;

  let conflictingWinnerIds = $derived.by(() => {
    const selectedEntryId = input.getSelectedEntry().id;
    const conflicts = new Set<WireNumber>();

    for (const [
      databaseId,
      shindenIds
    ] of input.getWinnerClaimsByDatabaseId()) {
      if (
        shindenIds.some(
          (shindenId) => !wireNumberEquals(shindenId, selectedEntryId)
        )
      ) {
        conflicts.add(databaseId);
      }
    }

    return conflicts;
  });
  let automaticResults = $derived.by((): MatchSelectorResult[] => {
    const matchResult = input.getAutomaticMatchResult();

    if (matchResult === null) {
      return [];
    }

    return orderCandidatesByInitialAvailability(
      resolveCandidates(automaticCandidates(matchResult)),
      matchResult,
      automaticCandidateOrders
    );
  });
  let searchResults = $derived.by((): MatchSelectorResult[] => {
    if (searchState.status !== 'ready') {
      return [];
    }

    return orderCandidatesByInitialAvailability(
      resolveCandidates(searchState.result.items),
      searchState.result,
      searchCandidateOrders
    );
  });
  let hasResults = $derived(
    automaticResults.length > 0 || searchResults.length > 0
  );
  let isSearchCurrent = $derived.by(() => {
    const currentQuery = resolveSearchQuery(query);

    if (searchState.status === 'idle') {
      return currentQuery.length === 0;
    }

    return searchState.query === currentQuery;
  });
  let errorMessage = $derived(
    searchState.status === 'error' ? searchState.message : null
  );

  function resolveCandidates(
    candidates: ScoredCandidate[],
    excludedIds = new Set<WireNumber>()
  ) {
    const resolvedItems: MatchSelectorResult[] = [];
    const resolvedIds = new Set(excludedIds);

    for (const item of candidates) {
      if ([...resolvedIds].some((id) => wireNumberEquals(id, item.id))) {
        continue;
      }

      const entry = input.getDatabaseEntry(item.id);
      if (entry !== null) {
        resolvedItems.push({ ...item, entry });
        resolvedIds.add(item.id);
      }
    }

    return resolvedItems;
  }

  function orderCandidatesByInitialAvailability(
    candidates: MatchSelectorResult[],
    source: MatchResult,
    candidateOrders: WeakMap<MatchResult, WireNumber[]>
  ) {
    if (candidates.length === 0) {
      return [];
    }

    const cachedOrder = candidateOrders.get(source);

    if (cachedOrder !== undefined) {
      return orderCandidatesByIds(candidates, cachedOrder);
    }

    const available: MatchSelectorResult[] = [];
    const alreadyUsed: MatchSelectorResult[] = [];

    for (const candidate of candidates) {
      if (conflictingWinnerIds.has(candidate.id)) {
        alreadyUsed.push(candidate);
      } else {
        available.push(candidate);
      }
    }

    const orderedCandidates = [...available, ...alreadyUsed];
    candidateOrders.set(
      source,
      orderedCandidates.map((candidate) => candidate.id)
    );

    return orderedCandidates;
  }

  if (initialSearchState.status === 'idle') {
    search(resolveSearchQuery(query));
  }

  function resolveSearchQuery(inputQuery: string) {
    return resolveSearchQueryForEntry(input.getSelectedEntry(), inputQuery);
  }

  function updateQuery(nextQuery: string) {
    const selectedEntryId = input.getSelectedEntry().id;

    query = nextQuery;
    input.setRememberedQuery(selectedEntryId, nextQuery);
    search(resolveSearchQuery(nextQuery));
  }

  function resetQuery() {
    const selectedEntry = input.getSelectedEntry();
    const nextSearchState = getInitialSearchState(
      selectedEntry,
      '',
      input.getInitialSearch()
    );

    query = '';
    input.resetRememberedQuery(selectedEntry.id);
    requestId += 1;
    searchState = nextSearchState;

    if (nextSearchState.status === 'idle') {
      search(resolveSearchQuery(''));
    }
  }

  function search(nextQuery: string) {
    const currentQuery = nextQuery.trim();
    const currentRequestId = ++requestId;

    if (currentQuery.length === 0) {
      searchState = { status: 'idle' };
      return;
    }

    void fuzzyMatch(
      currentQuery,
      { limit: 50 },
      input.getSelectedEntry().id
    ).then(
      (response) => {
        if (currentRequestId !== requestId) {
          return;
        }

        searchState = {
          status: 'ready',
          query: currentQuery,
          result: response.result
        };
      },
      (error: unknown) => {
        if (currentRequestId !== requestId) {
          return;
        }

        searchState = {
          status: 'error',
          query: currentQuery,
          message: errorToMessage(error)
        };
      }
    );
  }

  function applyManualOverride(databaseId: WireNumber) {
    input.setManualOverride(input.getSelectedEntry().id, databaseId);
  }

  function clearManualOverride() {
    input.clearManualOverride(input.getSelectedEntry().id);
  }

  function applyIgnore() {
    input.setIgnored(input.getSelectedEntry().id);
  }

  return {
    get query() {
      return query;
    },
    get hasRememberedQuery() {
      return query.length > 0;
    },
    get automaticResults() {
      return automaticResults;
    },
    get searchResults() {
      return searchResults;
    },
    get hasResults() {
      return hasResults;
    },
    get isSearchCurrent() {
      return isSearchCurrent;
    },
    get conflictingWinnerIds() {
      return conflictingWinnerIds;
    },
    get errorMessage() {
      return errorMessage;
    },
    updateQuery,
    resetQuery,
    applyManualOverride,
    applyIgnore,
    clearManualOverride
  };
}

export async function loadInitialMatchSelectorSearch(
  selectedEntry: ShindenEntry,
  inputQuery = ''
): Promise<MatchSelectorInitialSearch> {
  const query = resolveSearchQueryForEntry(selectedEntry, inputQuery);

  if (query.length === 0) {
    return {
      status: 'idle',
      shindenId: selectedEntry.id,
      query
    };
  }

  try {
    const response = await fuzzyMatch(query, { limit: 50 }, selectedEntry.id);

    return {
      status: 'ready',
      shindenId: selectedEntry.id,
      query,
      result: response.result
    };
  } catch (error) {
    return {
      status: 'error',
      shindenId: selectedEntry.id,
      query,
      message: errorToMessage(error)
    };
  }
}

function getInitialSearchState(
  selectedEntry: ShindenEntry,
  inputQuery: string,
  initialSearch: MatchSelectorInitialSearch | null
): MatchSearchState {
  const query = resolveSearchQueryForEntry(selectedEntry, inputQuery);

  if (
    initialSearch === null ||
    !wireNumberEquals(initialSearch.shindenId, selectedEntry.id) ||
    initialSearch.query !== query
  ) {
    return { status: 'idle' };
  }

  if (initialSearch.status === 'ready') {
    return {
      status: 'ready',
      query,
      result: initialSearch.result
    };
  }

  if (initialSearch.status === 'error') {
    return {
      status: 'error',
      query,
      message: initialSearch.message
    };
  }

  return { status: 'idle' };
}

function resolveSearchQueryForEntry(
  selectedEntry: ShindenEntry,
  inputQuery: string
) {
  const explicitQuery = inputQuery.trim();

  return explicitQuery.length > 0 ? explicitQuery : selectedEntry.title.trim();
}

function automaticCandidates(matchResult: MatchResult | null) {
  if (matchResult === null) {
    return [];
  }

  const candidates = [...matchResult.top];
  const winner = matchResult.winner;
  if (
    winner !== null &&
    !candidates.some((candidate) => wireNumberEquals(candidate.id, winner.id))
  ) {
    candidates.unshift(winner);
  }

  return uniqueCandidates(candidates);
}

function uniqueCandidates(candidates: ScoredCandidate[]) {
  const usedIds = new Set<string>();
  const unique: ScoredCandidate[] = [];

  for (const candidate of candidates) {
    const candidateKey = candidate.id.toString();
    if (!usedIds.has(candidateKey)) {
      usedIds.add(candidateKey);
      unique.push(candidate);
    }
  }

  return unique;
}

function orderCandidatesByIds(
  candidates: MatchSelectorResult[],
  orderedIds: WireNumber[]
) {
  const candidatesById = new Map(
    candidates.map((candidate) => [candidate.id.toString(), candidate])
  );
  const orderedCandidates: MatchSelectorResult[] = [];

  for (const id of orderedIds) {
    const candidate = candidatesById.get(id.toString());

    if (candidate !== undefined) {
      orderedCandidates.push(candidate);
      candidatesById.delete(id.toString());
    }
  }

  return [...orderedCandidates, ...candidatesById.values()];
}

function errorToMessage(error: unknown) {
  return toUserFacingErrorMessage(error, 'Nie udało się wyszukać dopasowań');
}
