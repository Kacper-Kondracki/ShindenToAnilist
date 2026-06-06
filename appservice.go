package main

import (
	"context"

	"github.com/wailsapp/wails/v3/pkg/application"

	"shindentoanilist/internal/appsvc"
)

type AppService struct {
	service *appsvc.Service
}

func NewAppService() *AppService {
	return &AppService{service: appsvc.New()}
}

func (s *AppService) ServiceStartup(ctx context.Context, _ application.ServiceOptions) error {
	return s.service.Startup(ctx)
}

func (s *AppService) ServiceShutdown() error {
	return s.service.Shutdown()
}

func (s *AppService) AppName() string {
	return s.service.AppName()
}

func (s *AppService) EnsureDatabase() (appsvc.DatabaseInfo, error) {
	return s.service.EnsureDatabase()
}

func (s *AppService) LoadShindenList(userID int) (appsvc.ShindenListIndex, error) {
	return s.service.LoadShindenList(userID)
}

func (s *AppService) GetAnimeDatabase() (appsvc.AnimeDatabase, error) {
	return s.service.GetAnimeDatabase()
}

func (s *AppService) GetLoadedShindenEntries(entryIDs []uint64) ([]appsvc.ShindenEntry, error) {
	return s.service.GetLoadedShindenEntries(entryIDs)
}

func (s *AppService) GetAnimeDatabaseEntries(entryIDs []uint64) ([]appsvc.DatabaseEntry, error) {
	return s.service.GetAnimeDatabaseEntries(entryIDs)
}

func (s *AppService) MatchLoadedShindenList(options appsvc.MatchOptions) (appsvc.MatchListResult, error) {
	return s.service.MatchLoadedShindenList(options)
}

func (s *AppService) SearchAnime(query string, options appsvc.SearchOptions) (appsvc.SearchResult, error) {
	return s.service.SearchAnime(query, options)
}

func (s *AppService) MatchQuery(query string, options appsvc.MatchQueryOptions) (appsvc.MatchResult, error) {
	return s.service.MatchQuery(query, options)
}

func (s *AppService) ExportMatches(matches []appsvc.MatchSelection) (appsvc.ExportResult, error) {
	return s.service.ExportMatches(matches)
}
