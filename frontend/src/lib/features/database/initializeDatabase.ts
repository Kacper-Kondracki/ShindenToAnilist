import { reportFailure } from '../../api/diagnostics';
import { ensureDatabase, getDatabaseFull } from '../../api/appService';
import { AppFailure, ErrorKind, userErrorMessage } from '../../api/errors';
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
        throw new AppFailure(ErrorKind.DATABASE_CHANGED);
      }

      animeData.replaceDatabaseFull(databaseFull);
      return { status: 'ready', info };
    } catch (error) {
      lastError = error;
      if (!canRetry(error) || attempt === databaseRetryDelays.length - 1) {
        break;
      }
    }
  }

  reportFailure('initialize_database', lastError);
  return {
    status: 'error',
    message: userErrorMessage(
      lastError,
      'Nie udało się wczytać bazy anime. Spróbuj ponownie.'
    )
  };
}

function canRetry(error: unknown): boolean {
  if (!(error instanceof AppFailure)) return false;

  return (
    error.kind === ErrorKind.DATABASE_CHANGED ||
    error.kind === ErrorKind.DEADLINE_EXCEEDED ||
    (error.kind === ErrorKind.DATABASE_HTTP &&
      (error.httpStatus === undefined ||
        error.httpStatus === 429 ||
        error.httpStatus >= 500))
  );
}

function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
