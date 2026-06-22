

use anyhow::{anyhow, Result};
use common::crypto_config::{CryptoConfig, SignatureAlgorithm};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use pqcrypto_dilithium::dilithium3::{PublicKey, SecretKey, DetachedSignature};
use pqcrypto_dilithium::dilithium3;
use pqcrypto_traits::sign::{PublicKey as PqPublicKey, SecretKey as PqSecretKey, DetachedSignature as PqSignature};

/// Wrapper unificado para chave de assinatura
pub enum SigningKeyWrapper {
    Ed25519(SigningKey),
    MlDsa(SecretKey),
}

impl SigningKeyWrapper {
    pub fn generate(alg: SignatureAlgorithm) -> Result<Self> {
        match alg {
            SignatureAlgorithm::Ed25519 => {
                let mut rng = rand_core::OsRng;
                Ok(Self::Ed25519(SigningKey::generate(&mut rng)))
            }
            SignatureAlgorithm::MlDsa => {
                let (_, sk) = dilithium3::keypair();
                Ok(Self::MlDsa(sk))
            }
            _ => Err(anyhow!("Algoritmo não suportado para geração de chave")),
        }
    }

    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        match self {
            Self::Ed25519(sk) => {
                let sig = sk.sign(message);
                Ok(sig.to_bytes().to_vec())
            }
            Self::MlDsa(sk) => {
                let sig = dilithium3::detached_sign(message, sk);
                Ok(sig.as_bytes().to_vec())
            }
        }
    }

    pub fn algorithm(&self) -> SignatureAlgorithm {
        match self {
            Self::Ed25519(_) => SignatureAlgorithm::Ed25519,
            Self::MlDsa(_) => SignatureAlgorithm::MlDsa,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ed25519(sk) => sk.to_bytes().to_vec(),
            Self::MlDsa(sk) => sk.as_bytes().to_vec(),
        }
    }

    pub fn from_bytes(alg: SignatureAlgorithm, bytes: &[u8]) -> Result<Self> {
        match alg {
            SignatureAlgorithm::Ed25519 => {
                let arr: [u8; 32] = bytes.try_into()
                    .map_err(|_| anyhow!("Tamanho inválido para chave Ed25519"))?;
                Ok(Self::Ed25519(SigningKey::from_bytes(&arr)))
            }
            SignatureAlgorithm::MlDsa => {
                let sk = SecretKey::from_bytes(bytes)
                    .map_err(|e| anyhow!("Falha ao carregar chave ML-DSA: {}", e))?;
                Ok(Self::MlDsa(sk))
            }
            _ => Err(anyhow!("Algoritmo não suportado para desserialização")),
        }
    }
}

/// Wrapper unificado para chave de verificação
pub enum VerifyingKeyWrapper {
    Ed25519(VerifyingKey),
    MlDsa(PublicKey),
}

impl VerifyingKeyWrapper {
    pub fn from_bytes(alg: SignatureAlgorithm, bytes: &[u8]) -> Result<Self> {
        match alg {
            SignatureAlgorithm::Ed25519 => {
                let arr: [u8; 32] = bytes.try_into()
                    .map_err(|_| anyhow!("Tamanho inválido para chave Ed25519"))?;
                Ok(Self::Ed25519(VerifyingKey::from_bytes(&arr)?))
            }
            SignatureAlgorithm::MlDsa => {
                let pk = PublicKey::from_bytes(bytes)
                    .map_err(|e| anyhow!("Falha ao carregar chave ML-DSA: {}", e))?;
                Ok(Self::MlDsa(pk))
            }
            _ => Err(anyhow!("Algoritmo não suportado")),
        }
    }

    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool> {
        match self {
            Self::Ed25519(vk) => {
                let sig_bytes: [u8; 64] = signature.try_into().map_err(|_| anyhow!("Assinatura ed25519 deve ter 64 bytes"))?;
                let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);
                Ok(vk.verify(message, &sig).is_ok())
            }
            Self::MlDsa(vk) => {
                let sig = DetachedSignature::from_bytes(signature).map_err(|e| anyhow!("Assinatura inválida: {}", e))?;
                Ok(dilithium3::verify_detached_signature(&sig, message, vk).is_ok())
            }
        }
    }

    pub fn algorithm(&self) -> SignatureAlgorithm {
        match self {
            Self::Ed25519(_) => SignatureAlgorithm::Ed25519,
            Self::MlDsa(_) => SignatureAlgorithm::MlDsa,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ed25519(vk) => vk.to_bytes().to_vec(),
            Self::MlDsa(vk) => vk.as_bytes().to_vec(),
        }
    }
}

