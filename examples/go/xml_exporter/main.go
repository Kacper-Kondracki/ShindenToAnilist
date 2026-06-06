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
	outputPath := flag.String("out", "xml-test.xml", "MAL-compatible XML output path")
	step := flag.Int("step", 100, "export every nth winner, matching the Rust example")
	flag.Parse()

	driver := exampleutil.NewDriver()
	defer driver.Close()

	exampleutil.EnsureDatabase(driver, *databasePath)
	exampleutil.LoadShindenList(driver, *userID)

	matches, err := driver.MatchLoadedShindenList(anime.MatchOptions{})
	exampleutil.Check(err)

	selections := make([]anime.MatchSelection, 0, matches.Winners)
	for i, match := range matches.Entries {
		if *step > 1 && i%*step != 0 {
			continue
		}
		if match.Result.Winner == nil {
			continue
		}
		selections = append(selections, anime.MatchSelection{
			ShindenID:  match.ShindenID,
			DatabaseID: match.Result.Winner.ID,
		})
	}

	started := time.Now()
	result, err := driver.ExportMatches(*outputPath, selections)
	exampleutil.Check(err)

	fmt.Printf("exported %d entries to %s\n", result.ExportedCount, result.Path)
	fmt.Printf("took %s\n", exampleutil.Elapsed(started))
}
