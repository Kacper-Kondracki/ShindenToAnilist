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

	started := time.Now()
	info := exampleutil.EnsureDatabase(driver, *databasePath)

	fmt.Printf("database: %s\n", info.Path)
	fmt.Printf("took %s\n", exampleutil.Elapsed(started))

	if *outputPath != "" {
		exampleutil.WriteJSON(*outputPath, info)
	}
}
