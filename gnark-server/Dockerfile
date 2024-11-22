FROM golang:1.23-alpine AS builder

WORKDIR /app
COPY ./gnark-server .
RUN go mod download
RUN CGO_ENABLED=0 go build -o /gnark-server ./cmd/main.go

FROM alpine:3.17
COPY --from=builder /gnark-server /gnark-server
EXPOSE 4005
ENTRYPOINT ["/gnark-server"]
