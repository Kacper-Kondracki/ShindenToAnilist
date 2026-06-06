package stadriver

/*
#cgo linux,production LDFLAGS: ${SRCDIR}/../../target/release/libshinden_to_anilist_driver.a -ldl -lm -lpthread
#cgo linux,!production LDFLAGS: -L${SRCDIR}/../../target/debug -lshinden_to_anilist_driver -ldl -lm -lpthread -Wl,-rpath,${SRCDIR}/../../target/debug
#cgo windows,amd64,production LDFLAGS: ${SRCDIR}/../../target/x86_64-pc-windows-gnu/release/libshinden_to_anilist_driver.a -lws2_32 -ladvapi32 -luserenv -lbcrypt -lntdll
#cgo windows,amd64,!production LDFLAGS: ${SRCDIR}/../../target/x86_64-pc-windows-gnu/debug/libshinden_to_anilist_driver.a -lws2_32 -ladvapi32 -luserenv -lbcrypt -lntdll
#cgo windows,arm64,production LDFLAGS: ${SRCDIR}/../../target/aarch64-pc-windows-gnu/release/libshinden_to_anilist_driver.a -lws2_32 -ladvapi32 -luserenv -lbcrypt -lntdll
#cgo windows,arm64,!production LDFLAGS: ${SRCDIR}/../../target/aarch64-pc-windows-gnu/debug/libshinden_to_anilist_driver.a -lws2_32 -ladvapi32 -luserenv -lbcrypt -lntdll
#include "../../crates/shinden-to-anilist-driver/include/shinden_to_anilist_driver.h"
#include <stdlib.h>
*/
import "C"

import (
	"errors"
	"fmt"
	"runtime"
	"sync"
	"unsafe"

	"shindentoanilist/internal/anime"
)

type Driver struct {
	mu     sync.RWMutex
	loadMu sync.Mutex
	ptr    *C.StaDriver
	closed bool
}

func New() (*Driver, error) {
	ptr := C.sta_driver_new()
	if ptr == nil {
		return nil, errors.New("create shinden-to-anilist driver")
	}

	driver := &Driver{ptr: ptr}
	runtime.SetFinalizer(driver, (*Driver).finalize)

	return driver, nil
}

func (d *Driver) Close() error {
	if d == nil {
		return nil
	}

	d.Abort()

	d.mu.Lock()
	defer d.mu.Unlock()

	if d.closed {
		return nil
	}

	C.sta_driver_free(d.ptr)
	d.ptr = nil
	d.closed = true
	runtime.SetFinalizer(d, nil)

	return nil
}

func (d *Driver) Abort() {
	if d == nil {
		return
	}

	d.mu.RLock()
	defer d.mu.RUnlock()

	if d.closed || d.ptr == nil {
		return
	}

	C.sta_driver_abort(d.ptr)
}

func (d *Driver) EnsureDatabase(path string) (anime.DatabaseInfo, error) {
	if d == nil {
		return anime.DatabaseInfo{}, errors.New("driver is nil")
	}

	d.loadMu.Lock()
	defer d.loadMu.Unlock()

	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))

	var out C.StaDatabaseInfo
	if err := d.call(func(ptr *C.StaDriver) C.StaError {
		return C.sta_driver_ensure_database(ptr, cPath, &out)
	}); err != nil {
		return anime.DatabaseInfo{}, err
	}
	defer C.sta_database_info_free(out)

	return anime.DatabaseInfo{
		LastUpdate: cString(out.last_update),
		Release:    cString(out.release),
		Sha256:     cString(out.sha256),
		Path:       cString(out.path),
		Updated:    bool(out.updated),
	}, nil
}

