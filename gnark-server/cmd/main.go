package main

import (
	"flag"
	"fmt"
	"log"
	"net"

	"github.com/iotexproject/w3bstream-vm/gnark-server/api"
	"github.com/iotexproject/w3bstream-vm/gnark-server/proto"
	"google.golang.org/grpc"
)

var (
	port = flag.Int("port", 8080, "The server port")
)

func main() {
	flag.Parse()

	lis, err := net.Listen("tcp", fmt.Sprintf(":%d", *port))
	if err != nil {
		log.Fatalf("failed to listen: %v", err)
	}

	s := grpc.NewServer()
	proto.RegisterVMServer(s, api.NewServer())

	log.Printf("server listening at %v\n", lis.Addr())
	if err := s.Serve(lis); err != nil {
		log.Fatalf("failed to serve: %v", err)
	}
}
