use core::time::Duration;

use risc0_zkvm::{
    compute_image_id, default_prover, serde::to_vec, ExecutorEnv, Receipt
};

use ::bonsai_sdk::alpha::responses::SnarkReceipt;
use anyhow::{anyhow, bail, Context, Result};
use bonsai_sdk::alpha as bonsai_sdk;
use serde::{Deserialize, Serialize};

const POLL_INTERVAL_SEC: u64 = 4;

#[derive(Debug, Deserialize, Serialize)]
pub enum RiscReceipt {
    /// The [Receipt].
    Stark(Receipt),

    /// The [SnarkReceipt].
    Snark(SnarkReceipt),
}

pub fn generate_proof_with_elf(
    project_id: u64,
    task_id: u64,
    client_id: String,
    sequencer_sign: String,
    input_datas: Vec<String>,
    elf: &[u8],
) -> Result<RiscReceipt> {
    let env = ExecutorEnv::builder()
        .write(&project_id).unwrap()
        .write(&task_id).unwrap()
        .write(&client_id).unwrap()
        .write(&sequencer_sign).unwrap()
        .write(&input_datas).unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();
    Ok(RiscReceipt::Stark(prover.prove(env, elf).unwrap()))

}

pub fn bonsai_prove(project_id: u64, task_id: u64, client_id: String, sequencer_sign: String, input_datas: Vec<String>, elf: &[u8], bonsai_url: String, bonsai_key: String) -> Result<RiscReceipt> {

    let project_id = to_vec(&project_id).unwrap();
    let task_id = to_vec(&task_id).unwrap();
    let client_id = to_vec(&client_id).unwrap();
    let sequencer_sign = to_vec(&sequencer_sign).unwrap();
    let inputs = to_vec(&input_datas).unwrap();

    let mut input_data: Vec<u8> = vec![];
    input_data.extend_from_slice(bytemuck::cast_slice(&project_id));
    input_data.extend_from_slice(bytemuck::cast_slice(&task_id));
    input_data.extend_from_slice(bytemuck::cast_slice(&client_id));
    input_data.extend_from_slice(bytemuck::cast_slice(&sequencer_sign));
    input_data.extend_from_slice(bytemuck::cast_slice(&inputs));

    let client = bonsai_sdk::Client::from_parts(bonsai_url, bonsai_key, risc0_zkvm::VERSION)?;
    // upload the image
    let image_id = compute_image_id(elf)?;
    let image_id_hex = hex::encode(image_id);

    // ImageIdExists indicates that this image has already been uploaded to bonsai.
    // If this is the case, simply move on to uploading the input.
    client.upload_img(&image_id_hex, elf.to_vec())?;

    // upload input data
    let input_id = client.upload_input(input_data)?;

    // upload receipts
    let assumptions: Vec<String> = vec![];

    // While this is the executor, we want to start a session on the bonsai prover.
    // By doing so, we can return a session ID so that the prover can use it to
    // retrieve the receipt.
    let session = client.create_session(image_id_hex, input_id, assumptions)?;

    // Poll and await the result of the STARK rollup proving session.
    let _receipt: Receipt = (|| {
        loop {
            let res = match session.status(&client) {
                Ok(res) => res,
                Err(err) => {
                    eprint!("Failed to get session status: {err}");
                    std::thread::sleep(Duration::from_secs(POLL_INTERVAL_SEC));
                    continue;
                }
            };
            match res.status.as_str() {
                "RUNNING" => {
                    std::thread::sleep(Duration::from_secs(POLL_INTERVAL_SEC));
                }
                "SUCCEEDED" => {
                    println!("receipt url :{:?}", &res.receipt_url);
                    let receipt_buf = client
                        .download(
                            &res.receipt_url
                                .context("Missing 'receipt_url' on status response")?,
                        )
                        .context("Failed to download receipt")?;
                    let receipt: Receipt = bincode::deserialize(&receipt_buf)
                        .context("Failed to deserialize SessionReceipt")?;
                    println!("stark receipt {:?}", receipt);
                    // eprintln!("Completed STARK proof on bonsai alpha backend!");
                    return Ok(receipt);
                }
                _ => {
                    bail!(
                        "STARK proving session exited with bad status: {}",
                        res.status
                    );
                }
            }
        }
    })()?;

    let snark_session = client.create_snark(session.uuid)?;
    let snark_receipt: SnarkReceipt = (|| loop {
        let res = snark_session.status(&client)?;
        match res.status.as_str() {
            "RUNNING" => {
                std::thread::sleep(Duration::from_secs(POLL_INTERVAL_SEC));
            }
            "SUCCEEDED" => {
                // eprintln!("Completed SNARK proof on bonsai alpha backend!");
                return res
                    .output
                    .ok_or(anyhow!("output expected to be non-empty on success"));
            }
            _ => {
                bail!(
                    "SNARK proving session exited with bad status: {}",
                    res.status
                );
            }
        }
    })()?;
    Ok(RiscReceipt::Snark(snark_receipt))
}
