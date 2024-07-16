use std::{io::Read, str::FromStr};

use ethers::abi::{encode, Token};
use flate2::read::ZlibDecoder;
use hex::FromHex;
use serde_json::Value;
use tonic::{Request, Response, Status};

use crate::{core::prove::{tokenize_snark_receipt, RiscReceipt}, db, handlers::proof::get_receipt, model::models::ProofType};

use rust_grpc::grpc::vm_runtime::{vm_runtime_server::VmRuntime, CreateRequest, CreateResponse, ExecuteRequest, ExecuteResponse};


#[derive(Debug)]
pub struct Risc0Server {}

#[tonic::async_trait]
impl VmRuntime for Risc0Server {
    async fn create(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<Response<CreateResponse>, Status> {
        println!("risc0 instance create...");
        let request = request.into_inner();

        let project_id = request.project_id;
        let project = project_id.to_string();
        let content = request.content;
        let compressed_data = Vec::from_hex(content).unwrap();
        let mut decoder = ZlibDecoder::new(&compressed_data[..]);
        let mut content = Vec::new();
        decoder.read_to_end(&mut content)?;

        let exp_param = request.exp_param;

        // exp_param = {"image_id":"RANGE_ID", "elf":"RANGE_ELF"}
        if exp_param == "" {
            return Err(Status::invalid_argument("need exp_param"));
        }

        let v: Value = serde_json::from_str(&exp_param).unwrap();
        let image_id = v["image_id"].as_str().unwrap().to_string();
        let elf = v["elf"].as_str().unwrap().to_string();
        let content = String::from_utf8(content).unwrap();

        let mut elf_str = String::new();
        let mut id_str = String::new();

        for line in content.lines() {
            if line.contains(&elf) {
                let vec: Vec<&str> = line.split("=").collect();
                elf_str = vec[1].trim()[2..vec[1].trim().len() - 2].to_string();
            }
            if line.contains(&image_id) {
                let vec: Vec<&str> = line.split("=").collect();
                id_str = vec[1].trim()[1..vec[1].trim().len() - 2].to_string();
            }
        }

        let connection = &mut db::pgdb::establish_connection();
        let _ = db::pgdb::create_vm(connection, &project, &elf_str, &id_str);

        Ok(Response::new(CreateResponse {}))
    }

    async fn execute_operator(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        println!("risc0 instance execute...");
        let request = request.into_inner();

        let project_id = request.project_id;
        let task_id = request.task_id;
        let client_id = request.client_id;
        let sequencer_signature = request.sequencer_signature;
        let datas = request.datas;

        if datas.len() == 0 {
            return Err(Status::invalid_argument("need datas"))
        }

        let connection = &mut db::pgdb::establish_connection();
        let image_id = match db::pgdb::get_vm_by_project(connection, &project_id.to_string()) {
            Ok(v) => v.image_id,
            Err(_) => return Err(Status::not_found(format!("{} not found", project_id.to_string()))),
        };

        // TODO move to guest method
        // param = {"private_input":"14", "public_input":"3,34", "receipt_type":"Snark"}
        // let input_datas = json!(datas).to_string();
        let v: Value = serde_json::from_str(&datas[0]).unwrap();
        let mut receipt_type = ProofType::from_str("Snark").unwrap();
        // TODO check v
        if v.get("receipt_type").is_some() {
            receipt_type = v["receipt_type"].as_str().unwrap().to_string().parse().unwrap();
        }
        // let receipt_type: Result<ProofType, _> =
        //     v["receipt_type"].as_str().unwrap().to_string().parse();
        // let receipt_type = receipt_type.unwrap();

        let receipt = match get_receipt(
            project_id,
            task_id,
            client_id,
            sequencer_signature,
            image_id,
            datas,
            receipt_type.clone(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => return Err(Status::cancelled(e)),
        };

        println!("{:?}", receipt);
        let risc_receipt: RiscReceipt = serde_json::from_str(&receipt).unwrap();
        let result = match risc_receipt {
            RiscReceipt::Stark(_) => receipt.as_bytes().to_vec(),
            RiscReceipt::Snark(snark_receipt) => {
                let tokens = vec![
                    Token::Bytes(ethers::abi::encode(&[tokenize_snark_receipt(
                        &snark_receipt.snark,
                    ).unwrap()])),
                    Token::FixedBytes(snark_receipt.post_state_digest),
                    Token::Bytes(snark_receipt.journal),
                ];

                let packed = encode(&tokens);
                println!("bytes_calldata_journal {}", format!("0x{}", hex::encode(packed.clone())));
                packed
            },
        };

        Ok(Response::new(ExecuteResponse {
            result,
        }))
    }
}

#[test]
fn param() {
    let receipt = "{\"Snark\":{\"snark\":{\"a\":[[15,155,252,242,9,26,124,151,21,24,211,115,131,235,68,49,209,28,150,199,46,118,241,255,86,226,68,209,109,120,35,47],[33,224,18,148,206,102,224,66,80,117,215,130,118,55,94,179,94,53,212,116,5,165,226,142,248,241,141,130,52,35,128,67]],\"b\":[[[16,14,32,254,8,24,79,65,20,193,148,13,1,193,112,205,28,92,131,221,156,231,2,188,232,168,3,189,210,208,39,20],[4,31,195,116,249,130,44,45,124,142,249,191,172,22,136,236,220,245,163,164,21,15,24,220,91,173,156,126,103,197,98,115]],[[26,214,251,166,96,158,106,222,67,205,64,40,182,73,164,129,7,146,142,253,159,248,80,22,165,21,222,64,120,62,244,243],[10,60,178,134,188,178,36,3,72,198,206,165,155,20,0,86,254,87,96,235,186,172,122,29,229,115,250,116,43,187,20,164]]],\"c\":[[31,3,43,201,153,47,46,183,184,221,113,155,168,84,59,207,53,46,26,38,144,251,214,34,246,45,255,46,134,131,56,252],[35,27,218,215,220,204,109,38,61,166,87,219,101,49,152,114,107,170,246,58,197,73,58,45,158,181,73,124,182,253,198,6]]},\"post_state_digest\":[163,172,194,113,23,65,137,150,52,11,132,229,169,15,62,244,196,157,34,199,158,68,170,216,34,236,156,49,62,30,184,226],\"journal\":[82,0,0,0,73,32,107,110,111,119,32,121,111,117,114,32,112,114,105,118,97,116,101,32,105,110,112,117,116,32,105,115,32,103,114,101,97,116,101,114,32,116,104,97,110,32,49,49,32,97,110,100,32,108,101,115,115,32,116,104,97,110,32,52,51,44,32,97,110,100,32,73,32,99,97,110,32,112,114,111,118,101,32,105,116,33,0,0]}}";
    println!("{:?}", receipt);
    let risc_receipt: RiscReceipt = serde_json::from_str(&receipt).unwrap();
    println!("{:?}", risc_receipt);
    match risc_receipt {
        RiscReceipt::Stark(_) => todo!(),
        RiscReceipt::Snark(snark_receipt) => {
// bytes_memory_seal "0x07a0ccc0b07afce3eea573a3ac5fe40c640792d62bc9639da826ece65dfb7c1714eca637538e88554970cd9f5c645798fd7e12713eed8dd09a7aaa36895417511c690d1c3d10092f26b46af71836273932595e33192ce277a4d0406a7068e7442ba2a0a365b67efd7622bdd0291c934104b61272a27ac5431c6f61a19fc7256e02a84066ddc37f3d826b3f6c51c04e8f03672d6f8835c63fe4bb3c83d79286a11140900f46e946b2ca4e373c2681862fe8526d8371d0edd3f28a92fc414c628014f9eb8a84c582295163b0e33731775f22b72fec0931a0fe6e502f6e1169155917114e868efa132410d817938abc55da64081661296fa4acc0d7ebe05322d5e6"
// bytes32_post_state_digest 0x3b858303e7b1760b1f33353348638aed27c00d9198374fd2991c40c61afbea75
// bytes_calldata_journal 0x5100000049206b6e6f7720796f7572207072697661746520696e7075742069732067726561746572207468616e203320616e64206c657373207468616e2033342c20616e6420492063616e2070726f766520697421000000

            let tokens = vec![
                Token::Bytes(ethers::abi::encode(&[tokenize_snark_receipt(
                    &snark_receipt.snark,
                ).unwrap()])),
                Token::FixedBytes(snark_receipt.post_state_digest),
                Token::Bytes(snark_receipt.journal),
            ];

            let packed = encode(&tokens);
            println!("packed {}", format!("0x{}", hex::encode(packed)));
        },
    }
}