func (d *Driver) GetAnimeDatabase() (anime.AnimeDatabase, error) {
	if d == nil {
		return anime.AnimeDatabase{}, errors.New("driver is nil")
	}

	d.loadMu.Lock()
	defer d.loadMu.Unlock()

	var out C.StaAnimeDatabase
	if err := d.call(func(ptr *C.StaDriver) C.StaError {
		return C.sta_driver_get_anime_database(ptr, &out)
	}); err != nil {
		return anime.AnimeDatabase{}, err
	}
	defer C.sta_anime_database_free(out)

	entries := unsafe.Slice(out.entries, int(out.len))
	result := anime.AnimeDatabase{
		LastUpdate: optionalDate(out.last_update),
		Entries:    make([]anime.DatabaseEntry, 0, len(entries)),
	}

	for _, entry := range entries {
		result.Entries = append(result.Entries, anime.DatabaseEntry{
			ID:                   uint64(entry.id),
			ConsolidatedMetadata: consolidatedMetadata(entry.consolidated_metadata),
			Sources:              stringViewArray(entry.sources),
			Title:                stringView(entry.title),
			NormalizedTitle:      stringView(entry.normalized_title),
			Metadata:             titleMetadata(entry.metadata),
			AnimeType:            stringView(entry.anime_type),
			Episodes:             int(entry.episodes),
			Status:               stringView(entry.status),
			Season:               stringView(entry.season),
			Year:                 optionalInt(entry.year),
			Picture:              stringView(entry.picture),
			Thumbnail:            stringView(entry.thumbnail),
			Duration:             optionalInt(entry.duration),
			Synonyms:             stringViewArray(entry.synonyms),
			NormalizedSynonyms:   stringViewArray(entry.normalized_synonyms),
			Studios:              stringViewArray(entry.studios),
			Producers:            stringViewArray(entry.producers),
			RelatedAnime:         stringViewArray(entry.related_anime),
			Tags:                 stringViewArray(entry.tags),
		})
	}

	return result, nil
}

func (d *Driver) LoadShindenList(userID uint64) (anime.ShindenList, error) {
	if d == nil {
		return anime.ShindenList{}, errors.New("driver is nil")
	}

	d.loadMu.Lock()
	defer d.loadMu.Unlock()

	d.mu.RLock()
	defer d.mu.RUnlock()

	if d.closed || d.ptr == nil {
		return anime.ShindenList{}, errors.New("driver is closed")
	}

	var out C.StaShindenList
	if err := intoGoError(C.sta_driver_load_shinden_list(d.ptr, C.uint64_t(userID), &out)); err != nil {
		return anime.ShindenList{}, err
	}
	defer C.sta_shinden_list_free(out)

	entries := unsafe.Slice(out.entries, int(out.len))
	result := anime.ShindenList{
		Entries: make([]anime.ShindenEntry, 0, len(entries)),
	}

	for _, entry := range entries {
		result.Entries = append(result.Entries, anime.ShindenEntry{
			ID:              uint64(entry.id),
			CoverID:         optionalInt(entry.cover_id),
			Title:           stringView(entry.title),
			AnimeStatus:     stringView(entry.anime_status),
			AnimeType:       stringView(entry.anime_type),
			PremiereDate:    optionalDate(entry.premiere_date),
			FinishDate:      optionalDate(entry.finish_date),
			Episodes:        optionalInt(entry.episodes),
			IsFavourite:     bool(entry.is_favourite),
			WatchStatus:     stringView(entry.watch_status),
			WatchedEpisodes: int(entry.watched_episodes),
			Score:           optionalInt(entry.score),
			Note:            optionalString(entry.note),
			Description:     optionalString(entry.description),
		})
	}

	return result, nil
}

func (d *Driver) SearchAnime(query string, options anime.SearchOptions) (anime.SearchResult, error) {
	if d == nil {
		return anime.SearchResult{}, errors.New("driver is nil")
	}

	d.loadMu.Lock()
	defer d.loadMu.Unlock()

	cQuery := C.CString(query)
	defer C.free(unsafe.Pointer(cQuery))
	cMode := C.CString(options.Mode)
	defer C.free(unsafe.Pointer(cMode))

	var out C.StaSearchResult
	cOptions := C.StaSearchOptions{
		mode:      C.StaStringView{ptr: cMode, len: C.uintptr_t(len(options.Mode))},
		limit:     C.uintptr_t(options.Limit),
		threshold: C.float(options.Threshold),
	}
	if err := d.call(func(ptr *C.StaDriver) C.StaError {
		return C.sta_driver_search_anime(ptr, cQuery, cOptions, &out)
	}); err != nil {
		return anime.SearchResult{}, err
	}
	defer C.sta_search_result_free(out)

	items := unsafe.Slice(out.items, int(out.len))
	result := anime.SearchResult{
		Items: make([]anime.SearchItem, 0, len(items)),
	}
	for _, item := range items {
		result.Items = append(result.Items, anime.SearchItem{
			ID:    uint64(item.id),
			Score: float32(item.score),
		})
	}

	return result, nil
}

