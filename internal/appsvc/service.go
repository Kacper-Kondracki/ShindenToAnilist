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

type Service struct {
	mu       sync.RWMutex
	ensureMu sync.Mutex
	driver   *stadriver.Driver
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

	return driver.Close()
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

func (s *Service) LoadShindenList(userID int) (ShindenList, error) {
	if userID <= 0 || int64(userID) > maxShindenUserID {
		return ShindenList{}, fmt.Errorf("shinden user id must be between 1 and %d", maxShindenUserID)
	}

	driver, err := s.activeDriver()
	if err != nil {
		return ShindenList{}, err
	}

	return driver.LoadShindenList(uint64(userID))
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
