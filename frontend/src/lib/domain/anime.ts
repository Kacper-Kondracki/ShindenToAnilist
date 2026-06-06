import type { Provider } from "../config/providers";

export type DatabaseInfo = {
  lastUpdate: string;
  release: string;
  sha256: string;
  path: string;
  updated: boolean;
};

export type ShindenEntry = {
  id: number;
  coverId: number | null;
  title: string;
  animeStatus: string;
  animeType: string;
  premiereDate: string | null;
  finishDate: string | null;
  episodes: number | null;
  isFavourite: boolean;
  watchStatus: string;
  watchedEpisodes: number;
  score: number | null;
  note: string | null;
  description: string | null;
};

export type ShindenList = {
  entries: ShindenEntry[];
};

export type LoadedUserList = {
  provider: Provider;
  query: string;
  entries: ShindenEntry[];
};

export type DatabaseState =
  | { status: "loading" }
  | { status: "ready"; info: DatabaseInfo }
  | { status: "error"; message: string };

export type UserListRequestState =
  | { status: "idle" }
  | { status: "loading"; provider: Provider; query: string }
  | ({ status: "loaded" } & LoadedUserList)
  | { status: "error"; provider: Provider; query: string; message: string };

export type WorkspaceState =
  | { status: "empty" }
  | ({ status: "active" } & LoadedUserList);
