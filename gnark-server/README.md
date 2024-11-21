# Gnark Server

A gRPC server implementation for zero-knowledge proof generation using the [gnark](https://github.com/ConsenSys/gnark) framework. This server enables efficient management and execution of zero-knowledge proofs in a distributed environment.

## Overview

This server provides a gRPC interface for:

1. Loading zero-knowledge circuits and proving keys
2. Executing proving tasks with witness data
3. Managing multiple proving tasks through a project system

## Quick Start
1. Start the server:
```
go run cmd/main.go --port 4004
```

2. The server will listen for gRPC requests on the specified port.
## API Reference
NewProject
Creates a new project with associated circuit and proving key:

```
rpc NewProject(NewProjectRequest) returns (NewProjectResponse)
```


ExecuteTask
Generates a proof using the provided witness data:

```
rpc ExecuteTask(ExecuteTaskRequest) returns (ExecuteTaskResponse)
```

## Example Usage
```go
// Client example
client := proto.NewVMClient(conn)

// Create new project
projectResp, err := client.NewProject(context.Background(), &proto.NewProjectRequest{
    ProjectID: 1,
    Binary:    circuitBytes,
    Metadata:  provingKeyBytes,
})

// Execute proving task
execResp, err := client.ExecuteTask(context.Background(), &proto.ExecuteTaskRequest{
    ProjectID: 1,
    Payloads: [][]byte{witnessBytes},
})
```