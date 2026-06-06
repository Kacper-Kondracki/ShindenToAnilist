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

func LoadShindenList(driver *stadriver.Driver, userID uint64) anime.ShindenList {
	list, err := driver.LoadShindenList(userID)
	Check(err)
	return list
}

func GetAnimeDatabase(driver *stadriver.Driver) anime.AnimeDatabase {
	database, err := driver.GetAnimeDatabase()
	Check(err)
	return database
}

func DatabaseByID(database anime.AnimeDatabase) map[uint64]anime.DatabaseEntry {
	entries := make(map[uint64]anime.DatabaseEntry, len(database.Entries))
	for _, entry := range database.Entries {
		entries[entry.ID] = entry
	}
	return entries
}

func ShindenByID(list anime.ShindenList) map[uint64]anime.ShindenEntry {
	entries := make(map[uint64]anime.ShindenEntry, len(list.Entries))
	for _, entry := range list.Entries {
		entries[entry.ID] = entry
	}
	return entries
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
