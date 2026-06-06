package main

import (
	"context"
	"errors"
	"fmt"
	"path/filepath"

	"sync"

	"github.com/wailsapp/wails/v3/pkg/application"
)

type AppService struct {
	mu       sync.RWMutex
	ensureMu sync.Mutex
	driver   *Driver
}

const (
	maxCounterAmount = int64(1<<32 - 1)
	appDataDirName   = "ShindenToAnilist"
	databaseFileName = "anime-offline-database.jsonl"
)

func NewAppService() *AppService {
	return &AppService{}
}

func (s *AppService) ServiceStartup(ctx context.Context, _ application.ServiceOptions) error {
	if err := ctx.Err(); err != nil {
		return err
	}

	driver, err := NewDriver()
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

func (s *AppService) ServiceShutdown() error {
	s.mu.Lock()
	driver := s.driver
	s.driver = nil
	s.mu.Unlock()

	if driver == nil {
		return nil
	}

	return driver.Close()
}

func (s *AppService) AppName() string {
	return "ShindenToAnilist"
}

func (s *AppService) CounterValue() (int, error) {
	driver, err := s.activeDriver()
	if err != nil {
		return 0, err
	}

	value, err := driver.CounterValue()
	if err != nil {
		return 0, err
	}

	return int(value), nil
}

func (s *AppService) IncrementCounter() (int, error) {
	return s.IncrementCounterBy(1)
}

func (s *AppService) IncrementCounterBy(amount int) (int, error) {
	if amount < 0 || int64(amount) > maxCounterAmount {
		return 0, fmt.Errorf("counter amount must be between 0 and %d", maxCounterAmount)
	}

	driver, err := s.activeDriver()
	if err != nil {
		return 0, err
	}

	value, err := driver.IncrementCounter(uint32(amount))
	if err != nil {
		return 0, err
	}

	return int(value), nil
}

func (s *AppService) EnsureDatabase() (DatabaseInfo, error) {
	s.ensureMu.Lock()
	defer s.ensureMu.Unlock()

	driver, err := s.activeDriver()
	if err != nil {
		return DatabaseInfo{}, err
	}

	return driver.EnsureDatabase(databasePath())
}

func (s *AppService) activeDriver() (*Driver, error) {
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
