package main

import (
	"bytes"
	"compress/zlib"
	"context"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"log/slog"
	"net"
	"os"
	"sync"

	"github.com/google/uuid"
	"github.com/pkg/errors"
	"google.golang.org/grpc"

	pb "github.com/iotexproject/sprout-vm/wasm-server-go/proto"
	"github.com/iotexproject/sprout-vm/wasm-server-go/type/wasm"
	"github.com/iotexproject/sprout-vm/wasm-server-go/vm"
)

type server struct {
	pb.UnimplementedVmRuntimeServer
	data sync.Map
}

func (s *server) Create(ctx context.Context, req *pb.CreateRequest) (*pb.CreateResponse, error) {
	slog.Info("wasm instance create request: %v", req)
	projectId := req.ProjectID
	content := req.Content
	compressedData, err := hex.DecodeString(content)
	if err != nil {
		slog.Error("Failed to decode hex string: %v", err)
		return nil, err
	}

	reader, err := zlib.NewReader(bytes.NewReader(compressedData))
	if err != nil {
		slog.Error("Failed to create zlib reader: %v", err)
		return nil, err
	}
	defer reader.Close()

	var decompressedContent bytes.Buffer
	_, err = io.Copy(&decompressedContent, reader)
	if err != nil {
		slog.Error("Failed to read decompressed content: %v", err)
		return nil, err
	}

	contentBytes := decompressedContent.Bytes()

	id := uuid.NewString()
	ins, err := vm.NewInstance(ctx, contentBytes, id)

	//if _, exists := s.data.Load(projectId); exists {
	//	return nil, errors.New(fmt.Sprintf("instance for project ID %d already exists", req.ProjectID))
	//}
	s.data.Store(projectId, ins)

	return &pb.CreateResponse{}, nil
}

func (s *server) Execute(ctx context.Context, req *pb.ExecuteRequest) (*pb.ExecuteResponse, error) {
	slog.Info("wasm instance execute request: %v", req)

	data := map[string]interface{}{
		"projectID":          req.ProjectID,
		"taskID":             req.TaskID,
		"clientID":           req.ClientID,
		"sequencerSignature": req.SequencerSignature,
		"datas":              req.Datas,
	}
	jsonData, err := json.Marshal(data)
	if err != nil {
		slog.Error("error converting map to JSON: %v", err)
		return nil, errors.New(fmt.Sprintf("error converting map to JSON: %v", err))
	}

	ins, ok := s.data.Load(req.ProjectID)
	if !ok {
		slog.Error("instance for project ID %d not found", req.ProjectID)
		return nil, errors.New(fmt.Sprintf("instance for project ID %d not found", req.ProjectID))
	}
	result := ins.(wasm.Instance).HandleEvent(ctx, "start", jsonData)
	if result.Code < 0 {
		return nil, errors.New(result.ErrMsg)
	}
	if result.Code == 0 {
		result.Rsp = []byte("")
	}

	fmt.Println(string(result.Rsp))
	return &pb.ExecuteResponse{Result: result.Rsp}, nil
}

func main() {
	h := slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{Level: slog.LevelInfo})
	slog.SetDefault(slog.New(h))

	lis, err := net.Listen("tcp", ":4001")
	if err != nil {
		log.Fatalf("failed to listen: %v", err)
	}

	grpcServer := grpc.NewServer()
	pb.RegisterVmRuntimeServer(grpcServer, &server{})

	slog.Info("gRPC server is running on port 4001...")
	if err := grpcServer.Serve(lis); err != nil {
		log.Fatalf("failed to serve: %v", err)
	}
}
