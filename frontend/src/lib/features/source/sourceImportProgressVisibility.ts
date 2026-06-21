import type { Provider } from '../../config/providers';
import type { SourceImportProgress } from '../../domain/anime';
import { SourceFetchPhase } from '../../gen/shinden_to_anilist/v1/source_pb';

export const animeZoneSourceImportProgressDebounceMs = 250;

export function isReadySourceImportProgress(
  provider: Provider | string,
  progress: SourceImportProgress | null,
  options: { animeZoneListProgressDebounced?: boolean } = {}
) {
  if (progress === null) {
    return false;
  }

  if (progress.total > 0 || progress.phase !== SourceFetchPhase.FETCHING_LIST) {
    return true;
  }

  return (
    provider === 'anime-zone' && options.animeZoneListProgressDebounced === true
  );
}

export function shouldDebounceAnimeZoneListProgress(
  provider: Provider | string,
  progress: SourceImportProgress | null
) {
  return (
    provider === 'anime-zone' &&
    progress !== null &&
    progress.total === 0 &&
    progress.phase === SourceFetchPhase.FETCHING_LIST
  );
}
