import { ensureDatabase } from "../../api/appService";
import type { DatabaseState } from "../../domain/anime";

const databaseRetryDelays = [0, 500, 1500] as const;

export async function initializeDatabaseState(): Promise<DatabaseState> {
  let lastError: unknown = null;

  for (const [attempt, delayMs] of databaseRetryDelays.entries()) {
    if (delayMs > 0) {
      await delay(delayMs);
    }

    try {
      const info = await ensureDatabase();
      return { status: "ready", info };
    } catch (error) {
      lastError = error;
      if (attempt === databaseRetryDelays.length - 1) {
        break;
      }
    }
  }

  return { status: "error", message: errorMessage(lastError) };
}

export function errorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === "string") {
    return error;
  }

  return "Nie udało się wczytać bazy danych";
}

function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
