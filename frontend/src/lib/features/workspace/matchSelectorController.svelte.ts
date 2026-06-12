import { fuzzyMatch } from '../../api/appService';
import type {
  DatabaseEntry,
  MatchResult,
  ScoredCandidate,
  ShindenEntry
} from '../../domain/anime';

type MatchSelectorControllerInput = {
  getSelectedEntry: () => ShindenEntry | null;
  getDatabaseEntry: (entryId: number) => DatabaseEntry | null;
  setManualOverride: (shindenId: number, databaseId: number) => void;
  clearManualOverride: (shindenId: number) => void;
};

export type MatchSelectorResult = ScoredCandidate & {
  entry: DatabaseEntry;
};

export type MatchSelectorController = ReturnType<
  typeof createMatchSelectorController
>;

export function createMatchSelectorController(
  input: MatchSelectorControllerInput
) {
  let query = $state('');
  let result = $state<MatchResult | null>(null);
  let errorMessage = $state<string | null>(null);
  let lastSelectedEntryId = $state<number | null>(null);
  let requestId = 0;

  let results = $derived.by((): MatchSelectorResult[] => {
    const items = result?.items ?? [];
    const resolvedItems: MatchSelectorResult[] = [];

    for (const item of items) {
      const entry = input.getDatabaseEntry(item.id);
      if (entry !== null) {
        resolvedItems.push({ ...item, entry });
      }
    }

    return resolvedItems;
  });

  $effect(() => {
    const selectedEntry = input.getSelectedEntry();
    const selectedEntryId = selectedEntry?.id ?? null;

    if (selectedEntryId === lastSelectedEntryId) {
      return;
    }

    lastSelectedEntryId = selectedEntryId;
    query = selectedEntry?.title ?? '';
  });

  $effect(() => {
    const currentQuery = query.trim();
    const currentRequestId = ++requestId;

    errorMessage = null;

    if (currentQuery.length === 0) {
      result = null;
      return;
    }

    void fuzzyMatch(currentQuery, { limit: 20 }).then(
      (response) => {
        if (currentRequestId !== requestId) {
          return;
        }

        result = response.result;
      },
      (error: unknown) => {
        if (currentRequestId !== requestId) {
          return;
        }

        result = null;
        errorMessage = errorToMessage(error);
      }
    );
  });

  function updateQuery(nextQuery: string) {
    query = nextQuery;
  }

  function applyManualOverride(databaseId: number) {
    const selectedEntry = input.getSelectedEntry();
    if (selectedEntry === null) {
      return;
    }

    input.setManualOverride(selectedEntry.id, databaseId);
  }

  function clearManualOverride() {
    const selectedEntry = input.getSelectedEntry();
    if (selectedEntry === null) {
      return;
    }

    input.clearManualOverride(selectedEntry.id);
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

function errorToMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === 'string') {
    return error;
  }

  return 'Nie udało się wyszukać dopasowań';
}
