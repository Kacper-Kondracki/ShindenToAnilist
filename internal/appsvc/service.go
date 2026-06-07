package appsvc

import (
	"context"
	"errors"
	"fmt"
	"path/filepath"
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
	mu       sync.RWMutex
	ensureMu sync.Mutex
	driver   *stadriver.Driver
}

func New() *Service {
	return &Service{}
}

// Startup creates the stateful Rust driver used by Wails-visible service calls.
// User input validation stays in this package before calls cross into stadriver.
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

// EnsureDatabase serializes update/load attempts so concurrent frontend calls
// cannot ask the driver to mutate the offline database state in parallel.
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
	driverUserID, err := validateShindenUserID(userID)
	if err != nil {
		return ShindenListIndex{}, err
	}

	driver, err := s.activeDriver()
	if err != nil {
		return ShindenListIndex{}, err
	}

	return driver.LoadShindenList(driverUserID)
}

func (s *Service) GetLoadedShindenEntryIDs(view string) (ShindenListIndex, error) {
	if err := validateShindenListView(view); err != nil {
		return ShindenListIndex{}, err
	}

	driver, err := s.activeDriver()
	if err != nil {
		return ShindenListIndex{}, err
	}

	return driver.GetLoadedShindenEntryIDs(view)
}

func (s *Service) GetLoadedShindenEntries(entryIDs []uint64) ([]ShindenEntry, error) {
	if len(entryIDs) == 0 {
		return []ShindenEntry{}, nil
	}

	if err := validatePositiveIDs(entryIDs, "shinden entry id"); err != nil {
		return nil, err
	}

	driver, err := s.activeDriver()
	if err != nil {
		return nil, err
	}

	return driver.GetLoadedShindenEntries(entryIDs)
}

func (s *Service) GetAnimeDatabaseEntries(entryIDs []uint64) ([]DatabaseEntry, error) {
	if len(entryIDs) == 0 {
		return []DatabaseEntry{}, nil
	}
	if err := validatePositiveIDs(entryIDs, "database entry id"); err != nil {
		return nil, err
	}

	driver, err := s.activeDriver()
	if err != nil {
		return nil, err
	}

	return driver.GetAnimeDatabaseEntries(entryIDs)
}

func (s *Service) MatchLoadedShindenList(options MatchOptions) (MatchListResult, error) {
	if err := validateMatchOptions(options); err != nil {
		return MatchListResult{}, err
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
	if err := validateSearchOptions(options); err != nil {
		return SearchResult{}, err
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
	if err := validateMatchQueryOptions(options); err != nil {
		return MatchResult{}, err
	}

	driver, err := s.activeDriver()
	if err != nil {
		return MatchResult{}, err
	}

	return driver.MatchQuery(query, options)
}

func (s *Service) ExportMatches(matches []MatchSelection) (ExportResult, error) {
	if err := validateMatchSelections(matches); err != nil {
		return ExportResult{}, err
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

func validateShindenUserID(userID int) (uint64, error) {
	if userID <= 0 || int64(userID) > maxShindenUserID {
		return 0, fmt.Errorf("shinden user id must be between 1 and %d", maxShindenUserID)
	}

	return uint64(userID), nil
}

func validateShindenListView(view string) error {
	switch view {
	case "manual", "automatic", "all":
		return nil
	default:
		return errors.New("shinden list view must be manual, automatic, or all")
	}
}

func validatePositiveIDs(ids []uint64, label string) error {
	for _, id := range ids {
		if id == 0 {
			return fmt.Errorf("%s must be positive", label)
		}
	}

	return nil
}

func validateSearchOptions(options SearchOptions) error {
	if err := validateSearchMode(options.Mode); err != nil {
		return err
	}
	if options.Limit < 0 {
		return errors.New("search limit must not be negative")
	}
	if options.Threshold < 0 {
		return errors.New("search threshold must not be negative")
	}

	return nil
}

func validateSearchMode(mode string) error {
	switch mode {
	case "", "fuzzy", "strict":
		return nil
	default:
		return errors.New("search mode must be empty, fuzzy, or strict")
	}
}

func validateMatchOptions(options MatchOptions) error {
	if options.CandidateLimit < 0 {
		return errors.New("candidate limit must not be negative")
	}
	if options.SearchThreshold < 0 {
		return errors.New("search threshold must not be negative")
	}
	if options.ResultLimit != nil && *options.ResultLimit < 0 {
		return errors.New("result limit must not be negative")
	}

	return nil
}

func validateMatchQueryOptions(options MatchQueryOptions) error {
	if err := validateSearchOptions(options.Search); err != nil {
		return err
	}
	if options.ResultLimit != nil && *options.ResultLimit < 0 {
		return errors.New("result limit must not be negative")
	}

	return nil
}

func validateMatchSelections(matches []MatchSelection) error {
	if len(matches) == 0 {
		return errors.New("at least one match selection is required")
	}
	for _, match := range matches {
		if match.ShindenID == 0 {
			return errors.New("match selection shinden id must be positive")
		}
		if match.DatabaseID == 0 {
			return errors.New("match selection database id must be positive")
		}
	}

	return nil
}
