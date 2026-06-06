package appsvc

import (
	"context"
	"errors"
	"fmt"
	"path/filepath"
	"slices"
	"sync"

	"github.com/wailsapp/wails/v3/pkg/application"

	"shindentoanilist/internal/anime"
	"shindentoanilist/internal/stadriver"
)

const (
	maxShindenUserID = int64(1<<53 - 1)
	appDataDirName   = "ShindenToAnilist"
	databaseFileName = "anime-offline-database.jsonl"
)

type DatabaseInfo = anime.DatabaseInfo
type AnimeDatabase = anime.AnimeDatabase
type ShindenList = anime.ShindenList
type ShindenListIndex = anime.ShindenListIndex
type ShindenEntry = anime.ShindenEntry
type DatabaseEntry = anime.DatabaseEntry
type SearchOptions = anime.SearchOptions
type MatchOptions = anime.MatchOptions
type MatchQueryOptions = anime.MatchQueryOptions
type SearchResult = anime.SearchResult
type MatchResult = anime.MatchResult
type MatchListResult = anime.MatchListResult
type MatchSelection = anime.MatchSelection
type ExportResult = anime.ExportResult

type Service struct {
	mu                   sync.RWMutex
	ensureMu             sync.Mutex
	loadedShindenEntries map[uint64]anime.ShindenEntry
	loadedShindenOrder   []uint64
	driver               *stadriver.Driver
}

func New() *Service {
	return &Service{}
}

func (s *Service) Startup(ctx context.Context) error {
	if err := ctx.Err(); err != nil {
		return err
	}

	driver, err := stadriver.New()
	if err != nil {
		return err
	}

	s.mu.Lock()
	defer s.mu.Unlock()

	if s.driver != nil {
		return driver.Close()
	}

	s.driver = driver
	return nil
}

func (s *Service) Shutdown() error {
	s.mu.Lock()
	driver := s.driver
	s.driver = nil
	s.mu.Unlock()

	if driver == nil {
		return nil
	}

	driver.Abort()
	go func() {
		_ = driver.Close()
	}()
	return nil
}

func (s *Service) AppName() string {
	return "ShindenToAnilist"
}

func (s *Service) EnsureDatabase() (DatabaseInfo, error) {
	s.ensureMu.Lock()
	defer s.ensureMu.Unlock()

	driver, err := s.activeDriver()
	if err != nil {
		return DatabaseInfo{}, err
	}

	return driver.EnsureDatabase(databasePath())
}

func (s *Service) LoadShindenList(userID int) (ShindenListIndex, error) {
	if userID <= 0 || int64(userID) > maxShindenUserID {
		return ShindenListIndex{}, fmt.Errorf("shinden user id must be between 1 and %d", maxShindenUserID)
	}

	driver, err := s.activeDriver()
	if err != nil {
		return ShindenListIndex{}, err
	}

	list, err := driver.LoadShindenList(uint64(userID))
	if err != nil {
		return ShindenListIndex{}, err
	}

	entries := make(map[uint64]anime.ShindenEntry, len(list.Entries))
	entryIDs := make([]uint64, 0, len(list.Entries))

	for _, entry := range list.Entries {
		entries[entry.ID] = entry
		entryIDs = append(entryIDs, entry.ID)
	}
	slices.SortStableFunc(entryIDs, func(leftID, rightID uint64) int {
		left := entries[leftID]
		right := entries[rightID]

		if left.PremiereDate == right.PremiereDate {
			return 0
		}
		if left.PremiereDate == nil {
			return 1
		}
		if right.PremiereDate == nil {
			return -1
		}
		if *left.PremiereDate > *right.PremiereDate {
			return -1
		}
		if *left.PremiereDate < *right.PremiereDate {
			return 1
		}
		return 0
	})

	s.mu.Lock()
	s.loadedShindenEntries = entries
	s.loadedShindenOrder = entryIDs
	s.mu.Unlock()

	return ShindenListIndex{EntryIDs: entryIDs}, nil
}

func (s *Service) GetAnimeDatabase() (AnimeDatabase, error) {
	driver, err := s.activeDriver()
	if err != nil {
		return AnimeDatabase{}, err
	}

	return driver.GetAnimeDatabase()
}

