using System;
using System.Collections.Generic;

namespace ShindenToAnilist
{
    public record DataAnime(
        List<Uri> Sources,
        string Title,
        AnimeType Type,
        int Episodes,
        AirStatus Status,
        AnimeSeason AnimeSeason,
        Uri Picture,
        Uri Thumbnail,
        List<string> Synonyms,
        List<Uri> Relations,
        List<string> Tags);

    public record AnimeSeason(Season Season, int? Year);

    public enum Season
    {
        Undefined,
        Spring,
        Summer,
        Fall,
        Winter
    }
}