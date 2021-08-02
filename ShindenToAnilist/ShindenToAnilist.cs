using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using AngleSharp;
using AngleSharp.Dom;
using Flurl;

namespace ShindenToAnilist
{
    public class ShindenToAnilist
    {
        private readonly IHtmlContext _context;

        public ShindenToAnilist(IHtmlContext? htmlContext = null)
        {
            _context = htmlContext ?? new HtmlContext(BrowsingContext.New(Configuration.Default
                .WithDefaultLoader()
                .WithDefaultCookies()));
        }

        public async Task<IEnumerable<ShindenAnime>> DownloadShindenListAsync(string userId)
        {
            using var document = await _context.GetDocumentAsync("https://shinden.pl/animelist"
                .AppendPathSegment(userId)
                .AppendPathSegment("all"));

            return ParseDocument(document);
        }

        public async Task<int?> GetYearAsync(ShindenAnime anime)
        {
            using var infoDocument = await _context.GetDocumentAsync(anime.Link);

            var date = infoDocument.All
                .First(x => x.LocalName == "dl" && x.ClassName == "info-aside-list").Children
                .SkipWhile(x => x.TextContent != "Data emisji:")
                .Skip(1)
                .FirstOrDefault()?.TextContent;

            int? dateCorrect = date is null ? null : int.Parse(date.Split('.').Last());

            return dateCorrect;
        }

        private static IEnumerable<ShindenAnime> ParseDocument(IDocument document)
        {
            var result = document.All.Where(x =>
                x.ClassName != null &&
                x.LocalName == "tr" &&
                x.ClassName.StartsWith("title-row")).ToList();

            if (result.Count == 0) throw new InvalidOperationException();

            Console.WriteLine("adkjhkh");

            return (from element in result
                let title = element.Children[1].Children[0].TextContent
                let link = "https://shinden.pl".AppendPathSegment(element.Children[1].Children[0].GetAttribute("href")!)
                let score = element.Children[2].TextContent
                let status = element.Children[3].TextContent
                let progressAndEpisodes = element.Children[4].TextContent.Split('/', 2).Select(x => x.Trim()).ToArray()
                let type = element.Children[5].TextContent
                let correctScore = (int?) (score == "?" ? null : int.Parse(score))
                let progress = int.Parse(progressAndEpisodes[0])
                let correctEpisodes = (int?) (progressAndEpisodes[1] == "?" ? null : int.Parse(progressAndEpisodes[1]))
                let correctStatus = status switch
                {
                    "Oglądam" => WatchStatus.CurrentlyWatching,
                    "Obejrzane" => WatchStatus.Completed,
                    "Pomijam" => WatchStatus.Skipped,
                    "Wstrzymane" => WatchStatus.OnHold,
                    "Porzucone" => WatchStatus.Dropped,
                    "Planuję" => WatchStatus.PlanToWatch,
                    _ => throw new ArgumentOutOfRangeException(status)
                }
                let correctType = type switch
                {
                    "TV" => AnimeType.Tv,
                    "Movie" => AnimeType.Movie,
                    "OVA" => AnimeType.Ova,
                    "ONA" => AnimeType.Ona,
                    "Music" => AnimeType.Music,
                    "Special" => AnimeType.Special,
                    _ => throw new ArgumentOutOfRangeException(type)
                }
                select new ShindenAnime(title, link, correctScore, correctStatus, progress, correctEpisodes,
                    correctType)).ToList();
        }
    }
}