package api

import (
	"context"
	"fmt"

	"github.com/iotexproject/w3bstream-vm/gnark-server/proto"
	"github.com/iotexproject/w3bstream-vm/gnark-server/prover"
)

type Server struct {
	proto.UnimplementedVMServer
	manager *prover.ProverManager
}

func NewServer() *Server {
	return &Server{
		manager: &prover.ProverManager{},
	}
}

func (s *Server) NewProject(ctx context.Context, req *proto.NewProjectRequest) (*proto.NewProjectResponse, error) {
	// Extract proving key from metadata
	provingKey := req.Metadata

	if err := s.manager.NewProject(req.ProjectID, req.Binary, provingKey); err != nil {
		return nil, fmt.Errorf("failed to create new project: %w", err)
	}

	return &proto.NewProjectResponse{}, nil
}

func (s *Server) ExecuteTask(ctx context.Context, req *proto.ExecuteTaskRequest) (*proto.ExecuteTaskResponse, error) {
	if len(req.Payloads) != 1 {
		return nil, fmt.Errorf("expected exactly one payload")
	}

	proof, err := s.manager.Exec(req.ProjectID, req.Payloads[0])
	if err != nil {
		return nil, fmt.Errorf("failed to execute task: %w", err)
	}

	return &proto.ExecuteTaskResponse{
		Result: proof,
	}, nil
}
