use risc0_server::start_grpc_server;

#[tokio::main]
async fn main() {
    // start grpc server
    println!("{}", "start grpc server...");
    start_grpc_server("0.0.0.0:4001").await;
}
