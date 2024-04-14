package searcher

import (
	"context"
	"golang.org/x/sync/semaphore"
	"reflect"
	"shinden-to-anilist/lib"
	"shinden-to-anilist/lib/db"
	"slices"
	"strings"
	"sync"
)

type SearchAnimeJSON struct {
	Title       string           `json:"title"`
	Year        *int             `json:"year"`
	TitleStatus *lib.TitleStatus `json:"title_status"`
	Episodes    *int             `json:"episodes"`
	AnimeType   *lib.AnimeType   `json:"anime_type"`
	Source      *string          `json:"source"`
	Notes       *string          `json:"notes"`
}

type SearchAnime interface {
	GetTitle() string
	GetYear() *int
	GetTitleStatus() *lib.TitleStatus
	GetEpisodes() *int
	GetAnimeType() *lib.AnimeType
	GetSource() *string
	GetNotes() *string
}

type SearchSuccess struct {
	SearchAnime SearchAnime `json:"search_anime,omitempty"`
	DBAnime     db.Anime    `json:"db_anime"`
}

type SearchMultiple struct {
	SearchAnime SearchAnime `json:"search_anime,omitempty"`
	DBAnime     []db.Anime  `json:"db_anime,omitempty"`
}

type SearchFail struct {
	SearchAnime SearchAnime `json:"search_anime,omitempty"`
}

func Search(searchAnime []SearchAnime, dbAnime []db.Anime) ([]SearchSuccess, []SearchMultiple, []SearchFail) {
	successChan := make(chan SearchSuccess, len(searchAnime))
	multipleChan := make(chan SearchMultiple, len(searchAnime))
	failChan := make(chan SearchFail, len(searchAnime))

	var wg sync.WaitGroup
	sem := semaphore.NewWeighted(20)
	for _, anime := range searchAnime {

		wg.Add(1)
		_ = sem.Acquire(context.Background(), 1)
		anime := anime
		go func() {
			defer wg.Done()
			defer sem.Release(1)

			success, multiple, fail := matchAnime(anime, dbAnime)
			if success != nil {
				successChan <- *success
			} else if multiple != nil {
				multipleChan <- *multiple
			} else if fail != nil {
				failChan <- *fail
			}
		}()
	}
	wg.Wait()
	close(successChan)
	close(multipleChan)
	close(failChan)

	success := make([]SearchSuccess, 0)
	multiple := make([]SearchMultiple, 0)
	fail := make([]SearchFail, 0)

	for searchSuccess := range successChan {
		success = append(success, searchSuccess)
	}

	for searchMultiple := range multipleChan {
		multiple = append(multiple, searchMultiple)
	}

	for searchFail := range failChan {
		fail = append(fail, searchFail)
	}

	return success, multiple, fail
}

func matchAnime(anime SearchAnime, animeDB []db.Anime) (*SearchSuccess, *SearchMultiple, *SearchFail) {
	results := make([]db.Anime, 0)
	for _, dbAnime := range animeDB {
		for _, source := range dbAnime.Sources {
			if !strings.Contains(source, "myanimelist") {
				continue
			}
			if !strings.Contains(source, "anilist") {
				continue
			}
		}

		if dbAnime.AnimeSeason.Year != nil && anime.GetYear() != nil {
			if *dbAnime.AnimeSeason.Year != *anime.GetYear() {
				continue
			}
		}

		animeTitle := strings.ToUpper(anime.GetTitle())
		dbTitle := strings.ToUpper(dbAnime.Title)
		if dbTitle == animeTitle {
			results = []db.Anime{dbAnime}
			break
		}

		if strings.Contains(dbTitle, animeTitle) {
			if !slices.ContainsFunc(results, func(anime db.Anime) bool {
				return reflect.DeepEqual(anime, dbAnime)
			}) {
				results = append(results, dbAnime)
			}
			continue
		}

		for _, synonym := range dbAnime.Synonyms {
			synonym := strings.ToUpper(synonym)
			if strings.Contains(synonym, animeTitle) {
				if !slices.ContainsFunc(results, func(anime db.Anime) bool {
					return reflect.DeepEqual(anime, dbAnime)
				}) {
					results = append(results, dbAnime)
				}
				continue
			}
		}
	}

	if len(results) == 1 {
		return &SearchSuccess{
			SearchAnime: anime,
			DBAnime:     results[0],
		}, nil, nil
	}

	if len(results) > 1 {
		return nil, &SearchMultiple{
			SearchAnime: anime,
			DBAnime:     results,
		}, nil
	}

	return nil, nil, &SearchFail{SearchAnime: anime}
}
