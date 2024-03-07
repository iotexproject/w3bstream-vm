use std::env;
use dotenvy::dotenv;
use anyhow::Result;

use crate::{
    core::prove::{bonsai_prove, generate_proof_with_elf, RiscReceipt},
    db::{self},
    model::models::ProofType,
    tools::parse_elf_from_str,
};

pub async fn get_receipt(
    image_id: String,
    input_datas: String,
    receipt_type: ProofType,
) -> Result<String, String> {
    let connection = &mut db::pgdb::establish_connection();
    let proof = db::pgdb::create_proof(
        connection,
        &image_id,
        &input_datas,
        "public_input",
        &receipt_type.to_string(),
        "generating",
    );
    let mut vm = db::models::Vm::new();
    let vm_result = db::pgdb::get_vm(connection, &image_id);
    match vm_result {
        Ok(v) => vm = v,
        Err(e) => println!("image_id parse error: {}", e),
    }
    let elf_cont: Vec<u8> = parse_elf_from_str(&vm.elf);

    let receipt: Result<RiscReceipt>;
    match receipt_type {
        ProofType::Stark => {
            receipt = generate_proof_with_elf(&input_datas, &elf_cont);
        }
        ProofType::Snark => {
            dotenv().ok();

            let bonsai_url = env::var("BONSAI_URL").expect("BONSAI_URL must be set");
            let bonsai_key = env::var("BONSAI_KEY").expect("BONSAI_KEY must be set");
            // TODO
            receipt = tokio::task::spawn_blocking(move || { 
                bonsai_prove(
                    &input_datas,
                    &elf_cont,
                    bonsai_url,
                    bonsai_key,
               )
            }).await.unwrap();
        }
    }

    match receipt {
        Ok(r) => {
            let receipt_str = serde_json::to_string(&r).unwrap();
            let _ = db::pgdb::update_proof_with_receipt(connection, &proof, &receipt_str);
            return Ok(receipt_str);
        }
        Err(e) => {
            println!("generate proof error, Error: {:?}", e);
            let _ = db::pgdb::update_proof_status_with_receipt(
                connection,
                &proof,
                &"failed".to_string(),
            );
            return Err(format!("generate proof error, Error: {:?}", e));
        }
    }
}
