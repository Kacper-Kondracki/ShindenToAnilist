package converter

import (
	"bufio"
	"encoding/xml"
	"errors"
	"os"
	"shinden-to-anilist/lib"
	"shinden-to-anilist/lib/animezone"
	"shinden-to-anilist/lib/searcher"
	"shinden-to-anilist/lib/shinden"
	"strings"
)

func Convert(fileName string, anime []searcher.SearchSuccess) error {
	file, err := os.Create(fileName)
	if err != nil {
		return err
	}
	defer file.Close()
	bufferWriter := bufio.NewWriter(file)

	_, err = bufferWriter.WriteString("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n")
	if err != nil {
		return err
	}

	animeList := animeListXAML{
		Info: infoXAML{
			UserExportType: 1,
		},
		Animes: make([]animeXAML, 0),
	}
	for _, searchAnime := range anime {
		convertAnime := animeXAML{
			ID:              "0",
			WatchedEpisodes: 0,
			StartDate:       "0000-00-00",
			FinishDate:      "0000-00-00",
			Score:           0,
			Status:          "",
			Update:          1,
		}

		for _, source := range searchAnime.DBAnime.Sources {
			if strings.Contains(source, "myanimelist") {
				split := strings.Split(source, "/")
				convertAnime.ID = split[len(split)-1]
				break
			}
		}

		switch searchAnime.SearchAnime.(type) {
		case *shinden.Anime:
			shindenAnime := searchAnime.SearchAnime.(*shinden.Anime)

			convertAnime.WatchedEpisodes = shindenAnime.WatchedEspisodesCnt
			if shindenAnime.RateTotal != nil {
				convertAnime.Score = *shindenAnime.RateTotal
			}

			if shindenAnime.GetNotes() != nil {
				convertAnime.Comments = *shindenAnime.GetNotes()
			}

			switch shindenAnime.WatchStatus {
			case lib.WatchSkip:
				convertAnime.Status = "Dropped"
			case lib.WatchCompleted:
				convertAnime.Status = "Completed"
			case lib.WatchDropped:
				convertAnime.Status = "Dropped"
			case lib.WatchInProgress:
				convertAnime.Status = "Watching"
			case lib.WatchHold:
				convertAnime.Status = "On-Hold"
			case lib.WatchPlan:
				convertAnime.Status = "Plan to Watch"
			default:
				return errors.New("watch status out of range")
			}
		case *animezone.Anime:
			animeZoneAnime := searchAnime.SearchAnime.(*animezone.Anime)
			switch animeZoneAnime.WatchStatus {
			case lib.WatchCompleted:
				convertAnime.Status = "Completed"
				convertAnime.WatchedEpisodes = searchAnime.DBAnime.Episodes
			case lib.WatchInProgress:
				convertAnime.Status = "Watching"
			case lib.WatchPlan:
				convertAnime.Status = "Plan to Watch"
			default:
				return errors.New("watch status out of range")
			}

		}

		animeList.Animes = append(animeList.Animes, convertAnime)
	}

	encoder := xml.NewEncoder(bufferWriter)
	encoder.Indent("", "  ")
	err = encoder.Encode(animeList)
	if err != nil {
		return err
	}

	return err
}

type infoXAML struct {
	UserExportType int `xml:"user_export_type"`
}

type animeXAML struct {
	ID              string `xml:"series_animedb_id"`
	WatchedEpisodes int    `xml:"my_watched_episodes"`
	StartDate       string `xml:"my_start_date"`
	FinishDate      string `xml:"my_finish_date"`
	Score           int    `xml:"my_score"`
	Status          string `xml:"my_status"`
	Update          int    `xml:"update_on_import"`
	Comments        string `xml:"my_comments"`
}

type animeListXAML struct {
	XMLName xml.Name    `xml:"myanimelist"`
	Info    infoXAML    `xml:"myinfo"`
	Animes  []animeXAML `xml:"anime"`
}
