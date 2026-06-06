package main

import (
	"flag"
	"fmt"
	"time"

	"shindentoanilist/examples/go/internal/exampleutil"
)

func main() {
	databasePath := exampleutil.DatabasePathFlag()
	flag.Parse()

	driver := exampleutil.NewDriver()
	defer driver.Close()

	started := time.Now()
	exampleutil.EnsureDatabase(driver, *databasePath)

	fmt.Printf("took: %s\n", exampleutil.Elapsed(started))
}
