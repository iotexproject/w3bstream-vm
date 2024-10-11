use grpc::server::Risc0Server;
use rust_grpc::grpc::vm::vm_server::VmServer;

use tonic::transport::Server;

mod core;
mod grpc;

pub async fn start_grpc_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse()?;
    let risc0_server = Risc0Server::new();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tracing::info!(message = "Starting server.", %addr);

    Server::builder()
        .trace_fn(|_| tracing::info_span!("risc0_server"))
        .add_service(VmServer::new(risc0_server))
        .serve(addr)
        .await?;

    Ok(())
}
