import type { EntryStore } from "../../data/entryStore.svelte";

type AnimeRowControllerInput = {
  getEntryStore: () => EntryStore;
  getEntryId: () => number;
};

export type AnimeRowController = ReturnType<typeof createAnimeRowController>;

export function createAnimeRowController(input: AnimeRowControllerInput) {
  $effect(() => {
    return input.getEntryStore().retainShindenEntry(input.getEntryId());
  });

  return {};
}
