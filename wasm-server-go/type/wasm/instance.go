package wasm

import (
	"context"
)

type VM interface {
	Name() string
	Init()
	NewModule(code []byte) Module
}

type Module interface {
	Init()
	NewInstance() Instance
	GetABI() []string
}

type Instance interface {
	ID() string
	Start(context.Context) error
	Stop(context.Context) error

	EventConsumer
}

type EventHandleResult struct {
	InstanceID string `json:"instanceID"`
	Rsp        []byte `json:"-"`
	Code       int32  `json:"code"`
	ErrMsg     string `json:"errMsg"`
}

type EventConsumer interface {
	HandleEvent(ctx context.Context, handler string, payload []byte) *EventHandleResult
}

type ContextHandler interface {
	Name() string
	GetImports() ImportsHandler
	SetImports(ImportsHandler)
	GetExports() ExportsHandler
	GetInstance() Instance
	SetInstance(Instance)
}

type ABI interface {
	Log(loglevel, ptr, size int32) int32
	GetData(rid, vmAddrPtr, vmSizePtr int32) int32
	SetData(rid, addr, size int32) int32
}

type Memory interface {
	Read(context.Context, uint32, uint32) ([]byte, error)
	Write(context.Context, []byte)
}

type ImportsHandler interface {
	GetData()
	SetData()
	Log(level uint32)
}

type Handler interface {
	Name() string
	Call(context.Context, ...interface{})
}

type ExportsHandler interface {
	Start()
	Alloc()
	Free()
}
