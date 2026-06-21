import { ensureDatabase, getDatabaseFull } from '../../api/appService';
import { toUserFacingErrorMessage } from '../../api/runtime';
import type { LoadedAnimeData } from '../../data/loadedAnimeData.svelte';
import type { DatabaseState } from '../../domain/anime';

const databaseRetryDelays = [0, 500, 1500] as const;

export async function initializeDatabaseState(
  animeData: LoadedAnimeData
): Promise<DatabaseState> {
  let lastError: unknown = null;

  for (const [attempt, delayMs] of databaseRetryDelays.entries()) {
    if (delayMs > 0) {
      await delay(delayMs);
    }

    try {
      const info = await ensureDatabase();
      const databaseFull = await getDatabaseFull();
      if (databaseFull.databaseVersion !== info.databaseVersion) {
        throw new Error(
          'Baza danych zmieniła się podczas wczytywania. Spróbuj ponownie.'
        );
      }

      animeData.replaceDatabaseFull(databaseFull);
      return { status: 'ready', info };
    } catch (error) {
      lastError = error;
      if (attempt === databaseRetryDelays.length - 1) {
        break;
      }
    }
  }

  return { status: 'error', message: errorMessage(lastError) };
}

export function errorMessage(error: unknown) {
  return toUserFacingErrorMessage(error, 'Nie udało się wczytać bazy danych');
}

function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
