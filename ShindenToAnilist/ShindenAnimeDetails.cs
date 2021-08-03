using System.Collections.Generic;

namespace ShindenToAnilist
{
    public record ShindenAnimeDetails(int? Year, IReadOnlyList<string> AlternativeTitles);
}