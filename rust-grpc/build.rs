use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "../proto/vm_runtime.proto";
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .file_descriptor_set_path(out_dir.join("runtime_descriptor.bin"))
        .out_dir("./src/grpc")
        .compile_protos(&[proto_file], &["../proto"])?;

    Ok(())
}