func (d *Driver) MatchQuery(query string, options anime.MatchQueryOptions) (anime.MatchResult, error) {
	if d == nil {
		return anime.MatchResult{}, errors.New("driver is nil")
	}

	d.loadMu.Lock()
	defer d.loadMu.Unlock()

	cQuery := C.CString(query)
	defer C.free(unsafe.Pointer(cQuery))
	cMode := C.CString(options.Search.Mode)
	defer C.free(unsafe.Pointer(cMode))

	cOptions := C.StaMatchQueryOptions{
		search: C.StaSearchOptions{
			mode:      C.StaStringView{ptr: cMode, len: C.uintptr_t(len(options.Search.Mode))},
			limit:     C.uintptr_t(options.Search.Limit),
			threshold: C.float(options.Search.Threshold),
		},
	}
	if options.ResultLimit != nil {
		cOptions.result_limit = C.uintptr_t(*options.ResultLimit)
		cOptions.has_result_limit = true
	}

	var out C.StaMatchResult
	if err := d.call(func(ptr *C.StaDriver) C.StaError {
		return C.sta_driver_match_query(ptr, cQuery, cOptions, &out)
	}); err != nil {
		return anime.MatchResult{}, err
	}
	defer C.sta_match_result_free(out)

	return matchResult(out), nil
}

func (d *Driver) MatchLoadedShindenList(options anime.MatchOptions) (anime.MatchListResult, error) {
	if d == nil {
		return anime.MatchListResult{}, errors.New("driver is nil")
	}

	d.loadMu.Lock()
	defer d.loadMu.Unlock()

	cOptions := C.StaMatchOptions{
		candidate_limit:  C.uintptr_t(options.CandidateLimit),
		search_threshold: C.float(options.SearchThreshold),
	}
	if options.ResultLimit != nil {
		cOptions.result_limit = C.uintptr_t(*options.ResultLimit)
		cOptions.has_result_limit = true
	}

	var out C.StaMatchListResult
	if err := d.call(func(ptr *C.StaDriver) C.StaError {
		return C.sta_driver_match_loaded_shinden_list(ptr, cOptions, &out)
	}); err != nil {
		return anime.MatchListResult{}, err
	}
	defer C.sta_match_list_result_free(out)

	entries := unsafe.Slice(out.entries, int(out.len))
	result := anime.MatchListResult{
		Entries:   make([]anime.ShindenMatchResult, 0, len(entries)),
		Total:     int(out.total),
		Winners:   int(out.winners),
		HasTop:    int(out.has_top),
		Unmatched: int(out.unmatched),
	}
	for _, entry := range entries {
		result.Entries = append(result.Entries, anime.ShindenMatchResult{
			ShindenID: uint64(entry.shinden_id),
			Result:    matchResult(entry.result),
		})
	}

	return result, nil
}

func (d *Driver) ExportMatches(path string, selections []anime.MatchSelection) (anime.ExportResult, error) {
	if d == nil {
		return anime.ExportResult{}, errors.New("driver is nil")
	}

	d.loadMu.Lock()
	defer d.loadMu.Unlock()

	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))

	cSelections := make([]C.StaMatchSelection, len(selections))
	for i, selection := range selections {
		cSelections[i] = C.StaMatchSelection{
			shinden_id:  C.uint64_t(selection.ShindenID),
			database_id: C.uint64_t(selection.DatabaseID),
		}
	}

	var selectionPtr *C.StaMatchSelection
	if len(cSelections) > 0 {
		selectionPtr = &cSelections[0]
	}

	var out C.StaExportResult
	if err := d.call(func(ptr *C.StaDriver) C.StaError {
		return C.sta_driver_export_matches(ptr, cPath, selectionPtr, C.uintptr_t(len(cSelections)), &out)
	}); err != nil {
		return anime.ExportResult{}, err
	}
	defer C.sta_export_result_free(out)

	return anime.ExportResult{
		Path:          cString(out.path),
		ExportedCount: int(out.exported_count),
		Cancelled:     false,
	}, nil
}

func (d *Driver) call(f func(*C.StaDriver) C.StaError) error {
	if d == nil {
		return errors.New("driver is nil")
	}

	d.mu.RLock()
	defer d.mu.RUnlock()

	if d.closed || d.ptr == nil {
		return errors.New("driver is closed")
	}

	return intoGoError(f(d.ptr))
}

