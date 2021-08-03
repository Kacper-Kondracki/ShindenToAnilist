using System;
using System.Collections.Generic;

namespace ShindenToAnilist
{
    public record DataAnime(
        string Title,
        IReadOnlyList<Uri> Sources,
        AnimeType Type,
        int Episodes,
        AirStatus Status,
        AnimeSeason AnimeSeason,
        Uri Picture,
        Uri Thumbnail,
        IReadOnlyList<string> Synonyms,
        IReadOnlyList<Uri> Relations,
        IReadOnlyList<string> Tags);

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