use std::{io::Read, str::FromStr};

use ethers::abi::{encode, Token};
use flate2::read::ZlibDecoder;
use hex::FromHex;
use risc0_ethereum_contracts::groth16;
use risc0_zkvm::{InnerReceipt, Receipt};
use serde_json::Value;
use tonic::{Request, Response, Status};

use crate::{db, handlers::proof::get_receipt, model::models::ProofType};

use rust_grpc::grpc::vm_runtime::{
    vm_runtime_server::VmRuntime, CreateRequest, CreateResponse, ExecuteRequest, ExecuteResponse,
};

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

        let exp_param = &request.exp_params[0];

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

    async fn execute(
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
            return Err(Status::invalid_argument("need datas"));
        }

        let connection = &mut db::pgdb::establish_connection();
        let image_id = match db::pgdb::get_vm_by_project(connection, &project_id.to_string()) {
            Ok(v) => v.image_id,
            Err(_) => {
                return Err(Status::not_found(format!(
                    "{} not found",
                    project_id.to_string()
                )))
            }
        };

        // TODO move to guest method
        // param = {"private_input":"14", "public_input":"3,34", "receipt_type":"Snark"}
        // let input_datas = json!(datas).to_string();
        let v: Value = serde_json::from_str(&datas[0]).unwrap();
        let mut receipt_type = ProofType::from_str("Snark").unwrap();
        // TODO check v
        if v.get("receipt_type").is_some() {
            receipt_type = v["receipt_type"]
                .as_str()
                .unwrap()
                .to_string()
                .parse()
                .unwrap();
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
        let mut result = receipt.as_bytes().to_vec();
        let risc_receipt: Receipt = serde_json::from_str(&receipt).unwrap();
        if matches!(risc_receipt.inner, InnerReceipt::Groth16(_)) {
            let seal = groth16::encode(risc_receipt.inner.groth16().unwrap().seal.clone()).unwrap();
            let journal = risc_receipt.journal.bytes.clone();

            let tokens = vec![Token::Bytes(seal), Token::Bytes(journal)];

            result = encode(&tokens);
        }

        Ok(Response::new(ExecuteResponse { result }))
    }
}

#[test]
fn param() {
    let receipt = "{\"inner\":{\"Groth16\":{\"seal\":[13,130,138,212,74,234,176,239,95,29,228,225,143,171,188,84,199,197,244,148,214,38,199,17,199,58,134,101,217,241,136,201,12,22,9,213,157,120,249,214,255,96,0,200,170,149,120,109,143,229,183,161,226,83,220,46,139,2,50,113,217,0,187,186,47,205,143,50,147,175,156,184,200,208,39,186,129,20,149,87,98,224,66,93,24,245,84,67,67,81,183,244,100,51,144,189,18,149,157,185,195,219,178,223,162,234,214,12,219,208,23,44,21,3,90,125,93,114,77,34,109,20,69,101,59,212,158,7,8,118,175,66,105,237,183,63,191,203,227,40,40,215,38,212,214,1,47,97,188,60,124,142,116,128,113,135,4,47,141,194,16,214,58,85,109,62,206,209,79,51,245,98,179,153,217,229,235,72,105,61,144,182,7,95,192,149,159,8,152,218,246,235,42,107,65,59,55,111,1,209,243,208,74,120,248,240,155,215,124,207,189,31,221,94,179,134,199,130,187,202,254,49,2,181,31,205,80,114,104,112,98,188,156,219,152,234,30,6,141,109,111,194,112,12,58,16,53,191,130,8,192,148,245,77,216,35],\"claim\":{\"Value\":{\"pre\":{\"Value\":{\"pc\":2131092,\"merkle_root\":[257142831,212867192,3019768776,4077566949,3774766206,1955124911,2139138887,437463669]}},\"post\":{\"Value\":{\"pc\":0,\"merkle_root\":[0,0,0,0,0,0,0,0]}},\"exit_code\":{\"Halted\":0},\"input\":{\"Pruned\":[0,0,0,0,0,0,0,0]},\"output\":{\"Value\":{\"journal\":{\"Value\":[82,0,0,0,73,32,107,110,111,119,32,121,111,117,114,32,112,114,105,118,97,116,101,32,105,110,112,117,116,32,105,115,32,103,114,101,97,116,101,114,32,116,104,97,110,32,49,49,32,97,110,100,32,108,101,115,115,32,116,104,97,110,32,52,51,44,32,97,110,100,32,73,32,99,97,110,32,112,114,111,118,101,32,105,116,33,0,0]},\"assumptions\":{\"Value\":[]}}}}},\"verifier_parameters\":[2565148465,803857384,666633640,2019065645,1753987992,2480395716,2877601933,1558279850]}},\"journal\":{\"bytes\":[82,0,0,0,73,32,107,110,111,119,32,121,111,117,114,32,112,114,105,118,97,116,101,32,105,110,112,117,116,32,105,115,32,103,114,101,97,116,101,114,32,116,104,97,110,32,49,49,32,97,110,100,32,108,101,115,115,32,116,104,97,110,32,52,51,44,32,97,110,100,32,73,32,99,97,110,32,112,114,111,118,101,32,105,116,33,0,0]},\"metadata\":{\"verifier_parameters\":[2565148465,803857384,666633640,2019065645,1753987992,2480395716,2877601933,1558279850]}}";
    println!("{:?}", receipt);
    let risc_receipt: Receipt = serde_json::from_str(&receipt).unwrap();
    println!("{:?}", risc_receipt);
    match risc_receipt.inner {
        InnerReceipt::Groth16(_) => {
            let seal = groth16::encode(risc_receipt.inner.groth16().unwrap().seal.clone()).unwrap();
            println!("seal {}", format!("0x{}", hex::encode(seal.clone())));
            let journal = risc_receipt.journal.bytes.clone();
            println!("journal {}", format!("0x{}", hex::encode(journal.clone())));

            let tokens = vec![Token::Bytes(seal), Token::Bytes(journal)];

            let result = encode(&tokens);
            println!(
                "bytes_seal_journal {}",
                format!("0x{}", hex::encode(result.clone()))
            );
        }
        InnerReceipt::Composite(_) => todo!(),
        InnerReceipt::Succinct(_) => todo!(),
        InnerReceipt::Fake(_) => todo!(),
        _ => todo!(),
    }
}
