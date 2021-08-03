using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Xml;
using Newtonsoft.Json;
using Formatting = System.Xml.Formatting;

namespace ShindenToAnilist
{
    public static class AnimeListConverter
    {
        public static IReadOnlyList<DataAnime>? GetAnimeDatabase()
        {
            using var stream = File.OpenText("anime-offline-database.json");
            using var jsonReader = new JsonTextReader(stream);

            var jsonSerializer = new JsonSerializer();
            var deserialized = jsonSerializer.Deserialize<AnimeDatabase>(jsonReader);

            return deserialized?.Data;
        }

        public static void Convert(IEnumerable<(ShindenAnime, DataAnime)> animes, string path)
        {
            using var xmlWriter = new XmlTextWriter(path, Encoding.UTF8) {Formatting = Formatting.Indented};

            xmlWriter.WriteStartDocument();

            xmlWriter.WriteStartElement("myanimelist");

            xmlWriter.WriteStartElement("myinfo");
            xmlWriter.WriteElementString("user_export_type", "1");
            xmlWriter.WriteEndElement();

            foreach ((ShindenAnime shindenAnime, DataAnime dataAnime) in animes)
            {
                xmlWriter.WriteStartElement("anime");

                var animeId = dataAnime.Sources.First(x => x.ToString().Contains("myanimelist", StringComparison.Ordinal)).ToString().Split('/')
                    .Last();

                xmlWriter.WriteElementString("series_animedb_id", animeId);
                xmlWriter.WriteElementString("my_watched_episodes", shindenAnime.Progress.ToString());
                xmlWriter.WriteElementString("my_start_date", "0000-00-00");
                xmlWriter.WriteElementString("my_finish_date", "0000-00-00");
                xmlWriter.WriteElementString("my_score",
                    shindenAnime.Score.HasValue ? shindenAnime.Score.ToString() : "0");

                var status = shindenAnime.WatchStatus switch
                {
                    WatchStatus.Skipped => "Dropped",
                    WatchStatus.Completed => "Completed",
                    WatchStatus.Dropped => "Dropped",
                    WatchStatus.CurrentlyWatching => "Watching",
                    WatchStatus.OnHold => "On-Hold",
                    WatchStatus.PlanToWatch => "Plan to Watch",
                    _ => throw new ArgumentOutOfRangeException(null, nameof(shindenAnime.WatchStatus))
                };

                xmlWriter.WriteElementString("my_status", status);
                xmlWriter.WriteElementString("update_on_import", "1");

                xmlWriter.WriteEndElement();
            }

            xmlWriter.WriteEndElement();

            xmlWriter.WriteEndDocument();
        }
    }
}