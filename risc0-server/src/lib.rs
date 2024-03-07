use grpc::server::Risc0Server;
use rust_grpc::grpc::vm_runtime::vm_runtime_server::VmRuntimeServer;

use tonic::transport::Server;

mod db;
mod core;
mod grpc;
mod handlers;
mod model;
mod tools;

pub async fn start_grpc_server(addr: &str) {
    let addr = addr.parse().unwrap();
    let risc0_server = Risc0Server{};
 
    Server::builder()
            .add_service(VmRuntimeServer::new(risc0_server))
            .serve(addr)
            .await.unwrap();
}