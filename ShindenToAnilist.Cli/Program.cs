using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Threading.Tasks;

namespace ShindenToAnilist.Cli
{
    public static class Program
    {
        public static async Task Main(string[] args)
        {
            CultureInfo.DefaultThreadCurrentCulture = new CultureInfo("pl-PL");

            string user;

            if (args.Length == 0)
            {
                Console.Write("Podaj ID użytkownika: ");
                user = Console.ReadLine() ?? "";
            }
            else
            {
                user = string.Join(' ', args);
            }


            Console.WriteLine("Ładowanie bazy danych");
            var database = AnimeListConverter.GetAnimeDatabase();

            if (database == null)
            {
                Console.WriteLine("Nie można wczytać bazy danych anime.");
                return;
            }

            var shindenToAnilist = new ShindenToAnilist();

            Console.WriteLine($"Pobieranie listy anime użytkownika {user}");


            List<ShindenAnime> animes;

            try
            {
                animes = (await shindenToAnilist.DownloadShindenListAsync(user)).ToList();
            }
            catch (InvalidOperationException)
            {
                Console.WriteLine(
                    "Nie można wczytać listy anime, sprawdź czy ID użytkownika jest prawidłowe i czy lista zawiera jakiekolwiek anime");
                return;
            }

            var found = new List<(ShindenAnime, DataAnime)>();
            await using var errorFile = File.CreateText("bledne_anime.txt");

            Console.WriteLine("Rozpoczynanie wyszukiwania:");
            foreach (var anime in animes)
            {
                var search = SearchAnime(database, anime.Title).ToList();

                if (search.Count == 1)
                {
                    found.Add((anime, search[0]));
                }
                else
                {
                    var (year, alternativeTitles) = await shindenToAnilist.GetDetailsAsync(anime);
                    
                    var alternative = alternativeTitles
                        .Select(x => SearchAnime(database, x))
                        .SelectMany(x => x)
                        .ToList();

                    search.AddRange(alternative);
                    search = search.Distinct().ToList();

                    var research = search.Where(x => x.AnimeSeason.Year == year).ToList();

                    if (research.Count > 1)
                    {
                        research = research.Where(x => x.Type == anime.AnimeType).ToList();
                    }
                    
                    switch (research.Count)
                    {
                        case 1:
                            found.Add((anime, research[0]));
                            break;
                        case 0:
                            var notFound = $"Nie znaleziono: {anime.Title} {anime.Link}";
                            await errorFile.WriteLineAsync(notFound);
                            
                            Console.ForegroundColor = ConsoleColor.Red;
                            Console.WriteLine(notFound);
                            Console.ResetColor();
                            break;
                        case > 1:
                            var multipleResults = $"Wiele wyników: {anime.Title} {anime.Link}";
                            await errorFile.WriteLineAsync(multipleResults);
                            
                            Console.ForegroundColor = ConsoleColor.Yellow;
                            Console.WriteLine(multipleResults);
                            Console.ResetColor();
                            break;
                    }
                }
            }

            Console.ForegroundColor = ConsoleColor.Cyan;
            Console.WriteLine($"\nZakończono szukanie\nZnaleziono {found.Count}/{animes.Count}\n");
            Console.ResetColor();
            Console.WriteLine("Rozpoczynanie konwertowania:");

            AnimeListConverter.Convert(found, "lista_anime.xml");

            Console.ForegroundColor = ConsoleColor.Green;
            Console.WriteLine("Zakończono konwertowanie do pliku: lista_anime.xml\n" +
                              "Zaimportuj swoją listę\n" +
                              "https://myanimelist.net/import.php\n" +
                              "https://anilist.co/settings/import");

            Console.ResetColor();
            Console.WriteLine("Wciśnij dowolny klawisz, aby kontynuować...");
            Console.ReadKey(true);
        }

        private static IEnumerable<DataAnime> SearchAnime(IEnumerable<DataAnime> database, string title)
        {
            var search = database.Where(x =>
                (
                    string.Equals(x.Title, title, StringComparison.InvariantCultureIgnoreCase) ||
                    x.Synonyms.Contains(title, StringComparer.InvariantCultureIgnoreCase)
                ) &&
                x.Sources.Any(uri => uri.ToString().Contains("myanimelist", StringComparison.Ordinal)) &&
                x.Sources.Any(uri => uri.ToString().Contains("anilist", StringComparison.Ordinal))).ToList();

            return search;
        }
    }
}