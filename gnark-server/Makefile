.PHONY: proto
proto:
	mkdir -p proto
	protoc -I=.. \
		--go_out=. \
		--go_opt=paths=source_relative \
		--go-grpc_out=. \
		--go-grpc_opt=paths=source_relative \
		../proto/vm_runtime.proto

.PHONY: build
build:
	go build -o bin/gnark-server ./cmd/main.go

.PHONY: docker
docker:
	docker build -t gnark-server:latest .