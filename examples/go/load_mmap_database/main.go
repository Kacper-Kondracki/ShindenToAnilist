package main

import (
	"flag"
	"fmt"
	"time"

	"shindentoanilist/examples/go/internal/exampleutil"
)

func main() {
	databasePath := exampleutil.DatabasePathFlag()
	outputPath := flag.String("out", "db-test.json", "optional JSON output path")
	flag.Parse()

	driver := exampleutil.NewDriver()
	defer driver.Close()

	exampleutil.EnsureDatabase(driver, *databasePath)

	started := time.Now()
	database := exampleutil.GetAnimeDatabase(driver)

	fmt.Printf("%d entries\n", len(database.Entries))
	fmt.Printf("took %s\n", exampleutil.Elapsed(started))

	if *outputPath != "" {
		exampleutil.WriteJSON(*outputPath, database)
	}
}
