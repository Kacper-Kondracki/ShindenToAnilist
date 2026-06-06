package main

import (
	"flag"
	"fmt"
	"time"

	"shindentoanilist/examples/go/internal/exampleutil"
	"shindentoanilist/internal/anime"
)

func main() {
	databasePath := exampleutil.DatabasePathFlag()
	userID := exampleutil.ShindenUserFlag()
	resultLimit := flag.Int("result-limit", 0, "optional scored candidate limit per Shinden entry")
	flag.Parse()

	driver := exampleutil.NewDriver()
	defer driver.Close()

	exampleutil.EnsureDatabase(driver, *databasePath)

	shinden := exampleutil.LoadShindenList(driver, *userID)

	options := anime.MatchOptions{}
	if *resultLimit > 0 {
		options.ResultLimit = resultLimit
	}

	started := time.Now()
	matches, err := driver.MatchLoadedShindenList(options)
	exampleutil.Check(err)
	elapsed := exampleutil.Elapsed(started)
	shindenEntryIDs := make([]uint64, 0, len(matches.Entries))
	databaseEntryIDs := make([]uint64, 0)
	for _, match := range matches.Entries {
		shindenEntryIDs = append(shindenEntryIDs, match.ShindenID)
		for _, item := range match.Result.Items {
			databaseEntryIDs = append(databaseEntryIDs, item.ID)
		}
	}
	shindenEntries := exampleutil.ShindenByID(driver, shindenEntryIDs)
	databaseEntries := exampleutil.DatabaseByID(driver, databaseEntryIDs)

	for _, match := range matches.Entries {
		shindenEntry := shindenEntries[match.ShindenID]
		fmt.Printf("=== %s ===\n", shindenEntry.Title)
		for _, item := range match.Result.Items {
			databaseEntry := databaseEntries[item.ID]
			label := ""
			if match.Result.Winner != nil && match.Result.Winner.ID == item.ID {
				label = "WIN"
			}
			fmt.Printf("[%.2f %3s] %s\n", item.Score, label, databaseEntry.Title)
		}
	}

	fmt.Printf("TOOK       : %s\n", elapsed)
	fmt.Printf("ENTRIES    : %d\n", len(shinden.EntryIDs))
	fmt.Printf("HAS TOP    : %d/%d\n", matches.HasTop, matches.Total)
	fmt.Printf("HAS WINNER : %d/%d\n", matches.Winners, matches.Total)
}
