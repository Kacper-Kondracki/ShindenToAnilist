using System.Collections.Generic;

namespace ShindenToAnilist
{
    public record AnimeDatabase(IReadOnlyList<DataAnime> Data);
}