use grpc::server::WasmtimeGrpcServer;
use rust_grpc::grpc::vm_runtime::vm_runtime_server::VmRuntimeServer;
use tonic::transport::Server;

mod grpc;
#[cfg(test)]
mod tests;
mod wasmtime;

pub async fn start_grpc_server(addr: &str) {
    let addr = addr.parse().unwrap();
    let wasm_grpc_server = WasmtimeGrpcServer::default();

    Server::builder()
        .add_service(VmRuntimeServer::new(wasm_grpc_server))
        .serve(addr)
        .await.unwrap();
}
