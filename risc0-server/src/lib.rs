use grpc::server::Risc0Server;
use rust_grpc::grpc::vm_runtime::vm_runtime_server::VmRuntimeServer;

use tonic::transport::Server;

mod core;
mod db;
mod grpc;
mod handlers;
mod model;
#[cfg(test)]
mod tests;
mod tools;

pub async fn start_grpc_server(addr: &str) {
    let addr = addr.parse().unwrap();
    let risc0_server = Risc0Server {};

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tracing::info!(message = "Starting server.", %addr);

    Server::builder()
        .trace_fn(|_| tracing::info_span!("risc0_server"))
        .add_service(VmRuntimeServer::new(risc0_server))
        .serve(addr)
        .await
        .unwrap();
}
