package main

import (
	"flag"
	"fmt"

	"shindentoanilist/examples/go/internal/exampleutil"
	"shindentoanilist/internal/anime"
)

func main() {
	databasePath := exampleutil.DatabasePathFlag()
	limit := flag.Int("limit", 5, "number of scored candidates to print per query")
	flag.Parse()

	driver := exampleutil.NewDriver()
	defer driver.Close()

	exampleutil.EnsureDatabase(driver, *databasePath)
	database := exampleutil.GetAnimeDatabase(driver)
	entries := exampleutil.DatabaseByID(database)

	queries := []string{
		"oshi no ko 1",
		"oshi no ko 2",
		"oshi no ko 3",
		"shingeki no kyojin 1",
		"shingeki no kyojin 2",
		"shingeki no kyojin 3",
	}

	for _, query := range queries {
		fmt.Printf("=== %s ===\n", query)
		result, err := driver.MatchQuery(query, anime.MatchQueryOptions{
			Search: anime.SearchOptions{
				Mode: "fuzzy",
			},
			ResultLimit: limit,
		})
		exampleutil.Check(err)

		for _, item := range result.Items {
			entry := entries[item.ID]
			fmt.Printf("[%.2f] %s\n", item.Score.FinalScore, entry.Title)
		}
	}
}
