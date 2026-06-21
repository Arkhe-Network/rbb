use async_trait::async_trait;

pub struct TrinityCore;
impl TrinityCore {
    pub fn new() -> Self { Self }
    pub async fn get_consciousness(&self) -> crate::moe::ConsciousnessState {
        crate::moe::ConsciousnessState::Aware
    }
    pub async fn submit_code_snippet(&self, _code: &str) -> Result<(), String> {
        Ok(())
    }
}

pub struct NgramDraftModel;
impl NgramDraftModel {
    pub fn new() -> Self { Self }
}
#[async_trait]
impl crate::mtp::DraftModel for NgramDraftModel {
    async fn draft(&self, prefix: &[u32], num_tokens: usize) -> Result<Vec<Vec<u32>>, String> {
        Ok(vec![vec![prefix.last().copied().unwrap_or(0); num_tokens]])
    }
}

pub struct VerifierImpl;
impl VerifierImpl {
    pub fn new() -> Self { Self }
}
#[async_trait]
impl crate::mtp::Verifier for VerifierImpl {
    async fn verify(&self, draft: &[Vec<u32>]) -> Result<Vec<bool>, String> {
        Ok(draft.iter().map(|_| true).collect())
    }
}
