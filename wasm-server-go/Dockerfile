FROM golang:1.21 AS builder

ENV GO111MODULE=on

WORKDIR /go/src
COPY ./ ./

RUN cd ./wasm-server-go/cmd && go build -o wasm-server-go

FROM golang:1.21 AS runtime

COPY --from=builder /go/src/wasm-server-go/cmd/wasm-server-go /go/bin/wasm-server-go
EXPOSE 4001

WORKDIR /go/bin
ENTRYPOINT ["/go/bin/wasm-server-go"]
