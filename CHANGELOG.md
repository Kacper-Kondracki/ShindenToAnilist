# Changelog

## V3 1.0.1

Małe wydanie porządkowe: wewnętrzny refactor importu źródeł, usunięcie roboczego tekstu z ekranu startowego i wyrównanie wersji w manifestach.

## V3 1.0.0

W końcu. Wersja V3 to całościowe przepisanie aplikacji od zera. Dzięki silnikowi w Ruście i interfejsowi w Svelte 5 aplikacja śmiga i konwertuje listy anime z polskich serwisów jak marzenie.

To nadal ta sama idea. Importujesz listę i poprawiasz te wpisy, których automatyczny algorytm nie był w stanie dopasować. A na końcu eksportujesz XML, którego możesz użyć w MyAnimeList albo AniList.

Obsługuje Shinden, AnimeZone i Oglądaj Anime. Działa zarówno w Windowsie, jak i w systemie Linux.

### Najważniejsze zmiany

- Nowy frontend w Svelte 5 z widokiem do przeglądania listy, kandydatów i ręcznej korekty.
- Nowy backend w Ruście, podzielony na core, gRPC i warstwę desktopową.
- Wsparcie dla Shindena, AnimeZone i Oglądaj Anime.
- Nowy algorytm automatycznego dopasowywania tytułów do lokalnej bazy anime.
- Ręczne poprawki, ignorowanie wpisów i zapamiętywanie decyzji dla konkretnego źródła/użytkownika.
- Eksport XML do importu w MyAnimeList albo AniList.
- Paczki desktopowe dla Windowsa i Linuksa.
- Lepsze komunikaty błędów, powiadomienia, logi i obsługa problemów z zewnętrznymi serwisami.

Duża część nowej aplikacji powstała przy wsparciu narzędzi AI, szczególnie frontend, gRPC i powłoki
desktopowe. Sam UX, flow aplikacji i decyzje produktowe były projektowane ręcznie do perfekcji.

### Jak działa dopasowywanie

V3 nie patrzy już tylko na podobieństwo tytułu. Program najpierw wyszukuje kandydatów w bazie, a
potem punktuje ich według różnych kategorii:

- tytuł,
- sezon/część,
- rok premiery,
- typ anime,
- status emisji,
- sezon emisji,
- liczba odcinków.

Jeżeli dane źródło czegoś nie podaje, algorytm nie obniża oceny wpisu na siłę. Brakujący sygnał jest pomijany,
a reszta wag jest przeliczana. Dzięki temu ten sam mechanizm działa dla Shindena, AnimeZone i
Oglądaj Anime, mimo że każde z tych źródeł podaje trochę inne dane.

Program wybiera automatyczny wynik tylko wtedy, gdy jest wystarczająco mocny i wyraźnie lepszy od
innych kandydatów. Jeśli wynik jest niepewny, wpis trafia do ręcznej korekty. To celowa decyzja:
lepiej pokazać użytkownikowi wątpliwy przypadek niż zapisać zły tytuł w eksporcie.

AnimeZone i Oglądaj Anime mogą też dawać bezpośrednie linki/ID MAL. Jeśli taki identyfikator pasuje
do bazy, aplikacja używa go jako mocnego dopasowania.

### Windows

1. Pobierz `ShindenToAnilist-*-windows-x64-tauri.exe` z GitHub Releases.
2. Uruchom plik.
3. Jeśli SmartScreen ostrzeże przed aplikacją, upewnij się, że plik pochodzi z oficjalnego wydania,
   i dopiero wtedy potwierdź uruchomienie.
4. W aplikacji wybierz źródło, pobierz listę, sprawdź dopasowania i wyeksportuj XML.

### Linux

W systemie Linux najlepiej użyć builda Electronowego. W praktyce działa najsprawniej.

Polecana opcja:

```bash
chmod +x ShindenToAnilist-*-linux-x64-system-electron.AppImage
./ShindenToAnilist-*-linux-x64-system-electron.AppImage
```

Ten wariant używa Electrona zainstalowanego w systemie. Jeśli aplikacja nie startuje i pyta o
Electrona, zainstaluj pakiet `electron` z repozytorium swojej dystrybucji.

Jeśli nie chcesz niczego doinstalowywać, pobierz cięższą paczkę `*-bundled.AppImage`, która ma
Electrona w środku. Wariant Tauri (`*-tauri.AppImage`) też jest dostępny, ale na Linuksie nie jest
preferowaną opcją.

```bash
chmod +x ShindenToAnilist-*.AppImage
./ShindenToAnilist-*.AppImage
```

### Warto wiedzieć

- Shinden może czasem pokazać challenge Cloudflare. Aplikacja rozpoznaje ten przypadek i otwiera
  okno weryfikacji, po którym próbuje kontynuować pobieranie listy.
- AnimeZone i Oglądaj Anime są wolniejsze, bo dane są pobierane przez scraping. Dlatego import ma
  limity zapytań i widoczny postęp.
- Prywatne albo niedostępne listy powinny dawać czytelny komunikat zamiast technicznego błędu.
- Jeśli przed eksportem zostały nierozwiązane wpisy, aplikacja ostrzega zamiast udawać, że wszystko
  jest pewne.

### Technicznie

V3 używa Rusta 2024, gRPC/Protocol Buffers, Svelte 5, TypeScript, Tauri, Electron, Tailwind CSS 4 i
pnpm/Cargo workspace. Core odpowiada za domenę, matcher, providerów i eksport. Frontend odpowiada za
UX i komunikaty dla użytkownika. Warstwy Tauri/Electron spinają to z systemem operacyjnym.
