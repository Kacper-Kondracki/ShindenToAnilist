package appsvc

import "testing"

func TestLoadShindenListRejectsInvalidUserID(t *testing.T) {
	service := New()

	for _, userID := range []int{0, -1} {
		if _, err := service.LoadShindenList(userID); err == nil {
			t.Fatalf("LoadShindenList(%d) returned nil error", userID)
		}
	}
}
