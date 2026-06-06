package main

import (
	"flag"
	"fmt"
	"time"

	"shindentoanilist/examples/go/internal/exampleutil"
)

func main() {
	userID := exampleutil.ShindenUserFlag()
	outputPath := flag.String("out", "shinden-test.json", "optional JSON output path")
	flag.Parse()

	driver := exampleutil.NewDriver()
	defer driver.Close()

	started := time.Now()
	list := exampleutil.LoadShindenList(driver, *userID)

	fmt.Printf("%d entries\n", len(list.EntryIDs))
	fmt.Printf("took %s\n", exampleutil.Elapsed(started))

	if *outputPath != "" {
		entries := exampleutil.LoadShindenEntries(driver, list.EntryIDs)
		exampleutil.WriteJSON(*outputPath, entries)
	}
}
