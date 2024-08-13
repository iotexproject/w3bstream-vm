use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, Receipt, VerifierContext};

use anyhow::Result;

pub fn generate_proof_with_elf(
    project_id: u64,
    task_id: u64,
    client_id: String,
    sequencer_sign: String,
    input_datas: Vec<String>,
    elf: &[u8],
) -> Result<Receipt> {
    let env = ExecutorEnv::builder()
        .write(&project_id)
        .unwrap()
        .write(&task_id)
        .unwrap()
        .write(&client_id)
        .unwrap()
        .write(&sequencer_sign)
        .unwrap()
        .write(&input_datas)
        .unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();
    Ok(prover.prove(env, elf).unwrap().receipt)
}

pub fn bonsai_groth16_prove_with_env(
    project_id: u64,
    task_id: u64,
    client_id: String,
    sequencer_sign: String,
    input_datas: Vec<String>,
    elf: &[u8],
    bonsai_url: String,
    bonsai_key: String,
) -> Result<Receipt> {
    let env = ExecutorEnv::builder()
        .write(&project_id)
        .unwrap()
        .write(&task_id)
        .unwrap()
        .write(&client_id)
        .unwrap()
        .write(&sequencer_sign)
        .unwrap()
        .write(&input_datas)
        .unwrap()
        .build()
        .unwrap();

    std::env::set_var("BONSAI_API_URL", bonsai_url);
    std::env::set_var("BONSAI_API_KEY", bonsai_key);

    // Obtain the default prover.
    let receipt = default_prover()
        .prove_with_ctx(
            // no bonsai -> Groth16Receipt   with bonsai -> Groth16
            env,
            &VerifierContext::default(),
            elf,
            &ProverOpts::groth16(),
        )
        .unwrap()
        .receipt;

    Ok(receipt)
}
