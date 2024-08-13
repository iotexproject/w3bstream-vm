use anyhow::Result;
use dotenvy::dotenv;
use risc0_zkvm::Receipt;
use std::env;

use crate::{
    core::prove::{bonsai_groth16_prove_with_env, generate_proof_with_elf},
    db::{self},
    model::models::ProofType,
    tools::parse_elf_from_str,
};

pub async fn get_receipt(
    project_id: u64,
    task_id: u64,
    client_id: String,
    sequencer_sign: String,
    image_id: String,
    datas_input: Vec<String>,
    receipt_type: ProofType,
) -> Result<String, String> {
    let connection = &mut db::pgdb::establish_connection();
    let proof = db::pgdb::create_proof(
        connection,
        &project_id.to_string(),
        &task_id.to_string(),
        &client_id,
        &sequencer_sign,
        &image_id,
        &datas_input.join("#"),
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

    let receipt: Result<Receipt>;
    match receipt_type {
        ProofType::Stark => {
            receipt = generate_proof_with_elf(
                project_id,
                task_id,
                client_id,
                sequencer_sign,
                datas_input,
                &elf_cont,
            );
        }
        ProofType::Snark => {
            dotenv().ok();

            let bonsai_url = env::var("BONSAI_URL").expect("BONSAI_URL must be set");
            let bonsai_key = env::var("BONSAI_KEY").expect("BONSAI_KEY must be set");
            // TODO
            receipt = tokio::task::spawn_blocking(move || {
                bonsai_groth16_prove_with_env(
                    project_id,
                    task_id,
                    client_id,
                    sequencer_sign,
                    datas_input,
                    &elf_cont,
                    bonsai_url,
                    bonsai_key,
                )
            })
            .await
            .unwrap();
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
