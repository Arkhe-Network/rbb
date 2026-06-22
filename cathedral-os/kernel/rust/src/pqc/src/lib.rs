// src/pqc.rs — Implementação PQC em Rust
use slh_dsa::*;
use kyber::*;
use serde::{Serialize, Deserialize};

pub struct PQCManager {
    slh_dsa: SLHDSA,
    kyber: Kyber,
}

impl PQCManager {
    pub fn new() -> Self {
        Self {
            slh_dsa: SLHDSA::new(SecurityLevel::SLH_DSA_256),
            kyber: Kyber::new(SecurityLevel::Kyber768),
        }
    }

    pub fn generate_slh_dsa_keypair(&self) -> (Vec<u8>, Vec<u8>) {
        let (public, private) = self.slh_dsa.generate_keypair();
        (public, private)
    }

    pub fn sign_slh_dsa(&self, message: &[u8], private_key: &[u8]) -> Result<Vec<u8>> {
        self.slh_dsa.sign(message, private_key)
    }

    pub fn verify_slh_dsa(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
        self.slh_dsa.verify(message, signature, public_key)
    }

    pub fn generate_kyber_keypair(&self) -> (Vec<u8>, Vec<u8>) {
        let (public, private) = self.kyber.generate_keypair();
        (public, private)
    }

    pub fn encapsulate_kyber(&self, public_key: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let (ciphertext, shared_secret) = self.kyber.encapsulate(public_key).unwrap();
        (ciphertext, shared_secret)
    }

    pub fn decapsulate_kyber(&self, ciphertext: &[u8], private_key: &[u8]) -> Result<Vec<u8>> {
        self.kyber.decapsulate(ciphertext, private_key)
    }
}
