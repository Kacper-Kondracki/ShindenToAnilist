package main

import (
	"flag"
	"fmt"

	"shindentoanilist/examples/go/internal/exampleutil"
)

func main() {
	databasePath := exampleutil.DatabasePathFlag()
	flag.Parse()

	driver := exampleutil.NewDriver()
	defer driver.Close()

	info := exampleutil.EnsureDatabase(driver, *databasePath)
	if info.Updated {
		fmt.Printf("updated %s from %s (%s)\n", info.Path, info.Release, info.Sha256)
		return
	}

	fmt.Printf("%s is up to date (%s, %s)\n", info.Path, info.Release, info.Sha256)
}
