# ShindenToAnilist

> Eksportuj swoją listę z Shindena do pliku XML, który zaimportujesz do AniList albo MyAnimeList.

[![License: MPL-2.0](https://img.shields.io/badge/license-MPL--2.0-blue.svg)](LICENSE)

**ShindenToAnilist** to aplikacja desktopowa dla osób, które prowadzą swoją listę anime na
[Shindenie](https://shinden.pl) i chcą przenieść ją do serwisów takich jak
[AniList](https://anilist.co/settings/import) albo [MyAnimeList](https://myanimelist.net/import.php).

Aplikacja pozwala
przejrzeć lub ręcznie poprawić niepewne dopasowania, a na końcu zapisuje eksport XML zgodny z MAL.

## Wydania

Gotowe paczki aplikacji są publikowane na stronie wydań:

<https://github.com/Kacper-Kondracki/ShindenToAnilist/releases>

> [!NOTE]
> W porównaniu ze starszą wersją nacisk jest
> położony na szybsze dopasowywanie tytułów, wygodniejszy interfejs desktopowy i ręczną korektę
> niepewnych wyników. Wprowadzono zupełnie nowy algorytm znajdujący dopasowania, zoptymalizowany pod wielowątkowość i dostrojony by automatycznie znajdował dokladnie jak najwięcej tytułów. Nikt nie lubi ręcznie męczyć się z tym :)

## Co robi aplikacja

1. Wczytuje publiczną listę anime z Shindena po ID lub URL użytkownika.
2. Pobiera i zapisuje lokalną bazę używaną do dopasowywania tytułów.
3. Automatycznie szuka najlepiej pasującę tytuły w bazie.
4. Pokazuje dopasowania i listę kandydatów.
5. Pozwala ręcznie wybrać lepsze dopasowanie, gdy automatyczny wynik nie wystarcza.
6. Eksportuje wybrane dopasowania do pliku XML.

> [!IMPORTANT]
> Aktualnie zaimplementowane jest wczytywanie list z Shindena. AnimeZone i Oglądaj Anime są widoczne
> w aplikacji jako przyszłe cele, ale wczytywanie list użytkownika dla nich nie jest jeszcze aktywne.
> AnimeZone i Oglądaj Anime linkują pod seriami linki do MALA, więc późniejszy eksport będzie całkowicie automatyczny bez ręcznych dopasowań!

## Import eksportu

Po wyeksportowaniu XML-a z aplikacji zaimportuj go na jednej z tych stron:

- AniList: <https://anilist.co/settings/import>
- MyAnimeList: <https://myanimelist.net/import.php>

## Dla developerów

Instrukcje deweloperskie znajdują się tu
[DEVELOPMENT.md](DEVELOPMENT.md).

## Licencja

Projekt jest dostępny na licencji [Mozilla Public License 2.0](LICENSE).
