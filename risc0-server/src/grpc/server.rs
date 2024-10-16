use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::{io::Read, str::FromStr};

use ethers::abi::{encode, Token};
use flate2::read::ZlibDecoder;
use lazy_static::lazy_static;
use regex::Regex;
use risc0_ethereum_contracts::groth16;
use risc0_zkvm::InnerReceipt;
use rust_grpc::grpc::vm::{
    vm_server::Vm, ExecuteTaskRequest, ExecuteTaskResponse, NewProjectRequest, NewProjectResponse,
};
use serde_json::Value;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use tracing::{error, info, warn};

use crate::core::prover::{BonsaiProver, LocalProver, Prover};

pub struct Risc0Server {
    // TODO: replace with LRU
    projects: Arc<RwLock<HashMap<(u64, u64), Project>>>,
}

#[derive(Clone)]
struct Project {
    pub project_id: u64,
    // TODO: share prover across threads
    pub elf: Vec<u8>,
    pub image_id: Vec<u32>,
}

lazy_static! {
    static ref ELF_RE: Regex = Regex::new(r"pub const (\w+)_ELF: \&\[u8\] = \&\[(.+?)\];").unwrap();
    static ref IMAGE_ID_RE: Regex =
        Regex::new(r"pub const (\w+)_ID: \[u32; \d+\] = \[(.+?)\];").unwrap();
}