/// Fábrica criptográfica com suporte a dual‑stack
pub struct CryptoFactory {
    config: CryptoConfig,
}

impl CryptoFactory {
    pub fn new(config: CryptoConfig) -> Self {
        Self { config }
    }

    /// Gera um par de chaves para o algoritmo principal
    pub fn generate_signing_key(&self) -> Result<SigningKeyWrapper> {
        SigningKeyWrapper::generate(self.config.signature_algorithm)
    }

    /// Gera um par de chaves para o algoritmo de fallback (se configurado)
    pub fn generate_fallback_key(&self) -> Result<Option<SigningKeyWrapper>> {
        if let Some(alg) = self.config.fallback_signature_algorithm {
            Ok(Some(SigningKeyWrapper::generate(alg)?))
        } else {
            Ok(None)
        }
    }

    /// Carrega chave de verificação a partir de bytes (tenta ambos os algoritmos se dual‑stack)
    pub fn load_verifying_key(&self, bytes: &[u8]) -> Result<VerifyingKeyWrapper> {
        // Primeiro tenta o algoritmo principal
        if let Ok(key) = VerifyingKeyWrapper::from_bytes(self.config.signature_algorithm, bytes) {
            return Ok(key);
        }
        // Se falhar e houver fallback, tenta o fallback
        if let Some(fallback) = self.config.fallback_signature_algorithm {
            if let Ok(key) = VerifyingKeyWrapper::from_bytes(fallback, bytes) {
                return Ok(key);
            }
        }
        Err(anyhow!("Não foi possível carregar chave de verificação"))
    }

    /// Assina uma mensagem usando o algoritmo principal (e opcionalmente o fallback para dual‑stack)
    pub fn sign(&self, key: &SigningKeyWrapper, message: &[u8]) -> Result<Vec<u8>> {
        key.sign(message)
    }

    /// Verifica uma assinatura, tentando ambos os algoritmos se dual‑stack
    pub fn verify_dual(
        &self,
        primary_key: &VerifyingKeyWrapper,
        fallback_key: Option<&VerifyingKeyWrapper>,
        message: &[u8],
        signature: &[u8],
    ) -> Result<bool> {
        // Tenta verificar com a chave primária
        if primary_key.verify(message, signature)? {
            return Ok(true);
        }
        // Se falhar e houver fallback, tenta com o fallback
        if let Some(fb_key) = fallback_key {
            if fb_key.verify(message, signature)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_stack_ed25519_ml_dsa() {
        let config = CryptoConfig {
            signature_algorithm: SignatureAlgorithm::Ed25519,
            fallback_signature_algorithm: Some(SignatureAlgorithm::MlDsa),
            dual_stack_mode: true,
            ..Default::default()
        };
        let factory = CryptoFactory::new(config);

        // Gera ambas as chaves
        let primary_sk = factory.generate_signing_key().unwrap();
        let fallback_sk = factory.generate_fallback_key().unwrap().unwrap();

        let primary_vk = match &primary_sk {
            SigningKeyWrapper::Ed25519(sk) => VerifyingKeyWrapper::Ed25519(sk.verifying_key()),
            _ => panic!(),
        };
        let fallback_vk = match &fallback_sk {
            SigningKeyWrapper::MlDsa(_) => {
                let (pk, _) = dilithium3::keypair();
                VerifyingKeyWrapper::MlDsa(pk)
            }
            _ => panic!(),
        };

        let msg = b"dual-stack test";
        // Assina com primário e verifica com ambos
        let sig = factory.sign(&primary_sk, msg).unwrap();
        assert!(factory.verify_dual(&primary_vk, Some(&fallback_vk), msg, &sig).unwrap());
    }
}
