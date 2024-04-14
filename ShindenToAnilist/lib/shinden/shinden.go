package shinden

import (
	"encoding/json"
	"fmt"
	"net/http"
	"shinden-to-anilist/lib"
	"shinden-to-anilist/lib/searcher"
	"strconv"
	"time"
)

func GetList(id, limit, offset int) ([]Anime, error) {
	uri := fmt.Sprintf("https://lista.shinden.pl/api/userlist/"+
		"%d/anime?limit=%d&offset=%d", id, limit, offset)
	res, err := http.Get(uri)
	if err != nil {
		return nil, err
	}
	defer res.Body.Close()

	var shinden responseJSON
	if err = json.NewDecoder(res.Body).Decode(&shinden); err != nil {
		return nil, err
	}

	return shinden.Result.Items, nil
}

type Anime struct {
	Title               string
	PremiereDate        *time.Time
	FinishDate          *time.Time
	TitleStatus         lib.TitleStatus
	Episodes            *int
	AnimeType           lib.AnimeType
	WatchedEspisodesCnt int
	RateTotal           *int
	UserNote            *string
	WatchStatus         lib.WatchStatus
	IsFavourite         bool
	TitleID             int
}

func (s *Anime) MarshalJSON() ([]byte, error) {
	a := searcher.SearchAnime(s)
	j := searcher.SearchAnimeJSON{
		Title:       a.GetTitle(),
		Year:        a.GetYear(),
		TitleStatus: a.GetTitleStatus(),
		Episodes:    a.GetEpisodes(),
		AnimeType:   a.GetAnimeType(),
		Source:      a.GetSource(),
		Notes:       a.GetNotes(),
	}
	return json.Marshal(j)
}
func (s *Anime) GetTitle() string {
	return s.Title
}

func (s *Anime) GetYear() *int {
	if s.PremiereDate == nil {
		return nil
	}
	year := s.PremiereDate.Year()
	return &year
}

func (s *Anime) GetTitleStatus() *lib.TitleStatus {
	return &s.TitleStatus
}

func (s *Anime) GetEpisodes() *int {
	if s.Episodes == nil {
		tmp := 0
		return &tmp
	}
	return s.Episodes
}

func (s *Anime) GetAnimeType() *lib.AnimeType {
	return &s.AnimeType
}

func (s *Anime) GetSource() *string {
	url := fmt.Sprintf("https://shinden.pl/series/%d", s.TitleID)
	return &url
}

func (s *Anime) GetNotes() *string {
	return s.UserNote
}

/*
To wkurwia, że w Go nie możesz po prostu podmienić konwerter dla poszczególnego pola,
tylko trzeba kompletnie ręcznie mapować json który serwer wypluł np isFavourite jako liczbę do boola
*/

func (s *Anime) UnmarshalJSON(bytes []byte) error {
	var animeJSON animeJSON
	if err := json.Unmarshal(bytes, &animeJSON); err != nil {
		return err
	}

	s.Title = animeJSON.Title
	if animeJSON.PremiereDate != nil {
		t := time.Unix(int64(*animeJSON.PremiereDate), 0)
		s.PremiereDate = &t
	}
	if animeJSON.FinishDate != nil {
		t := time.Unix(int64(*animeJSON.FinishDate), 0)
		s.FinishDate = &t
	}
	s.TitleStatus = animeJSON.TitleStatus
	s.Episodes = animeJSON.Episodes
	watchedEpisodesCnt, err := strconv.Atoi(animeJSON.WatchedEspisodesCnt)
	if err != nil {
		return err
	}
	s.AnimeType = animeJSON.AnimeType
	s.WatchedEspisodesCnt = watchedEpisodesCnt
	s.RateTotal = animeJSON.RateTotal
	s.UserNote = animeJSON.UserNote
	s.WatchStatus = animeJSON.WatchStatus
	if animeJSON.IsFavourite == 1 {
		s.IsFavourite = true
	}
	s.TitleID = animeJSON.TitleID

	return nil
}

type animeJSON struct {
	Title               string          `json:"title"`
	PremiereDate        *int            `json:"premiereDate"`
	FinishDate          *int            `json:"finishDate"`
	TitleStatus         lib.TitleStatus `json:"titleStatus"`
	Episodes            *int            `json:"episodes"`
	AnimeType           lib.AnimeType   `json:"animeType"`
	WatchedEspisodesCnt string          `json:"watchedEpisodesCnt"`
	RateTotal           *int            `json:"rateTotal"`
	UserNote            *string         `json:"userNote"`
	WatchStatus         lib.WatchStatus `json:"watchStatus"`
	IsFavourite         int             `json:"isFavourite"`
	TitleID             int             `json:"titleId"`
}

type responseJSON struct {
	Success bool       `json:"success,omitempty"`
	Message string     `json:"message,omitempty"`
	Result  resultJSON `json:"result,omitempty"`
}

type resultJSON struct {
	Count int     `json:"count,omitempty"`
	Items []Anime `json:"items,omitempty"`
}
