export const animeListTabs = [
  {
    id: "manual",
    label: "Ręczna interwencja",
  },
  {
    id: "automatic",
    label: "Automatyczne",
  },
  {
    id: "all",
    label: "Wszystko",
  },
] as const;

export type AnimeListTabId = (typeof animeListTabs)[number]["id"];
