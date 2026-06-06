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
