package exampleutil

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"time"

	"shindentoanilist/internal/anime"
	"shindentoanilist/internal/stadriver"
)

const (
	DefaultDatabasePath = "anime-offline-database.jsonl"
	DefaultShindenUser  = uint64(196402)
)

func NewDriver() *stadriver.Driver {
	driver, err := stadriver.New()
	Check(err)
	return driver
}

func EnsureDatabase(driver *stadriver.Driver, path string) anime.DatabaseInfo {
	info, err := driver.EnsureDatabase(path)
	Check(err)
	return info
}

func LoadShindenList(driver *stadriver.Driver, userID uint64) anime.ShindenListIndex {
	list, err := driver.LoadShindenList(userID)
	Check(err)
	return list
}

func LoadShindenEntries(driver *stadriver.Driver, entryIDs []uint64) []anime.ShindenEntry {
	entries, err := driver.GetLoadedShindenEntries(entryIDs)
	Check(err)
	return entries
}

func DatabaseByID(driver *stadriver.Driver, entryIDs []uint64) map[uint64]anime.DatabaseEntry {
	databaseEntries, err := driver.GetAnimeDatabaseEntries(UniqueIDs(entryIDs))
	Check(err)

	entries := make(map[uint64]anime.DatabaseEntry, len(databaseEntries))
	for _, entry := range databaseEntries {
		entries[entry.ID] = entry
	}
	return entries
}

func ShindenByID(driver *stadriver.Driver, entryIDs []uint64) map[uint64]anime.ShindenEntry {
	shindenEntries := LoadShindenEntries(driver, UniqueIDs(entryIDs))

	entries := make(map[uint64]anime.ShindenEntry, len(shindenEntries))
	for _, entry := range shindenEntries {
		entries[entry.ID] = entry
	}
	return entries
}

func UniqueIDs(entryIDs []uint64) []uint64 {
	seen := make(map[uint64]struct{}, len(entryIDs))
	result := make([]uint64, 0, len(entryIDs))

	for _, entryID := range entryIDs {
		if _, ok := seen[entryID]; ok {
			continue
		}

		seen[entryID] = struct{}{}
		result = append(result, entryID)
	}

	return result
}

func DatabasePathFlag() *string {
	return flag.String("database", DefaultDatabasePath, "anime offline database JSONL path")
}

func ShindenUserFlag() *uint64 {
	return flag.Uint64("user-id", DefaultShindenUser, "Shinden user ID")
}

func Check(err error) {
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func WriteJSON(path string, value any) {
	file, err := os.OpenFile(path, os.O_WRONLY|os.O_CREATE|os.O_EXCL, 0o644)
	Check(err)
	defer func() {
		Check(file.Close())
	}()

	encoder := json.NewEncoder(file)
	encoder.SetIndent("", "  ")
	Check(encoder.Encode(value))
}

func Elapsed(start time.Time) string {
	return time.Since(start).Round(10 * time.Millisecond).String()
}
