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

func (s *AppService) LoadShindenList(userID int) (appsvc.ShindenList, error) {
	return s.service.LoadShindenList(userID)
}
