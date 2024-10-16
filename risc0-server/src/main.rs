use risc0_server::start_grpc_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // start grpc server
    start_grpc_server("0.0.0.0:4001").await?;

    Ok(())
}
