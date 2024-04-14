package animezone

import (
	"encoding/json"
	"fmt"
	"github.com/gocolly/colly"
	"shinden-to-anilist/lib"
	"shinden-to-anilist/lib/searcher"
	"slices"
	"strconv"
	"strings"
)

type Anime struct {
	Title       string
	Year        int
	TitleStatus *lib.TitleStatus
	AnimeType   *lib.AnimeType
	Rate        *int
	WatchStatus lib.WatchStatus
	Source      string
}

func (az Anime) GetTitle() string {
	return az.Title
}

func (az Anime) GetYear() *int {
	return &az.Year
}

func (az Anime) GetTitleStatus() *lib.TitleStatus {
	return az.TitleStatus
}

func (az Anime) GetEpisodes() *int {
	return nil
}

func (az Anime) GetAnimeType() *lib.AnimeType {
	return az.AnimeType
}

func (az Anime) GetSource() *string {
	return &az.Source
}

func (az Anime) GetNotes() *string {
	return nil
}

func (az *Anime) MarshalJSON() ([]byte, error) {
	a := searcher.SearchAnime(az)
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

func GetList(user string) ([]Anime, error) {
	type ScrapResult struct {
		anime []Anime
		error error
	}

	results := make([]Anime, 0)
	completedChan := make(chan ScrapResult)
	watchingChan := make(chan ScrapResult)
	planningChan := make(chan ScrapResult)

	go func() {
		results := make([]Anime, 0)
		var res []Anime
		var err error
		for i := 1; i < 500; i++ {
			res, err = Scrape(user, i, lib.WatchCompleted)
			if err != nil {
				if err.Error() == "Not Found" {
					break
				}
				completedChan <- ScrapResult{nil, err}
				close(completedChan)
				return
			}
			results = append(results, res...)
		}
		completedChan <- ScrapResult{results, nil}
		close(completedChan)
	}()

	go func() {
		results := make([]Anime, 0)
		var res []Anime
		var err error
		for i := 1; i < 500; i++ {
			res, err = Scrape(user, i, lib.WatchInProgress)
			if err != nil {
				if err.Error() == "Not Found" {
					break
				}
				watchingChan <- ScrapResult{nil, err}
				close(watchingChan)
				return
			}
			results = append(results, res...)
		}
		watchingChan <- ScrapResult{results, nil}
		close(watchingChan)
	}()

	go func() {
		results := make([]Anime, 0)
		var res []Anime
		var err error
		for i := 1; i < 500; i++ {
			res, err = Scrape(user, i, lib.WatchPlan)
			if err != nil {
				if err.Error() == "Not Found" {
					break
				}
				planningChan <- ScrapResult{nil, err}
				close(planningChan)
				return
			}
			results = append(results, res...)
		}
		planningChan <- ScrapResult{results, nil}
		close(planningChan)
	}()

	completedResult := <-completedChan
	watchingResult := <-watchingChan
	planningResult := <-planningChan

	if completedResult.error != nil {
		return nil, completedResult.error
	}

	if watchingResult.error != nil {
		return nil, watchingResult.error
	}

	if planningResult.error != nil {
		return nil, watchingResult.error
	}

	completedResult.anime = slices.DeleteFunc(completedResult.anime, func(animeCompleted Anime) bool {
		return slices.ContainsFunc(watchingResult.anime, func(animeWatching Anime) bool {
			return animeWatching.Source == animeCompleted.Source
		})

	})

	results = append(results, completedResult.anime...)
	results = append(results, watchingResult.anime...)
	results = append(results, planningResult.anime...)

	return results, nil
}

func Scrape(user string, page int, status lib.WatchStatus) ([]Anime, error) {
	var uri string
	switch status {
	case lib.WatchInProgress:
		uri = "https://www.animezone.pl/user/%s/watching?page=%d"
	case lib.WatchPlan:
		uri = "https://www.animezone.pl/user/%s/plans?page=%d"
	case lib.WatchCompleted:
		uri = "https://www.animezone.pl/user/%s/rated?page=%d"
	default:
		panic("unhandled default case")
	}

	results := make([]Anime, 0)

	url := fmt.Sprintf(uri, user, page)
	c := colly.NewCollector()

	c.OnHTML(".well.well-sm.categories.col-xs-12", func(e *colly.HTMLElement) {
		anime := Anime{
			Title:       e.ChildText("a:not(.image)"),
			WatchStatus: status,
			Source:      fmt.Sprintf("https://www.animezone.pl%s", e.ChildAttr("a.image", "href")),
		}

		rateText := e.ChildText("span.pull-right.label.label-dark")
		splitRate := strings.Split(rateText, " ")
		if len(splitRate) == 2 {
			splitRate := strings.Split(splitRate[1], "/")
			if len(splitRate) == 2 {
				splitRateInt, _ := strconv.Atoi(splitRate[0])
				anime.Rate = &splitRateInt
			}
		}

		c2 := colly.NewCollector()
		c2.OnHTML(".panel.panel-default.category-description", func(e *colly.HTMLElement) {
			year := e.ChildText("div.panel-body.category-description-body>div.basic-info.pull-right>table>tbody>tr:nth-child(2)>td:nth-child(2)")
			yearInt, _ := strconv.Atoi(year)
			anime.Year = yearInt

			titleStatus := e.ChildText("table:nth-child(4)>tbody>tr:nth-child(6)>td:nth-child(2)")
			switch titleStatus {
			case "Nadchodzące":
				tmp := lib.TitleNotYetAired
				anime.TitleStatus = &tmp
			case "Zakończone":
				tmp := lib.TitleFinishedAiring
				anime.TitleStatus = &tmp
			case "Emitowane":
				tmp := lib.TitleCurrentlyAiring
				anime.TitleStatus = &tmp
			}

			animeType := e.ChildText("table:nth-child(4)>tbody>tr:nth-child(3)>td:nth-child(2)")

			switch animeType {
			case "TV":
				tmp := lib.AnimeTV
				anime.AnimeType = &tmp
			case "OVA":
				tmp := lib.AnimeOVA
				anime.AnimeType = &tmp
			case "Film":
				tmp := lib.AnimeMovie
				anime.AnimeType = &tmp

			}
		})
		c2.Visit(anime.Source)

		results = append(results, anime)
	})
	err := c.Visit(url)
	if err != nil {
		return nil, err
	}

	return results, nil
}
