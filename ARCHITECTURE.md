# Architektura

Ten dokument opisuje granice modułów i nazewnictwo, które pomagają utrzymać projekt po dodaniu
kolejnych źródeł list anime.

## Core

`crates/shinden-to-anilist-core` zawiera logikę domenową: modele list, scrapery dostawców,
dopasowywanie, bazę anime i eksport XML. Provider powinien wystawiać wspólne kontrakty listy oraz
opcjonalne możliwości, na przykład bezpośrednie linki MAL. Szczegóły pobierania stron, limitów i
parsowania zostają w module konkretnego providera.

## Usługa Rust i gRPC

`crates/shinden-to-anilist-grpc/src/server/` dzieli usługę na warstwy:

- `source.rs` obsługuje pobieranie list źródłowych, walidację providera, postęp i pełne wpisy źródła.
- `database.rs` obsługuje ładowanie, aktualizację, metadane i sidecary bazy.
- `matching.rs` obsługuje wyszukiwanie, dopasowanie list i eksport wyników.
- `providers/` zawiera logikę zależną od providera: `shinden`, `animezone`, `ogladajanime`, wspólny
  collector scrapowanych list oraz bezpośrednie dopasowania MAL.
- `streaming.rs` dzieli odpowiedzi gRPC na partie.
- `mod.rs` definiuje publiczny typ `ShindenToAnilist` i stan usługi.
- `grpc.rs` zawiera adapter dla samodzielnego serwera gRPC, dostępny z funkcją Cargo `grpc`.

`source.rs` i `matching.rs` mogą zawierać małe matche po enumach tylko jako routery. Kod pobierania,
walidacji i provider-specific matchingu powinien mieszkać w `providers/`, żeby nowy provider nie
rozpychał centralnych handlerów.

Nazwy RPC i wiadomości protobuf pozostają stabilne. Jeżeli pole protobuf ma historyczną nazwę
`shinden_id`, a dane są już provider-agnostic, kod serwera powinien preferować semantykę `source_id`
i traktować `shinden_id` jako zgodnościowy fallback.

## Electron

`crates/shinden-to-anilist-node` wystawia asynchroniczne wywołania usługi przez Node-API. Proces
główny Electrona ładuje `shinden_to_anilist.node` z zasobów aplikacji i udostępnia wywołania głównej
ramce renderera przez preload i IPC. Wiadomości oraz błędy przechodzą jako binarne protobuf.
Renderer pobiera pojedyncze partie strumienia i może zamknąć go przez IPC. Zamknięcie strumienia
przerywa oczekiwanie na kolejną partię i anuluje pobieranie listy.

## Tauri

`tauri/src/` jest mostem między frontendem i usługą domenową:

- `state.rs` przechowuje `AppState`, usługę i mapę anulowania pobierania źródeł.
- `dto.rs` zawiera kształty JSON i konwersje z protobuf, w tym bezpieczne przesyłanie `u64` jako string.
- `commands/` grupuje komendy według obszaru: source, shinden, database, matching, export i paths.
- `lib.rs` powinien zawierać tylko builder Tauri, pluginy, stan i rejestrację komend.

Nazwy komend, argumenty i serializowane pola DTO są kontraktem frontendu.

## Frontend

Warstwa `frontend/src/lib/api/` powinna rozdzielać:

- wykrywanie runtime'u i klientów transportowych,
- `protobufService.ts` obsługuje komunikaty protobuf przez IPC w Electronie i gRPC-Web w przeglądarce, a `tauriService.ts` obsługuje komendy Tauri,
- wspólne mapowanie DTO/protobuf do domeny UI w `mapping.ts`,
- śledzenie wersji danych,
- cienką fasadę `appService.ts`, która wybiera backend raz i deleguje funkcje usługowe,
- wybór ścieżki eksportu.

`appService.ts` nie powinien zawierać per-funkcyjnych przełączników `isTauriRuntime()`. Wyjątkiem jest
warstwa `runtime.ts`, która zna środowisko i udostępnia backendowi transport.

Kontrolery Svelte powinny utrzymywać reaktivność blisko workflow, a komponenty powinny renderować
stan i zdarzenia bez ukrytego pobierania danych. Przy asynchronicznym dopasowywaniu zachowujemy
wzorzec ostatniego zatwierdzonego wyniku: nie czyścimy użytecznych danych tylko dlatego, że nowa
odpowiedź jest w toku.

## Source kontra Shinden

`Shinden` oznacza wyłącznie provider Shinden albo historyczne RPC/DTO, których nie zmieniamy bez
migracji protobuf. Dane wspólne dla Shindena, AnimeZone i Oglądaj Anime nazywamy `source`.

Przykłady:

- provider-agnostic identyfikator wpisu: `sourceId`,
- provider-agnostic wersja listy: `sourceVersion`,
- lista lub endpoint tylko dla Shindena: `shindenList`, `fetchShindenList`.

Takie nazewnictwo zmniejsza ryzyko, że nowy provider przypadkiem odziedziczy założenia prawdziwe
tylko dla Shindena.
