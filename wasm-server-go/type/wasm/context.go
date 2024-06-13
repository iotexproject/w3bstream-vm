package wasm

import (
	"context"
	"log"
	"log/slog"
	"sync"
)

type (
	CtxLogger          struct{}
	CtxRuntimeResource struct{}
)

type WithContext = func(ctx context.Context) context.Context

func WithContextCompose(withs ...WithContext) WithContext {
	return func(ctx context.Context) context.Context {
		for i := range withs {
			ctx = withs[i](ctx)
		}
		return ctx
	}
}

func WithLogger(ctx context.Context, v *slog.Logger) context.Context {
	if ctx == nil {
		panic("with logger context, nil context")
	}
	return context.WithValue(ctx, CtxLogger{}, v)
}

func WithLoggerContext(v *slog.Logger) WithContext {
	return func(ctx context.Context) context.Context {
		return context.WithValue(ctx, CtxLogger{}, v)
	}
}

func LoggerFromContext(ctx context.Context) (*slog.Logger, bool) {
	v, ok := ctx.Value(CtxLogger{}).(*slog.Logger)
	return v, ok
}

func MustLoggerFromContext(ctx context.Context) *slog.Logger {
	v, ok := LoggerFromContext(ctx)
	if !ok {
		log.Panic("logger not ok")
	}
	return v
}

func WithRuntimeResource(ctx context.Context, v *sync.Map) context.Context {
	if ctx == nil {
		panic("with runtime resource context, nil context")
	}
	return context.WithValue(ctx, CtxRuntimeResource{}, v)
}

func WithRuntimeResourceContext(v *sync.Map) WithContext {
	return func(ctx context.Context) context.Context {
		return context.WithValue(ctx, CtxRuntimeResource{}, v)
	}
}

func RuntimeResourceFromContext(ctx context.Context) (*sync.Map, bool) {
	v, ok := ctx.Value(CtxRuntimeResource{}).(*sync.Map)
	return v, ok
}

func MustRuntimeResourceFromContext(ctx context.Context) *sync.Map {
	v, ok := RuntimeResourceFromContext(ctx)
	if !ok {
		log.Panic("runtime resource not ok")
	}
	return v
}
