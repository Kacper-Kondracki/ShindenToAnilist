package tests

import (
	"shinden-to-anilist/lib/converter"
	"shinden-to-anilist/lib/db"
	"shinden-to-anilist/lib/searcher"
	"shinden-to-anilist/lib/shinden"
	"testing"
)

func TestFullShinden(t *testing.T) {
	list, err := shinden.GetList(581621, 9999, 0)
	if err != nil {
		t.Error(err)
	}

	animeDB, err := db.Get()
	if err != nil {
		t.Error(err)
	}

	search := make([]searcher.SearchAnime, len(list))
	for i, anime := range list {
		search[i] = &anime
	}

	success, multiple, fail := searcher.Search(search, animeDB.Data)

	results := make([]searcher.SearchSuccess, 0)

	t.Log("Success")
	for _, result := range success {
		t.Log(result.DBAnime.Title)
		results = append(results, result)
	}
	t.Log("Multiple")
	for _, result := range multiple {
		for i, anime := range result.DBAnime {
			t.Log(i, anime.Title)
		}
	}
	t.Log("Fail")
	for _, result := range fail {
		t.Log(result.SearchAnime.GetTitle())
	}

	err = converter.Convert("lista_anime.xml", results)
	if err != nil {
		t.Error(err)
	}
}
