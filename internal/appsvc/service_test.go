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

func TestGetLoadedShindenEntryIDsRejectsInvalidView(t *testing.T) {
	service := New()

	if _, err := service.GetLoadedShindenEntryIDs("unknown"); err == nil {
		t.Fatal("GetLoadedShindenEntryIDs returned nil error for invalid view")
	}
}

func TestSearchAnimeRejectsInvalidSearchMode(t *testing.T) {
	service := New()

	if _, err := service.SearchAnime("naruto", SearchOptions{Mode: "loose"}); err == nil {
		t.Fatal("SearchAnime returned nil error for invalid search mode")
	}
}
