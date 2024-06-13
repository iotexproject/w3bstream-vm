package wasmtime

import (
	"context"
	"log/slog"
	"sync"

	"github.com/machinefi/sprout-vm/wasm-server-go/type/wasm"
)

type (
	Import func(module, name string, f interface{}) error

	ABILinker interface {
		LinkABI(Import) error
	}

	ExportFuncs struct {
		rt  *Runtime
		res *sync.Map
		log *slog.Logger
		ctx context.Context
	}
)

func NewExportFuncs(ctx context.Context, rt *Runtime) (*ExportFuncs, error) {
	ef := &ExportFuncs{
		res: wasm.MustRuntimeResourceFromContext(ctx),
		log: wasm.MustLoggerFromContext(ctx),
		rt:  rt,
		ctx: ctx,
	}

	return ef, nil
}

var (
	_ wasm.ABI = (*ExportFuncs)(nil)
)

func (ef *ExportFuncs) LinkABI(impt Import) error {
	for name, ff := range map[string]interface{}{
		"ws_log":      ef.Log,
		"ws_get_data": ef.GetData,
		"ws_set_data": ef.SetData,
	} {
		if err := impt("env", name, ff); err != nil {
			return err
		}
	}

	return nil
}

func (ef *ExportFuncs) Log(logLevel, ptr, size int32) int32 {
	buf, err := ef.rt.Read(ptr, size)
	if err != nil {
		ef.log.Error(err.Error())
		return int32(-1)
	}

	switch uint32(logLevel) {
	case 5:
		ef.log.Debug(string(buf))
	case 4:
		ef.log.Info(string(buf))
	case 3:
		ef.log.Warn(string(buf))
	case 2:
		ef.log.Error(string(buf))
	default:
		return int32(-1)
	}
	return int32(0)
}

func (ef *ExportFuncs) GetData(rid, vmAddrPtr, vmSizePtr int32) int32 {
	data, ok := ef.res.Load(uint32(rid))
	if !ok {
		return int32(-1)
	}

	if err := ef.rt.Copy(data.([]byte), vmAddrPtr, vmSizePtr); err != nil {
		return int32(-1)
	}

	return int32(0)
}

func (ef *ExportFuncs) SetData(rid, addr, size int32) int32 {
	buf, err := ef.rt.Read(addr, size)
	if err != nil {
		return int32(-1)
	}
	ef.res.Store(uint32(rid), buf)
	return int32(0)
}
