package main

import (
	"fmt"
	"os"
)

func main() {
	fmt.Fprintln(os.Stderr, "zstd database updates are not exposed by the stateful driver API; use the Rust core example for this path.")
	os.Exit(1)
}
