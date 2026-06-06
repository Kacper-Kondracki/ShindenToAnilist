package main

/*
#cgo linux,production LDFLAGS: ${SRCDIR}/target/release/libshinden_to_anilist_driver.a -ldl -lm -lpthread
#cgo linux,!production LDFLAGS: -L${SRCDIR}/target/debug -lshinden_to_anilist_driver -ldl -lm -lpthread -Wl,-rpath,${SRCDIR}/target/debug
#cgo windows,amd64,production LDFLAGS: ${SRCDIR}/target/x86_64-pc-windows-gnu/release/libshinden_to_anilist_driver.a -lws2_32 -ladvapi32 -luserenv -lbcrypt -lntdll
#cgo windows,amd64,!production LDFLAGS: ${SRCDIR}/target/x86_64-pc-windows-gnu/debug/libshinden_to_anilist_driver.a -lws2_32 -ladvapi32 -luserenv -lbcrypt -lntdll
#cgo windows,arm64,production LDFLAGS: ${SRCDIR}/target/aarch64-pc-windows-gnu/release/libshinden_to_anilist_driver.a -lws2_32 -ladvapi32 -luserenv -lbcrypt -lntdll
#cgo windows,arm64,!production LDFLAGS: ${SRCDIR}/target/aarch64-pc-windows-gnu/debug/libshinden_to_anilist_driver.a -lws2_32 -ladvapi32 -luserenv -lbcrypt -lntdll
#include "crates/shinden-to-anilist-driver/include/shinden_to_anilist_driver.h"
#include <stdlib.h>
*/
import "C"

import (
	"errors"
	"fmt"
	"runtime"
	"sync"
	"unsafe"
)

type DatabaseInfo struct {
	LastUpdate string `json:"lastUpdate"`
	Release    string `json:"release"`
	Sha256     string `json:"sha256"`
	Path       string `json:"path"`
	Updated    bool   `json:"updated"`
}

type ShindenList struct {
	Entries []ShindenEntry `json:"entries"`
}

type ShindenEntry struct {
	ID              uint64  `json:"id"`
	CoverID         *int    `json:"coverId"`
	Title           string  `json:"title"`
	AnimeStatus     string  `json:"animeStatus"`
	AnimeType       string  `json:"animeType"`
	PremiereDate    *string `json:"premiereDate"`
	FinishDate      *string `json:"finishDate"`
	Episodes        *int    `json:"episodes"`
	IsFavourite     bool    `json:"isFavourite"`
	WatchStatus     string  `json:"watchStatus"`
	WatchedEpisodes int     `json:"watchedEpisodes"`
	Score           *int    `json:"score"`
	Note            *string `json:"note"`
	Description     *string `json:"description"`
}

type Driver struct {
	mu     sync.RWMutex
	loadMu sync.Mutex
	ptr    *C.StaDriver
	closed bool
}

func NewDriver() (*Driver, error) {
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

func (d *Driver) CounterValue() (int64, error) {
	var out C.int64_t
	if err := d.call(func(ptr *C.StaDriver) C.StaError {
		return C.sta_driver_counter_value(ptr, &out)
	}); err != nil {
		return 0, err
	}

	return int64(out), nil
}

func (d *Driver) IncrementCounter(amount uint32) (int64, error) {
	var out C.int64_t
	if err := d.call(func(ptr *C.StaDriver) C.StaError {
		return C.sta_driver_counter_increment(ptr, C.uint32_t(amount), &out)
	}); err != nil {
		return 0, err
	}

	return int64(out), nil
}

func (d *Driver) EnsureDatabase(path string) (DatabaseInfo, error) {
	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))

	var out C.StaDatabaseInfo
	if err := d.call(func(ptr *C.StaDriver) C.StaError {
		return C.sta_driver_ensure_database(ptr, cPath, &out)
	}); err != nil {
		return DatabaseInfo{}, err
	}
	defer C.sta_database_info_free(out)

	return DatabaseInfo{
		LastUpdate: cString(out.last_update),
		Release:    cString(out.release),
		Sha256:     cString(out.sha256),
		Path:       cString(out.path),
		Updated:    bool(out.updated),
	}, nil
}

func (d *Driver) LoadShindenList(userID uint64) (ShindenList, error) {
	if d == nil {
		return ShindenList{}, errors.New("driver is nil")
	}

	d.loadMu.Lock()
	defer d.loadMu.Unlock()

	d.mu.RLock()
	defer d.mu.RUnlock()

	if d.closed || d.ptr == nil {
		return ShindenList{}, errors.New("driver is closed")
	}

	var out C.StaShindenList
	if err := intoGoError(C.sta_driver_load_shinden_list(d.ptr, C.uint64_t(userID), &out)); err != nil {
		return ShindenList{}, err
	}
	defer C.sta_shinden_list_free(out)

	entries := unsafe.Slice(out.entries, int(out.len))
	result := ShindenList{
		Entries: make([]ShindenEntry, 0, len(entries)),
	}

	for _, entry := range entries {
		result.Entries = append(result.Entries, ShindenEntry{
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

func optionalDate(value C.StaOptionalDate) *string {
	if !bool(value.has_value) {
		return nil
	}

	result := fmt.Sprintf("%04d-%02d-%02d", int(value.year), uint32(value.month), uint32(value.day))
	return &result
}

func (d *Driver) finalize() {
	_ = d.Close()
}
