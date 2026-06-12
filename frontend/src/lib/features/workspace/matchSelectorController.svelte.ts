import { fuzzyMatch } from '../../api/appService';
import type {
  DatabaseEntry,
  MatchResult,
  ScoredCandidate,
  ShindenEntry
} from '../../domain/anime';

type MatchSelectorControllerInput = {
  getSelectedEntry: () => ShindenEntry;
  getDatabaseEntry: (entryId: number) => DatabaseEntry | null;
  getInitialSearch: () => MatchSelectorInitialSearch | null;
  setManualOverride: (shindenId: number, databaseId: number) => void;
  clearManualOverride: (shindenId: number) => void;
};

export type MatchSelectorResult = ScoredCandidate & {
  entry: DatabaseEntry;
};

export type MatchSelectorController = ReturnType<
  typeof createMatchSelectorController
>;

export type MatchSelectorInitialSearch =
  | { status: 'idle'; shindenId: number; query: string }
  | {
      status: 'ready';
      shindenId: number;
      query: string;
      result: MatchResult;
    }
  | {
      status: 'error';
      shindenId: number;
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
  const initialSearchState = getInitialSearchState(
    initialSelectedEntry,
    input.getInitialSearch()
  );

  let query = $state(initialSelectedEntry.title);
  let searchState = $state<MatchSearchState>(initialSearchState);
  let requestId = 0;

  let results = $derived.by((): MatchSelectorResult[] => {
    const items =
      searchState.status === 'ready' ? searchState.result.items : [];
    const resolvedItems: MatchSelectorResult[] = [];

    for (const item of items) {
      const entry = input.getDatabaseEntry(item.id);
      if (entry !== null) {
        resolvedItems.push({ ...item, entry });
      }
    }

    return resolvedItems;
  });
  let errorMessage = $derived(
    searchState.status === 'error' ? searchState.message : null
  );

  if (initialSearchState.status === 'idle') {
    search(query);
  }

  function updateQuery(nextQuery: string) {
    query = nextQuery;
    search(nextQuery);
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

  function applyManualOverride(databaseId: number) {
    input.setManualOverride(input.getSelectedEntry().id, databaseId);
  }

  function clearManualOverride() {
    input.clearManualOverride(input.getSelectedEntry().id);
  }

  return {
    get query() {
      return query;
    },
    get results() {
      return results;
    },
    get errorMessage() {
      return errorMessage;
    },
    updateQuery,
    applyManualOverride,
    clearManualOverride
  };
}

export async function loadInitialMatchSelectorSearch(
  selectedEntry: ShindenEntry
): Promise<MatchSelectorInitialSearch> {
  const query = selectedEntry.title.trim();

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
  initialSearch: MatchSelectorInitialSearch | null
): MatchSearchState {
  const query = selectedEntry.title.trim();

  if (
    initialSearch === null ||
    initialSearch.shindenId !== selectedEntry.id ||
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

function errorToMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === 'string') {
    return error;
  }

  return 'Nie udało się wyszukać dopasowań';
}
