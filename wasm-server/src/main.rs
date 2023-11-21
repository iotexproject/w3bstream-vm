use grpc::server::WasmtimeGrpcServer;
use rust_grpc::grpc::vm_runtime::vm_runtime_server::VmRuntimeServer;
use tonic::transport::Server;

mod wasmtime;
mod grpc;
 
 #[tokio::main]
 async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:4001".parse()?;
    let wasm_grpc_server = WasmtimeGrpcServer::default();
 
    Server::builder()
            .add_service(VmRuntimeServer::new(wasm_grpc_server))
            .serve(addr)
            .await?;
    Ok(())
 }
