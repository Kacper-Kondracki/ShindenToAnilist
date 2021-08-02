using System;
using System.Collections.Generic;
using System.Globalization;
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
            var database = await AnimeListConverter.GetAnimeDatabaseAsync();
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

            Console.WriteLine("Rozpoczynanie wyszukiwania:");
            foreach (var anime in animes)
            {
                var search = database.Where(x =>
                    (
                        x.Title == anime.Title ||
                        x.Synonyms.Contains(anime.Title)
                    ) &&
                    x.Type == anime.AnimeType &&
                    x.Sources.Any(uri => uri.ToString().Contains("myanimelist")) &&
                    x.Sources.Any(uri => uri.ToString().Contains("anilist"))).ToList();


                switch (search.Count)
                {
                    case 0:
                        Console.ForegroundColor = ConsoleColor.Red;
                        Console.WriteLine($"Nie znaleziono: {anime.Title} {anime.Link}");
                        Console.ResetColor();
                        break;
                    case > 1:
                        var year = await shindenToAnilist.GetYearAsync(anime);
                        var research = search.Where(x => x.AnimeSeason.Year == year).ToList();
                        if (research.Count == 1)
                        {
                            found.Add((anime, research[0]));
                        }
                        else
                        {
                            Console.ForegroundColor = ConsoleColor.Yellow;
                            Console.WriteLine($"Wiele wyników: {anime.Title} {anime.Link}");
                            Console.ResetColor();
                        }

                        break;

                    case 1:
                        found.Add((anime, search[0]));
                        break;
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
    }
}