func (s *Service) GetLoadedShindenEntries(entryIDs []uint64) ([]ShindenEntry, error) {
	if len(entryIDs) == 0 {
		return []ShindenEntry{}, nil
	}

	s.mu.RLock()
	entriesByID := s.loadedShindenEntries
	s.mu.RUnlock()

	if len(entriesByID) == 0 {
		return nil, errors.New("shinden list is not loaded")
	}

	entries := make([]ShindenEntry, 0, len(entryIDs))
	for _, entryID := range entryIDs {
		if entryID == 0 {
			return nil, errors.New("shinden entry id must be positive")
		}

		entry, ok := entriesByID[entryID]
		if !ok {
			return nil, fmt.Errorf("shinden entry %d is not loaded", entryID)
		}

		entries = append(entries, entry)
	}

	return entries, nil
}

func (s *Service) GetAnimeDatabaseEntries(entryIDs []uint64) ([]DatabaseEntry, error) {
	if len(entryIDs) == 0 {
		return []DatabaseEntry{}, nil
	}
	for _, entryID := range entryIDs {
		if entryID == 0 {
			return nil, errors.New("database entry id must be positive")
		}
	}

	driver, err := s.activeDriver()
	if err != nil {
		return nil, err
	}

	return driver.GetAnimeDatabaseEntries(entryIDs)
}

func (s *Service) MatchLoadedShindenList(options MatchOptions) (MatchListResult, error) {
	if options.CandidateLimit < 0 {
		return MatchListResult{}, errors.New("candidate limit must not be negative")
	}
	if options.SearchThreshold < 0 {
		return MatchListResult{}, errors.New("search threshold must not be negative")
	}
	if options.ResultLimit != nil && *options.ResultLimit < 0 {
		return MatchListResult{}, errors.New("result limit must not be negative")
	}

	driver, err := s.activeDriver()
	if err != nil {
		return MatchListResult{}, err
	}

	return driver.MatchLoadedShindenList(options)
}

func (s *Service) SearchAnime(query string, options SearchOptions) (SearchResult, error) {
	if query == "" {
		return SearchResult{}, errors.New("search query must not be empty")
	}
	if options.Limit < 0 {
		return SearchResult{}, errors.New("search limit must not be negative")
	}
	if options.Threshold < 0 {
		return SearchResult{}, errors.New("search threshold must not be negative")
	}

	driver, err := s.activeDriver()
	if err != nil {
		return SearchResult{}, err
	}

	return driver.SearchAnime(query, options)
}

func (s *Service) MatchQuery(query string, options MatchQueryOptions) (MatchResult, error) {
	if query == "" {
		return MatchResult{}, errors.New("match query must not be empty")
	}
	if options.Search.Limit < 0 {
		return MatchResult{}, errors.New("search limit must not be negative")
	}
	if options.Search.Threshold < 0 {
		return MatchResult{}, errors.New("search threshold must not be negative")
	}
	if options.ResultLimit != nil && *options.ResultLimit < 0 {
		return MatchResult{}, errors.New("result limit must not be negative")
	}

	driver, err := s.activeDriver()
	if err != nil {
		return MatchResult{}, err
	}

	return driver.MatchQuery(query, options)
}

func (s *Service) ExportMatches(matches []MatchSelection) (ExportResult, error) {
	if len(matches) == 0 {
		return ExportResult{}, errors.New("at least one match selection is required")
	}
	for _, match := range matches {
		if match.ShindenID == 0 {
			return ExportResult{}, errors.New("match selection shinden id must be positive")
		}
		if match.DatabaseID == 0 {
			return ExportResult{}, errors.New("match selection database id must be positive")
		}
	}

	path, err := application.Get().Dialog.SaveFile().
		SetMessage("Export MAL XML").
		SetFilename("shinden-to-anilist.xml").
		AddFilter("MyAnimeList XML", "*.xml").
		PromptForSingleSelection()
	if err != nil {
		return ExportResult{}, err
	}
	if path == "" {
		return ExportResult{Cancelled: true}, nil
	}

	driver, err := s.activeDriver()
	if err != nil {
		return ExportResult{}, err
	}

	return driver.ExportMatches(path, matches)
}

func (s *Service) activeDriver() (*stadriver.Driver, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	if s.driver == nil {
		return nil, errors.New("driver is not ready")
	}

	return s.driver, nil
}

func databasePath() string {
	return filepath.Join(application.Path(application.PathDataHome), appDataDirName, databaseFileName)
}
