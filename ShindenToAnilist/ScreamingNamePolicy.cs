using System.Text.Json;

namespace ShindenToAnilist
{
    public class ScreamingNamePolicy : JsonNamingPolicy
    {
        public override string ConvertName(string name)
        {
            return name.ToUpperInvariant();
        }
    }
}