// src/cathedral_pqc.rs — Implementação completa PQC
use slh_dsa::*;
use kyber::*;
use falcon::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct QuantumCertificate {
    pub id: String,
    pub agent_id: String,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
    pub issuer: String,
    pub valid_from: u64,
    pub valid_until: u64,
    pub extensions: HashMap<String, Vec<u8>>,
    pub algorithm: String, // "SLH-DSA-256", "Falcon-1024"
}

impl QuantumCertificate {
    pub fn verify(&self, ca_public_key: &[u8]) -> bool {
        // Verifica a assinatura com SLH-DSA ou Falcon
        let mut verifier = SLHDSA::new(SecurityLevel::SLH_DSA_256);
        let data = self.to_verify_data();
        verifier.verify(&data, &self.signature, ca_public_key)
    }

    pub fn to_verify_data(&self) -> Vec<u8> {
        // Dados que foram assinados (exclui a assinatura)
        let mut data = Vec::new();
        data.extend_from_slice(self.id.as_bytes());
        data.extend_from_slice(self.agent_id.as_bytes());
        data.extend_from_slice(&self.public_key);
        data.extend_from_slice(self.issuer.as_bytes());
        data.extend_from_slice(&self.valid_from.to_be_bytes());
        data.extend_from_slice(&self.valid_until.to_be_bytes());
        data
    }
}

// JNI bindings para PQC
#[no_mangle]
pub extern "C" fn Java_cathedral_pqc_PQC_nativeGenerateSLHDSAPair(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    level: jint
) -> jbyteArray {
    let manager = unsafe { &*(handle as *const PQCManager) };
    let (public, private) = manager.generate_slh_dsa_keypair();

    // Combina public + private em um único byte array
    let mut result = Vec::with_capacity(public.len() + private.len());
    result.extend_from_slice(&public);
    result.extend_from_slice(&private);

    env.byte_array_from_slice(&result).unwrap()
}

#[no_mangle]
pub extern "C" fn Java_cathedral_pqc_PQC_nativeSignSLHDSA(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    message: jbyteArray,
    private_key: jbyteArray
) -> jbyteArray {
    let manager = unsafe { &*(handle as *const PQCManager) };
    let msg = env.convert_byte_array(message).unwrap();
    let priv_key = env.convert_byte_array(private_key).unwrap();

    let signature = manager.sign_slh_dsa(&msg, &priv_key).unwrap();
    env.byte_array_from_slice(&signature).unwrap()
}

#[no_mangle]
pub extern "C" fn Java_cathedral_pqc_PQC_nativeGenerateQSC(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    agent_id: jstring,
    public_key: jbyteArray,
    ca_private_key: jbyteArray
) -> jbyteArray {
    let manager = unsafe { &*(handle as *const PQCManager) };
    let agent = env.get_string(agent_id).unwrap().to_str().unwrap().to_string();
    let pub_key = env.convert_byte_array(public_key).unwrap();
    let ca_priv = env.convert_byte_array(ca_private_key).unwrap();

    let cert = QuantumCertificate {
        id: format!("qsc-{}", agent),
        agent_id: agent,
        public_key: pub_key.clone(),
        signature: manager.sign_slh_dsa(&pub_key, &ca_priv).unwrap(),
        issuer: "Cathedral CA".to_string(),
        valid_from: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        valid_until: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 31536000, // 1 ano
        extensions: HashMap::new(),
        algorithm: "SLH-DSA-256".to_string(),
    };

    let serialized = serde_json::to_vec(&cert).unwrap();
    env.byte_array_from_slice(&serialized).unwrap()
}
