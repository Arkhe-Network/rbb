#[derive(Debug, Clone)]
pub struct TEEContext {
    pub enclave_type: EnclaveType,
    pub attestation: AttestationMode,
    pub hard_reset_crypto: bool,
    pub zeroize_on_anomaly: bool,
}

impl Default for TEEContext {
    fn default() -> Self {
        Self {
            enclave_type: EnclaveType::None,
            attestation: AttestationMode::None,
            hard_reset_crypto: false,
            zeroize_on_anomaly: false,
        }
    }
}

impl TEEContext {
    pub fn is_valid(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnclaveType {
    None,
    SGX2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttestationMode {
    None,
    DCAP,
}
