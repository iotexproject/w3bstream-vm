FROM golang:1.21 AS builder

ENV GO111MODULE=on

WORKDIR /go/src
COPY ./ ./

RUN cd ./cmd && go build -o wasm-server-go

FROM scratch AS runtime

COPY --from=builder /go/src/cmd/wasm-server-go /go/bin/wasm-server-go
EXPOSE 4001

WORKDIR /go/bin
ENTRYPOINT ["/go/bin/wasm-server-go"]
