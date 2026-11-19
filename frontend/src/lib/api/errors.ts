import { ErrorKind } from '../gen/shinden_to_anilist/v1/error_pb';

export { ErrorKind };

export class AppFailure extends Error {
  constructor(
    readonly kind: ErrorKind,
    options?: ErrorOptions,
    readonly httpStatus?: number,
    readonly reported = false
  ) {
    super(ErrorKind[kind] ?? 'Unknown application failure', options);
    this.name = 'AppFailure';
  }
}

export function isFailure(error: unknown, kind: ErrorKind): boolean {
  return error instanceof AppFailure && error.kind === kind;
}

export function isInvalidClearance(error: unknown): boolean {
  return (
    isFailure(error, ErrorKind.CLEARANCE_INVALID) ||
    isFailure(error, ErrorKind.CLEARANCE_EXPIRED)
  );
}

const messages: Partial<Record<ErrorKind, string>> = {
  [ErrorKind.INVALID_SOURCE]: 'Wybierz jedno z obsługiwanych źródeł listy.',
  [ErrorKind.INVALID_SOURCE_USER]:
    'Sprawdź nazwę użytkownika lub adres listy dla wybranego źródła.',
  [ErrorKind.CANCELLED]: 'Operacja została anulowana.',
  [ErrorKind.DEADLINE_EXCEEDED]:
    'Operacja trwała zbyt długo. Spróbuj ponownie za chwilę.',
  [ErrorKind.SERVICE_UNAVAILABLE]:
    'Nie można połączyć się z usługą aplikacji. Uruchom aplikację ponownie.',
  [ErrorKind.CLOUDFLARE_CHALLENGE]:
    'Shinden wymaga weryfikacji. Otwórz okno weryfikacji i wykonaj instrukcje na stronie.',
  [ErrorKind.CLEARANCE_EXPIRED]:
    'Weryfikacja Shindena wygasła. Otwórz okno weryfikacji ponownie.',
  [ErrorKind.CLEARANCE_INVALID]:
    'Shinden nie może użyć zapisanej weryfikacji. Przeprowadź ją ponownie.',
  [ErrorKind.VERIFICATION_WINDOW_FAILED]:
    'Nie udało się przeprowadzić weryfikacji w oknie Shindena. Spróbuj otworzyć je ponownie.',
  [ErrorKind.VERIFICATION_WINDOW_ALREADY_OPEN]:
    'Dokończ weryfikację w otwartym oknie Shindena.',
  [ErrorKind.VERIFICATION_WINDOW_UNAVAILABLE]:
    'Otwórz aplikację komputerową, aby zweryfikować dostęp do Shindena.',
  [ErrorKind.SOURCE_LIST_PRIVATE]:
    'Ta lista jest prywatna. Ustaw ją jako publiczną i spróbuj ponownie.',
  [ErrorKind.SOURCE_LIST_NOT_FOUND]:
    'Nie znaleziono listy. Sprawdź nazwę użytkownika lub adres listy.',
  [ErrorKind.SHINDEN_LIST_NOT_LOADED]:
    'Najpierw wczytaj listę z wybranego źródła.',
  [ErrorKind.SHINDEN_IO]: 'Nie udało się odczytać listy z Shindena.',
  [ErrorKind.SHINDEN_JSON]: 'Shinden zwrócił listę w nierozpoznanym formacie.',
  [ErrorKind.SHINDEN_HTTP]:
    'Nie udało się pobrać listy z Shindena. Spróbuj ponownie za chwilę.',
  [ErrorKind.SHINDEN_API]:
    'Shinden nie udostępnił listy. Sprawdź, czy lista jest publiczna i czy jej adres jest poprawny.',
  [ErrorKind.ANIME_ZONE_IO]: 'Nie udało się odczytać listy z AnimeZone.',
  [ErrorKind.ANIME_ZONE_HTTP]:
    'Nie udało się pobrać listy z AnimeZone. Spróbuj ponownie za chwilę.',
  [ErrorKind.ANIME_ZONE_PARSE]:
    'Nie udało się odczytać listy AnimeZone. Sprawdź, czy lista otwiera się w przeglądarce.',
  [ErrorKind.ANIME_ZONE_RETRY_EXHAUSTED]:
    'AnimeZone nie odpowiedziało poprawnie po kilku próbach. Spróbuj ponownie później.',
  [ErrorKind.OGLADAJ_ANIME_IO]: 'Nie udało się odczytać listy z Oglądaj Anime.',
  [ErrorKind.OGLADAJ_ANIME_HTTP]:
    'Nie udało się pobrać listy z Oglądaj Anime. Spróbuj ponownie za chwilę.',
  [ErrorKind.OGLADAJ_ANIME_PARSE]:
    'Nie udało się odczytać listy Oglądaj Anime. Sprawdź, czy lista otwiera się w przeglądarce.',
  [ErrorKind.OGLADAJ_ANIME_RETRY_EXHAUSTED]:
    'Oglądaj Anime nie odpowiedziało poprawnie po kilku próbach. Spróbuj ponownie później.',
  [ErrorKind.DATABASE_NOT_LOADED]:
    'Baza anime nie jest gotowa. Wczytaj ją ponownie i spróbuj jeszcze raz.',
  [ErrorKind.DATABASE_CHANGED]:
    'Baza anime zmieniła się podczas wczytywania. Spróbuj ponownie.',
  [ErrorKind.SOURCE_LIST_CHANGED]:
    'Lista zmieniła się podczas dopasowywania. Wczytaj ją ponownie.',
  [ErrorKind.DATABASE_IO]:
    'Nie udało się odczytać lub zapisać lokalnej bazy anime. Sprawdź, czy na dysku jest wolne miejsce.',
  [ErrorKind.DATABASE_SIDECAR_IO]:
    'Nie udało się odczytać lub zapisać informacji o lokalnej bazie anime.',
  [ErrorKind.DATABASE_PERMISSION_DENIED]:
    'Aplikacja nie ma dostępu do plików bazy anime. Sprawdź uprawnienia do folderu aplikacji.',
  [ErrorKind.DATABASE_JSON]:
    'Plik bazy anime jest uszkodzony lub ma nieobsługiwany format.',
  [ErrorKind.DATABASE_SIDECAR_JSON]:
    'Informacje o zapisanej bazie anime są uszkodzone.',
  [ErrorKind.DATABASE_HTTP]:
    'Nie udało się pobrać bazy anime. Sprawdź połączenie i spróbuj ponownie.',
  [ErrorKind.DATABASE_EMPTY]: 'Plik bazy anime jest pusty.',
  [ErrorKind.DATABASE_MISSING_RELEASE_ASSET]:
    'W najnowszym wydaniu brakuje pliku bazy anime. Spróbuj ponownie później.',
  [ErrorKind.DATABASE_DIGEST_MISMATCH]:
    'Pobrany plik bazy anime jest niekompletny lub uszkodzony. Spróbuj pobrać go ponownie.',
  [ErrorKind.APP_PATHS_FAILED]:
    'Nie można otworzyć folderu danych aplikacji. Sprawdź uprawnienia do folderu.',
  [ErrorKind.EXPORT_XML]: 'Nie udało się przygotować pliku XML do eksportu.',
  [ErrorKind.EXPORT_OUT_OF_INDEX]:
    'Lista zmieniła się przed eksportem. Wczytaj ją ponownie i sprawdź wybór pozycji.',
  [ErrorKind.EXPORT_IO]:
    'Nie udało się zapisać pliku eksportu. Wybierz inny folder lub sprawdź wolne miejsce na dysku.',
  [ErrorKind.EXPORT_PERMISSION_DENIED]:
    'Brak uprawnień do zapisu w wybranym folderze. Wybierz inne miejsce.',
  [ErrorKind.EXPORT_DIALOG_FAILED]:
    'Nie udało się otworzyć okna wyboru pliku eksportu.'
};

export function userErrorMessage(error: unknown, fallback: string): string {
  return error instanceof AppFailure
    ? (messages[error.kind] ?? fallback)
    : fallback;
}
