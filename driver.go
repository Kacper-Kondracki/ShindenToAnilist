package main

/*
#cgo CFLAGS: -I${SRCDIR}/crates/shinden-to-anilist-driver/include
#cgo linux,production LDFLAGS: ${SRCDIR}/target/release/libshinden_to_anilist_driver.a -ldl -lm -lpthread
#cgo linux,!production LDFLAGS: -L${SRCDIR}/target/debug -lshinden_to_anilist_driver -ldl -lm -lpthread -Wl,-rpath,${SRCDIR}/target/debug
#cgo windows,amd64,production LDFLAGS: ${SRCDIR}/target/x86_64-pc-windows-gnu/release/libshinden_to_anilist_driver.a -lws2_32 -ladvapi32 -luserenv -lbcrypt -lntdll
#cgo windows,amd64,!production LDFLAGS: ${SRCDIR}/target/x86_64-pc-windows-gnu/debug/libshinden_to_anilist_driver.a -lws2_32 -ladvapi32 -luserenv -lbcrypt -lntdll
#cgo windows,arm64,production LDFLAGS: ${SRCDIR}/target/aarch64-pc-windows-gnu/release/libshinden_to_anilist_driver.a -lws2_32 -ladvapi32 -luserenv -lbcrypt -lntdll
#cgo windows,arm64,!production LDFLAGS: ${SRCDIR}/target/aarch64-pc-windows-gnu/debug/libshinden_to_anilist_driver.a -lws2_32 -ladvapi32 -luserenv -lbcrypt -lntdll
#include "shinden_to_anilist_driver.h"
*/
import "C"

import (
	"errors"
	"fmt"
	"runtime"
	"sync"
)

type Driver struct {
	mu     sync.RWMutex
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

func (d *Driver) finalize() {
	_ = d.Close()
}
