package main

import (
	"flag"
	"fmt"

	"shindentoanilist/examples/go/internal/exampleutil"
	"shindentoanilist/internal/anime"
)

func main() {
	databasePath := exampleutil.DatabasePathFlag()
	userID := exampleutil.ShindenUserFlag()
	flag.Parse()

	driver := exampleutil.NewDriver()
	defer driver.Close()

	exampleutil.EnsureDatabase(driver, *databasePath)
	shinden := exampleutil.LoadShindenList(driver, *userID)

	matches, err := driver.MatchLoadedShindenList(anime.MatchOptions{})
	exampleutil.Check(err)

	fmt.Println("The stateful driver exposes the strict matcher preset, not arbitrary matcher weights.")
	fmt.Printf("Preset score: %d/%d winners\n", matches.Winners, len(shinden.EntryIDs))
}
