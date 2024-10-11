use std::env;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Result;
use dotenvy::dotenv;
use dyn_clone::DynClone;
use risc0_zkvm::{
    default_prover, ExecutorEnv, Prover as risc0Prover, ProverOpts, Receipt, VerifierContext,
};

pub trait Prover: DynClone + Send + Sync {
    fn prove(&self, data: Vec<String>) -> Result<Receipt>;
}

dyn_clone::clone_trait_object!(Prover);

#[derive(Clone)]
pub struct LocalProver {
    prover: Arc<ProverWrapper>,
    elf: Vec<u8>,
}

struct ProverWrapper {
    inner: Rc<dyn risc0Prover>,
}

impl ProverWrapper {
    fn new(prover: Rc<dyn risc0Prover>) -> Self {
        ProverWrapper { inner: prover }
    }
}

unsafe impl Send for ProverWrapper {}
unsafe impl Sync for ProverWrapper {}

impl LocalProver {
    pub fn new(elf: &[u8]) -> Self {
        LocalProver {
            prover: Arc::new(ProverWrapper::new(default_prover())),
            elf: elf.to_vec(),
        }
    }
}

impl Prover for LocalProver {
    fn prove(&self, data: Vec<String>) -> Result<Receipt> {
        let env = ExecutorEnv::builder().write(&data)?.build()?;

        Ok(self.prover.inner.prove(env, &self.elf)?.receipt)
    }
}

#[derive(Clone)]
pub struct BonsaiProver {
    elf: Vec<u8>,
    prover: Arc<ProverWrapper>,
}

impl BonsaiProver {
    pub fn new(elf: &[u8]) -> Self {
        dotenv().ok();

        env::var("BONSAI_API_URL").expect("BONSAI_API_URL must be set");
        env::var("BONSAI_API_KEY").expect("BONSAI_API_KEY must be set");

        BonsaiProver {
            prover: Arc::new(ProverWrapper::new(default_prover())),
            elf: elf.to_vec(),
        }
    }
}

impl Prover for BonsaiProver {
    fn prove(&self, data: Vec<String>) -> Result<Receipt> {
        let env = ExecutorEnv::builder().write(&data)?.build()?;

        Ok(self
            .prover
            .inner
            .prove_with_ctx(
                // no bonsai -> Groth16Receipt   with bonsai -> Groth16
                env,
                &VerifierContext::default(),
                &self.elf,
                &ProverOpts::groth16(),
            )?
            .receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    pub const HELLO_GUEST_ID: [u32; 8] = [
        1729087496, 1782151309, 3201877536, 1628959901, 2308880694, 3762575800, 943710734,
        1150869179,
    ];

    fn create_dummy_elf() -> Vec<u8> {
        include_bytes!("../tests/hello_guest").to_vec()
    }

    #[test]
    fn test_local_prover_creation() {
        let elf = create_dummy_elf();
        let prover = LocalProver::new(&elf);
        assert_eq!(prover.elf, elf);
    }

    #[test]
    fn test_local_prover_prove() {
        let elf = create_dummy_elf();
        let prover = LocalProver::new(&elf);
        let result = prover.prove(vec!["test1".to_string(), "test2".to_string()]);
        let verify_result = result.unwrap().verify(HELLO_GUEST_ID);
        assert!(verify_result.is_ok(), "Error: {:?}", verify_result.err());
    }

    #[test]
    fn test_bonsai_prover_creation() {
        // Set dummy environment variables
        env::set_var("BONSAI_API_URL", "http://dummy-url.com");
        env::set_var("BONSAI_API_KEY", "dummy-key");

        let elf = create_dummy_elf();
        let prover = BonsaiProver::new(&elf);
        assert_eq!(prover.elf, elf);
    }

    #[test]
    fn test_bonsai_prover_prove() {
        // Set dummy environment variables
        env::set_var("BONSAI_API_URL", "https://api.bonsai.xyz");
        env::set_var("BONSAI_API_KEY", "");

        let elf = create_dummy_elf();
        let prover = BonsaiProver::new(&elf);
        let result = prover.prove(vec!["test".to_string(), "test2".to_string()]);

        let ans: u32 = result.unwrap().journal.decode().unwrap();
        println!("ans: {}", ans);
    }
}
