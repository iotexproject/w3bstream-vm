.PHONY: proto
proto:
	mkdir -p proto
	protoc -I=.. \
		--go_out=. \
		--go_opt=paths=source_relative \
		--go-grpc_out=. \
		--go-grpc_opt=paths=source_relative \
		../proto/vm_runtime.proto