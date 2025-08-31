package db

import (
	"bufio"
	"encoding/json"
	"errors"
	"net/http"
	"os"
	"shinden-to-anilist/lib"
)

func Get() (*Root, error) {
	db, err := Read()

	switch {
	case errors.Is(err, os.ErrNotExist):
		db, err = Download()
		if err != nil {
			return nil, err
		}

		file, err := os.Create("anime.json")
		if err != nil {
			return nil, err
		}
		defer file.Close()

		buf := bufio.NewWriter(file)
		err = json.NewEncoder(buf).Encode(db)
		if err != nil {
			return nil, err
		}

		err = buf.Flush()
		if err != nil {
			return nil, err
		}
	case err != nil:
		return nil, err
	}

	return db, nil
}

func Read() (*Root, error) {
	file, err := os.Open("anime.json")
	if err != nil {
		return nil, err
	}
	defer file.Close()
	buf := bufio.NewReader(file)

	var root Root
	if err = json.NewDecoder(buf).Decode(&root); err != nil {
		return nil, err
	}

	return &root, nil
}

func Download() (*Root, error) {
	const uri = "https://github.com/manami-project/anime-offline-database/releases/latest/download/anime-offline-database-minified.json"

	res, err := http.Get(uri)
	if err != nil {
		return nil, err
	}
	defer res.Body.Close()

	var root Root
	if err = json.NewDecoder(res.Body).Decode(&root); err != nil {
		return nil, err
	}

	return &root, nil
}

type AnimeSeason struct {
	Season string `json:"season"`
	Year   *int   `json:"year"`
}

type Anime struct {
	Sources     []string        `json:"sources"`
	Title       string          `json:"title"`
	Type        lib.AnimeType   `json:"type"`
	Episodes    int             `json:"episodes"`
	Status      lib.TitleStatus `json:"status"`
	AnimeSeason AnimeSeason     `json:"animeSeason"`
	Picture     string          `json:"picture"`
	Thumbnail   string          `json:"thumbnail"`
	Synonyms    []string        `json:"synonyms"`
	Relations   []string        `json:"relations"`
	Tags        []string        `json:"tags"`
}

type Root struct {
	LastUpdate string  `json:"lastUpdate"`
	Data       []Anime `json:"data"`
}
