use grpc::{server::Halo2GrpcServer, vm_runtime::vm_runtime_server::VmRuntimeServer};
use tonic::transport::Server;

mod grpc;

mod vm_runtime_proto {
    include!("grpc/vm_runtime.rs");
 
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
       tonic::include_file_descriptor_set!("runtime_descriptor");
}
 
 #[tokio::main]
 async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:4001".parse()?;
    let halo2_grpc_server = Halo2GrpcServer::default();
 
    let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(vm_runtime_proto::FILE_DESCRIPTOR_SET)
            .build()
            .unwrap();
 
    Server::builder()
            .add_service(VmRuntimeServer::new(halo2_grpc_server))
            .add_service(reflection_service)
            .serve(addr)
            .await?;
    Ok(())
 }
