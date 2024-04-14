package main

import (
	"context"
	"encoding/json"
	"fmt"
	"github.com/lithammer/fuzzysearch/fuzzy"
	"github.com/wailsapp/wails/v2/pkg/runtime"
	"shinden-to-anilist/lib/animezone"
	"shinden-to-anilist/lib/converter"
	"shinden-to-anilist/lib/db"
	"shinden-to-anilist/lib/searcher"
	"shinden-to-anilist/lib/shinden"
	"sort"
	"strconv"
	"strings"
	"sync"
)

// App struct
type App struct {
	ctx context.Context
}

// NewApp creates a new App application struct
func NewApp() *App {
	return &App{}
}

// startup is called when the app starts. The context is saved
// so we can call the runtime methods
func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
}

// Greet returns a greeting for the given name
func (a *App) Greet() string {
	return "Hello"
}

var (
	dbAnimeNames   []string
	onceAnimeNames sync.Once
)

func GetAnimeNames() []string {
	onceAnimeNames.Do(func() {
		dbAnimeNames = make([]string, 0)
		animeDb, err := GetAnimeDb()
		if err != nil {
			return
		}
		for _, anime := range animeDb.Data {
			dbAnimeNames = append(dbAnimeNames, anime.Title)
		}
	})
	return dbAnimeNames
}

var (
	animeDb     *db.Root
	onceAnimeDb sync.Once
)

func GetAnimeDb() (*db.Root, error) {
	var err error
	onceAnimeDb.Do(func() {
		animeDb, err = db.Get()
	})
	return animeDb, err
}

func (a *App) Search(name string) []db.Anime {
	searchNames := GetAnimeNames()
	results := make([]db.Anime, 0)
	animeDb, err := GetAnimeDb()
	if err != nil {
		return results
	}
	foundNames := fuzzy.RankFindNormalizedFold(name, searchNames)
	sort.Sort(foundNames)

	for i, foundName := range foundNames {
		if i >= 100 {
			break
		}
		for _, anime := range animeDb.Data {
			if anime.Title == foundName.Target {
				results = append(results, anime)
				break
			}
		}
	}

	return results
}

func (a *App) Export(fixed string) string {
	dialog, err := runtime.SaveFileDialog(a.ctx, runtime.SaveDialogOptions{
		DefaultFilename: "lista_anime.xml",
		Title:           "Wybierz miejsce zapisu listy",
	})

	var fixes []*db.Anime
	err = json.Unmarshal([]byte(fixed), &fixes)
	if err != nil {
		fmt.Println(err.Error())
		return "JSON_ERR"
	}

	success := results.SuccessAnime

	for i := 0; i < len(results.MultipleAnime); i++ {
		if fixes[i] != nil {
			success = append(success, searcher.SearchSuccess{
				SearchAnime: results.MultipleAnime[i].SearchAnime,
				DBAnime:     *fixes[i],
			})
		}
	}

	for i := 0; i < len(results.FailAnime); i++ {
		if fixes[i+len(results.MultipleAnime)] != nil {
			success = append(success, searcher.SearchSuccess{
				SearchAnime: results.FailAnime[i].SearchAnime,
				DBAnime:     *fixes[i+len(results.MultipleAnime)],
			})
		}
	}

	err = converter.Convert(dialog, success)
	if err != nil {
		return "EXPORT_ERR"
	}
	return ""
}

var results ConvertResultJSON

func (a *App) Convert(name string, site string) ConvertResultJSON {
	search := make([]searcher.SearchAnime, 0)

	switch site {
	case "shinden":
		shindenId, err := strconv.Atoi(name)
		if err != nil {
			split := strings.Split(name, "/")
			if len(split) > 0 {
				userFragment := strings.Split(split[len(split)-1], "-")
				if len(userFragment) > 0 {
					shindenId, err = strconv.Atoi(userFragment[0])
				}
			}
		}

		if err != nil {
			return ConvertResultJSON{
				Status: "NAME_ERR",
			}
		}

		list, err := shinden.GetList(shindenId, 99999, 0)
		if err != nil {
			return ConvertResultJSON{Status: "SHINDEN_ERR"}
		}
		for _, anime := range list {
			search = append(search, &anime)
		}
	case "animezone":
		split := strings.Split(name, "/")
		var username string
		if len(split) > 0 {
			username = split[len(split)-1]
		} else {
			username = name
		}

		list, err := animezone.GetList(username)
		if err != nil {
			return ConvertResultJSON{Status: "ANIMEZONE_ERR"}
		}
		for _, anime := range list {
			search = append(search, &anime)
		}
	default:
		return ConvertResultJSON{Status: "SITE_ERR"}
	}

	animeDB, err := GetAnimeDb()
	if err != nil {
		return ConvertResultJSON{Status: "DB_ERR"}
	}

	success, multiple, fail := searcher.Search(search, animeDB.Data)

	results = ConvertResultJSON{
		Status:        "OK",
		SuccessCount:  len(success),
		MultipleCount: len(multiple),
		FailCount:     len(fail),
		SuccessAnime:  success,
		MultipleAnime: multiple,
		FailAnime:     fail,
	}

	return results
}

type ConvertResultJSON struct {
	Status        string                    `json:"status"`
	SuccessCount  int                       `json:"successCount"`
	MultipleCount int                       `json:"multipleCount"`
	FailCount     int                       `json:"failCount"`
	SuccessAnime  []searcher.SearchSuccess  `json:"successAnime"`
	MultipleAnime []searcher.SearchMultiple `json:"multipleAnime"`
	FailAnime     []searcher.SearchFail     `json:"failAnime"`
}
