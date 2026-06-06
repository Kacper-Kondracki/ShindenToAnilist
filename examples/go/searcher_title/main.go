package main

import (
	"flag"
	"fmt"

	"shindentoanilist/examples/go/internal/exampleutil"
	"shindentoanilist/internal/anime"
)

func main() {
	databasePath := exampleutil.DatabasePathFlag()
	query := flag.String("query", "shingeki no kyojin", "title query")
	flag.Parse()

	driver := exampleutil.NewDriver()
	defer driver.Close()

	exampleutil.EnsureDatabase(driver, *databasePath)
	database := exampleutil.GetAnimeDatabase(driver)
	entries := exampleutil.DatabaseByID(database)

	results, err := driver.SearchAnime(*query, anime.SearchOptions{Mode: "fuzzy"})
	exampleutil.Check(err)

	for _, item := range results.Items {
		entry := entries[item.ID]
		fmt.Printf("[%.2f] %s\n", item.Score, entry.Title)
	}
}
