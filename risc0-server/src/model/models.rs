use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProofType {
    Stark,
    Snark,
}

impl FromStr for ProofType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Stark" => Ok(ProofType::Stark),
            "Snark" => Ok(ProofType::Snark),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ProofType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProofType::Stark => write!(f, "Stark"),
            ProofType::Snark => write!(f, "Snark"),
        }
    }
}