impl Risc0Server {
    pub fn new() -> Self {
        Risc0Server {
            projects: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn extract_data<T: FromStr + Debug>(&self, regex: &Regex, text: &str) -> Option<Vec<T>>
    where
        T::Err: Debug,
    {
        if let Some(captures) = regex.captures(text) {
            // let prefix = captures[1].to_string();
            let data_str = &captures[2];
            let data: Vec<T> = data_str
                .split(',')
                .map(|num| num.trim().parse::<T>().unwrap())
                .collect();
            return Some(data);
        }
        None
    }
}

#[tonic::async_trait]
impl Vm for Risc0Server {
    async fn new_project(
        &self,
        request: Request<NewProjectRequest>,
    ) -> Result<Response<NewProjectResponse>, Status> {
        let req = request.get_ref();

        // Decompress the binary data
        let compressed_data = req.binary.as_slice();
        let mut decoder = ZlibDecoder::new(compressed_data);
        let mut content = Vec::new();
        decoder
            .read_to_end(&mut content)
            .map_err(|e| Status::internal(format!("Failed to decompress data: {}", e)))?;
        let content_str = String::from_utf8(content).unwrap();

        let elf_data = self
            .extract_data::<u8>(&ELF_RE, &content_str)
            .ok_or(Status::internal("Failed to extract ELF data"))?;
        let id_data = self
            .extract_data::<u32>(&IMAGE_ID_RE, &content_str)
            .ok_or(Status::internal("Failed to extract ID data"))?;

        {
            let mut map = self.projects.write().await;
            map.insert(
                (req.project_id, 0),
                Project {
                    project_id: req.project_id,
                    elf: elf_data,
                    image_id: id_data,
                },
            );
        }

        info!("New project added: {}", req.project_id);

        Ok(Response::new(NewProjectResponse {}))
    }

    async fn execute_task(
        &self,
        request: Request<ExecuteTaskRequest>,
    ) -> Result<Response<ExecuteTaskResponse>, Status> {
        info!("risc0_server execute_task");

        let req = request.into_inner();

        if req.payloads.is_empty() {
            return Err(Status::invalid_argument("data is empty"));
        }

        let project = {
            let map = self.projects.read().await;
            map.get(&(req.project_id, 0)).cloned()
        }
        .ok_or_else(|| Status::not_found(format!("{} not found", req.project_id)))?;

        // TODO move to guest method
        // param = {"private_input":"14", "public_input":"3,34", "receipt_type":"Snark"}
        // let input_datas = json!(datas).to_string();
        let v: Value = serde_json::from_slice(&req.payloads[0]).unwrap();

        let receipt = tokio::task::spawn_blocking(move || {
            // TODO: move prover initialization to new_project
            let prover: Box<dyn Prover> = match v.get("receipt_type") {
                Some(receipt_type) => match receipt_type.as_str().unwrap() {
                    "Stark" => Box::new(LocalProver::new(&project.elf)),
                    "Snark" => Box::new(BonsaiProver::new(&project.elf)),
                    _ => Box::new(LocalProver::new(&project.elf)),
                },
                None => Box::new(LocalProver::new(&project.elf)),
            };

            let data: Vec<String> = req
                .payloads
                .clone()
                .into_iter()
                .map(|v| String::from_utf8(v).expect("Invalid UTF-8 sequence"))
                .collect();

            prover.prove(data)
        })
        .await
        .map_err(|e| Status::internal(format!("Failed to spawn blocking task: {}", e)))?
        .map_err(|e| Status::internal(format!("Failed to prove: {}", e)))?;

        info!("receipt: {:?}", receipt);

        // let mut result = receipt.as_bytes().to_vec();
        let mut result = serde_json::to_vec(&receipt).unwrap();
        // let risc_receipt: Receipt = serde_json::from_str(&receipt).unwrap();
        if matches!(receipt.inner, InnerReceipt::Groth16(_)) {
            let seal = groth16::encode(receipt.inner.groth16().unwrap().seal.clone()).unwrap();
            let journal = receipt.journal.bytes.clone();

            let tokens = vec![Token::Bytes(seal), Token::Bytes(journal)];

            result = encode(&tokens);
        }

        Ok(Response::new(ExecuteTaskResponse { result: result }))
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use risc0_zkvm::Receipt;

//     #[test]
//     fn param() {
//         let receipt = "{\"inner\":{\"Groth16\":{\"seal\":[13,130,138,212,74,234,176,239,95,29,228,225,143,171,188,84,199,197,244,148,214,38,199,17,199,58,134,101,217,241,136,201,12,22,9,213,157,120,249,214,255,96,0,200,170,149,120,109,143,229,183,161,226,83,220,46,139,2,50,113,217,0,187,186,47,205,143,50,147,175,156,184,200,208,39,186,129,20,149,87,98,224,66,93,24,245,84,67,67,81,183,244,100,51,144,189,18,149,157,185,195,219,178,223,162,234,214,12,219,208,23,44,21,3,90,125,93,114,77,34,109,20,69,101,59,212,158,7,8,118,175,66,105,237,183,63,191,203,227,40,40,215,38,212,214,1,47,97,188,60,124,142,116,128,113,135,4,47,141,194,16,214,58,85,109,62,206,209,79,51,245,98,179,153,217,229,235,72,105,61,144,182,7,95,192,149,159,8,152,218,246,235,42,107,65,59,55,111,1,209,243,208,74,120,248,240,155,215,124,207,189,31,221,94,179,134,199,130,187,202,254,49,2,181,31,205,80,114,104,112,98,188,156,219,152,234,30,6,141,109,111,194,112,12,58,16,53,191,130,8,192,148,245,77,216,35],\"claim\":{\"Value\":{\"pre\":{\"Value\":{\"pc\":2131092,\"merkle_root\":[257142831,212867192,3019768776,4077566949,3774766206,1955124911,2139138887,437463669]}},\"post\":{\"Value\":{\"pc\":0,\"merkle_root\":[0,0,0,0,0,0,0,0]}},\"exit_code\":{\"Halted\":0},\"input\":{\"Pruned\":[0,0,0,0,0,0,0,0]},\"output\":{\"Value\":{\"journal\":{\"Value\":[82,0,0,0,73,32,107,110,111,119,32,121,111,117,114,32,112,114,105,118,97,116,101,32,105,110,112,117,116,32,105,115,32,103,114,101,97,116,101,114,32,116,104,97,110,32,49,49,32,97,110,100,32,108,101,115,115,32,116,104,97,110,32,52,51,44,32,97,110,100,32,73,32,99,97,110,32,112,114,111,118,101,32,105,116,33,0,0]},\"assumptions\":{\"Value\":[]}}}}},\"verifier_parameters\":[2565148465,803857384,666633640,2019065645,1753987992,2480395716,2877601933,1558279850]}},\"journal\":{\"bytes\":[82,0,0,0,73,32,107,110,111,119,32,121,111,117,114,32,112,114,105,118,97,116,101,32,105,110,112,117,116,32,105,115,32,103,114,101,97,116,101,114,32,116,104,97,110,32,49,49,32,97,110,100,32,108,101,115,115,32,116,104,97,110,32,52,51,44,32,97,110,100,32,73,32,99,97,110,32,112,114,111,118,101,32,105,116,33,0,0]},\"metadata\":{\"verifier_parameters\":[2565148465,803857384,666633640,2019065645,1753987992,2480395716,2877601933,1558279850]}}";
//         println!("{:?}", receipt);
//         let risc_receipt: Receipt = serde_json::from_str(&receipt).unwrap();
//         println!("{:?}", risc_receipt);
//         match risc_receipt.inner {
//             InnerReceipt::Groth16(_) => {
//                 let seal =
//                     groth16::encode(risc_receipt.inner.groth16().unwrap().seal.clone()).unwrap();
//                 println!("seal {}", format!("0x{}", hex::encode(seal.clone())));
//                 let journal = risc_receipt.journal.bytes.clone();
//                 println!("journal {}", format!("0x{}", hex::encode(journal.clone())));

//                 let tokens = vec![Token::Bytes(seal), Token::Bytes(journal)];

//                 let result = encode(&tokens);
//                 println!(
//                     "bytes_seal_journal {}",
//                     format!("0x{}", hex::encode(result.clone()))
//                 );
//             }
//             InnerReceipt::Composite(_) => todo!(),
//             InnerReceipt::Succinct(_) => todo!(),
//             InnerReceipt::Fake(_) => todo!(),
//             _ => todo!(),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::abi::decode;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    fn create_dummy_elf() -> Vec<u8> {
        include_bytes!("../tests/hello_guest").to_vec()
        // vec![1, 2, 3, 4, 5]
    }

    fn create_compressed_binary() -> Vec<u8> {
        let binary = create_dummy_elf();
        let content = format!(
            r#"
        pub const TEST_ELF: &[u8] = &{:?};
        pub const TEST_ID: [u32; 8] = [1729087496, 1782151309, 3201877536, 1628959901, 2308880694, 3762575800, 943710734, 1150869179];
        "#,
            binary,
        );
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(content.as_bytes()).unwrap();
        encoder.finish().unwrap()
    }

    #[tokio::test]
    async fn test_new_project() {
        let server = Risc0Server::new();
        let compressed_binary = create_compressed_binary();

        let request = NewProjectRequest {
            project_id: 1,
            project_version: "".to_string(),
            binary: compressed_binary,
            metadata: vec![],
        };

        let response = server.new_project(Request::new(request)).await;
        assert!(response.is_ok());

        let projects = server.projects.read().await;
        assert!(projects.contains_key(&(1, 0)));

        let project = projects.get(&(1, 0)).unwrap();
        assert_eq!(project.project_id, 1);
        assert_eq!(project.elf, create_dummy_elf());
        assert_eq!(
            project.image_id,
            vec![
                1729087496, 1782151309, 3201877536, 1628959901, 2308880694, 3762575800, 943710734,
                1150869179
            ]
        );
    }

    #[tokio::test]
    async fn test_execute_task_local_prover() {
        let server = Risc0Server::new();

        // First, add a project
        let compressed_binary = create_compressed_binary();
        let new_project_request = NewProjectRequest {
            project_id: 1,
            project_version: "".to_string(),
            binary: compressed_binary,
            metadata: vec![],
        };
        server
            .new_project(Request::new(new_project_request))
            .await
            .unwrap();

        // Now, execute a task
        let payload = serde_json::json!({
            "private_input": "14",
            "public_input": "3,34",
            "receipt_type": "Stark"
        });
        let execute_request = ExecuteTaskRequest {
            project_id: 1,
            project_version: "".to_string(),
            task_id: "".as_bytes().to_vec(),
            payloads: vec![serde_json::to_vec(&payload).unwrap()],
        };

        let response = server.execute_task(Request::new(execute_request)).await;
        assert!(response.is_ok());

        let result = response.unwrap().into_inner().result;
        assert!(!result.is_empty());
    }

    // #[tokio::test]
    // async fn test_execute_task_bonsai_prover() {
    //     let server = Risc0Server::new();

    //     // First, add a project
    //     let compressed_binary = create_compressed_binary();
    //     let new_project_request = NewProjectRequest {
    //         project_id: 1,
    //         project_version: "".to_string(),
    //         binary: compressed_binary,
    //         metadata: vec![],
    //     };
    //     server
    //         .new_project(Request::new(new_project_request))
    //         .await
    //         .unwrap();

    //     // Now, execute a task
    //     let payload = serde_json::json!({
    //         "private_input": "14",
    //         "public_input": "3,34",
    //         "receipt_type": "Snark"
    //     });
    //     let execute_request = ExecuteTaskRequest {
    //         project_id: 1,
    //         project_version: "".to_string(),
    //         task_id: "".as_bytes().to_vec(),
    //         payloads: vec![serde_json::to_vec(&payload).unwrap()],
    //     };

    //     let response = server.execute_task(Request::new(execute_request)).await;
    //     assert!(response.is_ok());

    //     let result = response.unwrap().into_inner().result;
    //     // For Snark, we should be able to decode the result
    //     let decoded: (Vec<u8>, Vec<u8>) = decode(
    //         &[ethers::abi::ParamType::Bytes, ethers::abi::ParamType::Bytes],
    //         &result,
    //     )
    //     .unwrap();
    //     assert_eq!(decoded.0.len(), 192); // Groth16 seal length
    //     assert!(!decoded.1.is_empty()); // Journal should not be empty
    // }

    #[tokio::test]
    async fn test_execute_task_project_not_found() {
        let server = Risc0Server::new();

        let payload = serde_json::json!({
            "private_input": "14",
            "public_input": "3,34",
            "receipt_type": "Stark"
        });
        let execute_request = ExecuteTaskRequest {
            project_id: 1, // This project doesn't exist
            project_version: "".to_string(),
            task_id: "".as_bytes().to_vec(),
            payloads: vec![serde_json::to_vec(&payload).unwrap()],
        };

        let response = server.execute_task(Request::new(execute_request)).await;
        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn test_execute_task_empty_payload() {
        let server = Risc0Server::new();

        let execute_request = ExecuteTaskRequest {
            project_id: 1,
            project_version: "".to_string(),
            task_id: "".as_bytes().to_vec(),
            payloads: vec![], // Empty payload
        };

        let response = server.execute_task(Request::new(execute_request)).await;
        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn test_extract_data() {
        let server = Risc0Server::new();
        let content = r#"
        pub const TEST_ELF: &[u8] = &[1, 2, 3, 4, 5];
        pub const TEST_ID: [u32; 3] = [10, 20, 30];
        "#;

        let elf_data = server.extract_data::<u8>(&ELF_RE, content);
        assert_eq!(elf_data, Some(vec![1, 2, 3, 4, 5]));

        let id_data = server.extract_data::<u32>(&IMAGE_ID_RE, content);
        assert_eq!(id_data, Some(vec![10, 20, 30]));
    }
}
