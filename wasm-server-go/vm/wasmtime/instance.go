package wasmtime

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"sync"

	"github.com/bytecodealliance/wasmtime-go/v8"
	"github.com/google/uuid"

	"github.com/machinefi/sprout-vm/wasm-server-go/type/wasm"
)

const (
	maxUint = ^uint32(0)
	maxInt  = int(maxUint >> 1)
)

type Instance struct {
	ctx      context.Context
	id       string
	rt       *Runtime
	res      *sync.Map
	handlers map[string]*wasmtime.Func
}

func NewInstanceByCode(ctx context.Context, id string, code []byte) (*Instance, error) {

	res := &sync.Map{}
	rt := NewRuntime()
	lk, err := NewExportFuncs(wasm.WithContextCompose(
		wasm.WithRuntimeResourceContext(res),
		wasm.WithLoggerContext(slog.New(slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{Level: slog.LevelInfo}))),
	)(ctx), rt)
	if err != nil {
		return nil, err
	}
	if err := rt.Link(lk, code); err != nil {
		return nil, err
	}

	ins := &Instance{
		rt:       rt,
		res:      res,
		id:       id,
		handlers: make(map[string]*wasmtime.Func),
	}

	return ins, nil
}

var _ wasm.Instance = (*Instance)(nil)

func (i *Instance) ID() string { return i.id }

func (i *Instance) Start(ctx context.Context) error {
	return nil
}

func (i *Instance) Stop(ctx context.Context) error {
	return nil
}

func (i *Instance) HandleEvent(ctx context.Context, handler string, data []byte) *wasm.EventHandleResult {
	res := i.handle(ctx, handler, data)
	return res
}

func (i *Instance) handle(ctx context.Context, fn string, data []byte) *wasm.EventHandleResult {
	rid := i.AddResource(data)
	defer i.RmvResource(rid)

	if err := i.rt.Instantiate(ctx); err != nil {
		return &wasm.EventHandleResult{
			InstanceID: i.id,
			ErrMsg:     err.Error(),
			Code:       int32(-1),
		}
	}
	defer i.rt.Deinstantiate(ctx)

	result, err := i.rt.Call(ctx, fn, int32(rid))
	if err != nil {
		return &wasm.EventHandleResult{
			InstanceID: i.id,
			ErrMsg:     err.Error(),
			Code:       int32(-1),
		}
	}

	res := &wasm.EventHandleResult{
		InstanceID: i.id,
		Code:       result.(int32),
	}

	if result.(int32) > 0 {
		da, ok := i.GetResource(uint32(result.(int32)))
		if !ok {
			res.Code = -1
			res.ErrMsg = fmt.Sprintf("can not find resource id %d", res.Code)
			return res
		}
		res.Rsp = da
	}

	return res
}

func (i *Instance) AddResource(data []byte) uint32 {
	var id = int32(uuid.New().ID() % uint32(maxInt))
	i.res.Store(uint32(id), data)
	return uint32(id)
}

func (i *Instance) GetResource(id uint32) ([]byte, bool) {
	if value, ok := i.res.Load(id); ok {
		return value.([]byte), ok
	} else {
		return nil, ok
	}
}

func (i *Instance) RmvResource(id uint32) {
	i.res.Delete(id)
}
