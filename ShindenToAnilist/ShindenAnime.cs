namespace ShindenToAnilist
{
    public record ShindenAnime(
        string Title,
        string Link,
        int? Score,
        WatchStatus WatchStatus,
        int Progress,
        int? Episodes,
        AnimeType AnimeType);
}