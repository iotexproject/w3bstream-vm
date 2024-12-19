package main

import (
	"flag"
	"fmt"
	"log"
	"math"
	"net"

	"github.com/iotexproject/w3bstream-vm/gnark-server/api"
	"github.com/iotexproject/w3bstream-vm/gnark-server/proto"
	"google.golang.org/grpc"
)

var (
	port = flag.Int("port", 4005, "The server port")
)

func main() {
	flag.Parse()

	lis, err := net.Listen("tcp", fmt.Sprintf(":%d", *port))
	if err != nil {
		log.Fatalf("failed to listen: %v", err)
	}

	s := grpc.NewServer(grpc.MaxRecvMsgSize(math.MaxInt32))
	proto.RegisterVMServer(s, api.NewServer())

	log.Printf("server listening at %v\n", lis.Addr())
	if err := s.Serve(lis); err != nil {
		log.Fatalf("failed to serve: %v", err)
	}
}
