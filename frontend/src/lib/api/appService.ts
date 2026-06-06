import { AppService } from "../../../bindings/shindentoanilist";
import type { DatabaseInfo, ShindenList } from "../domain/anime";

export async function ensureDatabase() {
  return (await AppService.EnsureDatabase()) as DatabaseInfo;
}

export async function loadShindenList(userId: number) {
  return (await AppService.LoadShindenList(userId)) as ShindenList;
}