func intoGoError(errResult C.StaError) error {
	if errResult.status == C.StaStatusOk {
		return nil
	}

	if errResult.message == nil {
		return fmt.Errorf("driver call failed with status %d", int(errResult.status))
	}

	message := C.GoString(errResult.message)
	C.sta_string_free(errResult.message)

	if message == "" {
		return fmt.Errorf("driver call failed with status %d", int(errResult.status))
	}

	return errors.New(message)
}

func cString(value *C.char) string {
	if value == nil {
		return ""
	}

	return C.GoString(value)
}

func stringView(value C.StaStringView) string {
	if value.ptr == nil || value.len == 0 {
		return ""
	}

	return C.GoStringN(value.ptr, C.int(value.len))
}

func optionalString(value C.StaStringView) *string {
	if value.ptr == nil {
		return nil
	}

	result := stringView(value)
	return &result
}

func optionalInt(value C.StaOptionalI32) *int {
	if !bool(value.has_value) {
		return nil
	}

	result := int(value.value)
	return &result
}

func optionalFloat(value C.StaOptionalF32) *float32 {
	if !bool(value.has_value) {
		return nil
	}

	result := float32(value.value)
	return &result
}

func optionalDate(value C.StaOptionalDate) *string {
	if !bool(value.has_value) {
		return nil
	}

	result := fmt.Sprintf("%04d-%02d-%02d", int(value.year), uint32(value.month), uint32(value.day))
	return &result
}

func stringViewArray(value C.StaStringViewArray) []string {
	if value.entries == nil || value.len == 0 {
		return nil
	}

	entries := unsafe.Slice(value.entries, int(value.len))
	result := make([]string, 0, len(entries))
	for _, entry := range entries {
		result = append(result, stringView(entry))
	}

	return result
}

func titleMetadata(value C.StaTitleMetadata) anime.TitleMetadata {
	return anime.TitleMetadata{
		Season:            optionalFloat(value.season),
		Part:              optionalFloat(value.part),
		Episode:           optionalFloat(value.episode),
		HasSeasonKeyword:  bool(value.has_season_keyword),
		HasPartKeyword:    bool(value.has_part_keyword),
		HasEpisodeKeyword: bool(value.has_episode_keyword),
	}
}

func consolidatedMetadata(value C.StaConsolidatedMetadata) anime.ConsolidatedMetadata {
	return anime.ConsolidatedMetadata{
		Season:         optionalFloat(value.season),
		Part:           optionalFloat(value.part),
		Episode:        optionalFloat(value.episode),
		IsFinalSeason:  bool(value.is_final_season),
		IsFinalPart:    bool(value.is_final_part),
		IsFinalEpisode: bool(value.is_final_episode),
	}
}

func scoreBreakdown(value C.StaScoreBreakdown) anime.ScoreBreakdown {
	return anime.ScoreBreakdown{
		SearchScore:   float32(value.search_score),
		SeasonScore:   float32(value.season_score),
		YearScore:     float32(value.year_score),
		TypeScore:     float32(value.type_score),
		StatusScore:   float32(value.status_score),
		SeasonalScore: float32(value.seasonal_score),
		EpisodesScore: float32(value.episodes_score),
		FinalScore:    float32(value.final_score),
	}
}

func scoredCandidate(value C.StaScoredCandidate) anime.ScoredCandidate {
	return anime.ScoredCandidate{
		ID:    uint64(value.id),
		Score: scoreBreakdown(value.score),
	}
}

func scoredCandidates(entries *C.StaScoredCandidate, n C.uintptr_t) []anime.ScoredCandidate {
	if entries == nil || n == 0 {
		return nil
	}

	values := unsafe.Slice(entries, int(n))
	result := make([]anime.ScoredCandidate, 0, len(values))
	for _, value := range values {
		result = append(result, scoredCandidate(value))
	}

	return result
}

func matchResult(value C.StaMatchResult) anime.MatchResult {
	var winner *anime.ScoredCandidate
	if bool(value.winner.has_value) {
		result := scoredCandidate(value.winner.item)
		winner = &result
	}

	return anime.MatchResult{
		Items:  scoredCandidates(value.items, value.items_len),
		Top:    scoredCandidates(value.top, value.top_len),
		Winner: winner,
	}
}

func (d *Driver) finalize() {
	_ = d.Close()
}
