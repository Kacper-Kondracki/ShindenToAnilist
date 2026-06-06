package anime

type DatabaseInfo struct {
	LastUpdate string `json:"lastUpdate"`
	Release    string `json:"release"`
	Sha256     string `json:"sha256"`
	Path       string `json:"path"`
	Updated    bool   `json:"updated"`
}

type ShindenList struct {
	Entries []ShindenEntry `json:"entries"`
}

type ShindenListIndex struct {
	EntryIDs []uint64 `json:"entryIds"`
}

type ShindenEntry struct {
	ID              uint64  `json:"id"`
	CoverID         *int    `json:"coverId"`
	Title           string  `json:"title"`
	AnimeStatus     string  `json:"animeStatus"`
	AnimeType       string  `json:"animeType"`
	PremiereDate    *string `json:"premiereDate"`
	FinishDate      *string `json:"finishDate"`
	Episodes        *int    `json:"episodes"`
	IsFavourite     bool    `json:"isFavourite"`
	WatchStatus     string  `json:"watchStatus"`
	WatchedEpisodes int     `json:"watchedEpisodes"`
	Score           *int    `json:"score"`
	Note            *string `json:"note"`
	Description     *string `json:"description"`
}

type OptionalFloat = *float32

type TitleMetadata struct {
	Season            OptionalFloat `json:"season"`
	Part              OptionalFloat `json:"part"`
	Episode           OptionalFloat `json:"episode"`
	HasSeasonKeyword  bool          `json:"hasSeasonKeyword"`
	HasPartKeyword    bool          `json:"hasPartKeyword"`
	HasEpisodeKeyword bool          `json:"hasEpisodeKeyword"`
}

type ConsolidatedMetadata struct {
	Season         OptionalFloat `json:"season"`
	Part           OptionalFloat `json:"part"`
	Episode        OptionalFloat `json:"episode"`
	IsFinalSeason  bool          `json:"isFinalSeason"`
	IsFinalPart    bool          `json:"isFinalPart"`
	IsFinalEpisode bool          `json:"isFinalEpisode"`
}

type AnimeDatabase struct {
	LastUpdate *string         `json:"lastUpdate"`
	Entries    []DatabaseEntry `json:"entries"`
}

type DatabaseEntry struct {
	ID                   uint64               `json:"id"`
	ConsolidatedMetadata ConsolidatedMetadata `json:"consolidatedMetadata"`
	Sources              []string             `json:"sources"`
	Title                string               `json:"title"`
	NormalizedTitle      string               `json:"normalizedTitle"`
	Metadata             TitleMetadata        `json:"metadata"`
	AnimeType            string               `json:"animeType"`
	Episodes             int                  `json:"episodes"`
	Status               string               `json:"status"`
	Season               string               `json:"season"`
	Year                 *int                 `json:"year"`
	Picture              string               `json:"picture"`
	Thumbnail            string               `json:"thumbnail"`
	Duration             *int                 `json:"duration"`
	Synonyms             []string             `json:"synonyms"`
	NormalizedSynonyms   []string             `json:"normalizedSynonyms"`
	Studios              []string             `json:"studios"`
	Producers            []string             `json:"producers"`
	RelatedAnime         []string             `json:"relatedAnime"`
	Tags                 []string             `json:"tags"`
}

type SearchOptions struct {
	Mode      string  `json:"mode"`
	Limit     int     `json:"limit"`
	Threshold float32 `json:"threshold"`
}

type MatchOptions struct {
	CandidateLimit  int     `json:"candidateLimit"`
	SearchThreshold float32 `json:"searchThreshold"`
	ResultLimit     *int    `json:"resultLimit"`
}

type MatchQueryOptions struct {
	Search      SearchOptions `json:"search"`
	ResultLimit *int          `json:"resultLimit"`
}

type SearchItem struct {
	ID    uint64  `json:"id"`
	Score float32 `json:"score"`
}

type SearchResult struct {
	Items []SearchItem `json:"items"`
}

type ScoreBreakdown struct {
	SearchScore   float32 `json:"searchScore"`
	SeasonScore   float32 `json:"seasonScore"`
	YearScore     float32 `json:"yearScore"`
	TypeScore     float32 `json:"typeScore"`
	StatusScore   float32 `json:"statusScore"`
	SeasonalScore float32 `json:"seasonalScore"`
	EpisodesScore float32 `json:"episodesScore"`
	FinalScore    float32 `json:"finalScore"`
}

type ScoredCandidate struct {
	ID    uint64         `json:"id"`
	Score ScoreBreakdown `json:"score"`
}

type MatchResult struct {
	Items  []ScoredCandidate `json:"items"`
	Top    []ScoredCandidate `json:"top"`
	Winner *ScoredCandidate  `json:"winner"`
}

type ShindenMatchResult struct {
	ShindenID uint64      `json:"shindenId"`
	Result    MatchResult `json:"result"`
}

type MatchListResult struct {
	Entries   []ShindenMatchResult `json:"entries"`
	Total     int                  `json:"total"`
	Winners   int                  `json:"winners"`
	HasTop    int                  `json:"hasTop"`
	Unmatched int                  `json:"unmatched"`
}

type MatchSelection struct {
	ShindenID  uint64 `json:"shindenId"`
	DatabaseID uint64 `json:"databaseId"`
}

type ExportResult struct {
	Path          string `json:"path"`
	ExportedCount int    `json:"exportedCount"`
	Cancelled     bool   `json:"cancelled"`
}
