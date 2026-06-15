# Development

> Dokumentacja dla osób rozwijających, budujących albo paczkujących ShindenToAnilist.

[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](Cargo.toml)
[![Svelte](https://img.shields.io/badge/Svelte-5-ff3e00.svg)](frontend/package.json)
[![Electron](https://img.shields.io/badge/Electron-42-47848f.svg)](electron/package.json)

## Stack technologiczny

| Obszar | Technologia | Rola |
| --- | --- | --- |
| Desktop shell | Electron | Natywne okno, dialogi plików, cykl życia procesu sidecara |
| Renderer | Svelte 5, Vite, TypeScript | Interaktywny interfejs workspace'u |
| Style | Tailwind CSS 4, daisyUI, Catppuccin | Layout aplikacji i motyw graficzny |
| Silnik | Rust 2024 | Szybkie wczytywanie list, dopasowywanie, dostęp do bazy, eksport XML |
| Granica IPC/API | gRPC, tonic, tonic-web, Connect-Web, Protocol Buffers | Typowana komunikacja między rendererem i rustowym sidecarem |
| Paczkowanie | electron-builder, własne skrypty Node | Desktopowe paczki dla Linuksa i Windowsa |
| Narzędzia workspace'u | pnpm workspaces, Cargo workspace, Prettier, rustfmt | Development i formatowanie |

## Struktura repozytorium

```text
.
+-- crates/
|   +-- shinden-to-anilist-core/   # logika konwersji, dopasowywania, bazy i eksportu w Ruście
|   +-- shinden-to-anilist-grpc/   # sidecar gRPC wystawiający silnik aplikacji
+-- electron/                      # kod main/preload Electrona i konfiguracja buildera
+-- frontend/                      # aplikacja renderera w Svelte
+-- proto/                         # współdzielone definicje usług protobuf
+-- scripts/                       # skrypty buildowania, paczkowania, ikon i sidecara
+-- bruno/                         # kolekcja zapytań API dla usługi gRPC
```

## Wymagania

- Rust z toolchainem obsługującym edycję 2024.
- Nightly Rust tylko do formatowania: `cargo +nightly fmt`.
- pnpm 11.x. Manifesty workspace'u wymagają `pnpm ^11.3.0`.
- Node.js kompatybilny z toolchainem Electron/Vite.
- Do cross-buildów Windowsa z Linuksa: dodatkowe narzędzia wymagane przez `cargo xwin`.

Instalacja zależności JavaScript:

```bash
pnpm install
```

## Uruchamianie lokalne

Uruchom pełną aplikację desktopową w trybie developerskim:

```bash
pnpm dev
```

To uruchamia razem trzy procesy:

| Proces | Komenda | Rola |
| --- | --- | --- |
| Rust sidecar | `pnpm dev:grpc` | Uruchamia usługę gRPC na `127.0.0.1:45187` |
| Renderer | `pnpm dev:frontend` | Uruchamia dev server Vite/Svelte |
| Desktop shell | `pnpm dev:electron` | Buduje i otwiera aplikację Electron podpiętą do usług developerskich |

Po pierwszym buildzie możesz użyć szybszego sidecara:

```bash
pnpm dev:optimized
```

Przydatne komendy do pracy nad konkretnymi częściami:

```bash
pnpm dev:grpc
pnpm dev:frontend
pnpm dev:electron
pnpm --filter frontend check
pnpm --filter frontend proto:gen
```

## Buildowanie

Build pojedynczych części:

```bash
pnpm build:frontend
pnpm build:electron
pnpm build:grpc:linux
```

Tworzenie paczek dla Linuksa:

```bash
pnpm bundle:linux:system
pnpm bundle:linux:bundled
pnpm bundle:linux:appimage:system
pnpm bundle:linux:appimage:bundled
```

Tworzenie portable builda dla Windowsa:

```bash
pnpm bundle:windows:portable
```

Z Linuksa, po skonfigurowaniu `cargo xwin`:

```bash
pnpm bundle:windows:portable:xwin
```

Wyniki buildów trafiają do `dist/` oraz `dist/electron-builder/`.

## Formatowanie

Formatowanie całości:

```bash
pnpm format
```

Formatowanie tylko Rusta:

```bash
pnpm format:crates
```

> [!TIP]
> Formatowanie Rusta używa `cargo +nightly fmt`, ponieważ repozytorium korzysta z ustawień
> `rustfmt.toml` dostępnych tylko na nightly.

Formatowanie tylko workspace'ów JavaScript/TypeScript/Svelte:

```bash
pnpm format:root
pnpm format:frontend
pnpm format:electron
```

## Commitowanie

Przed commitem:

1. Trzymaj zmiany skupione na jednym temacie.
2. Uruchom odpowiedni formatter dla plików, które zostały zmienione.
3. Uruchom celowane checki tylko wtedy, gdy są istotne dla zmiany.
4. Przejrzyj diff:

```bash
git status --short
git diff
```

Używaj czytelnych komunikatów commitów:

```bash
git add README.md DEVELOPMENT.md
git commit -m "docs: split user and development docs"
```

Zalecane prefiksy commitów:

| Prefix | Kiedy używać |
| --- | --- |
| `feat:` | Funkcje widoczne dla użytkownika |
| `fix:` | Poprawki błędów |
| `docs:` | Zmiany wyłącznie w dokumentacji |
| `refactor:` | Wewnętrzne zmiany kodu bez zmiany zachowania |
| `build:` | Build, paczkowanie, zależności albo tooling |
| `style:` | Zmiany wyłącznie formatowania |
