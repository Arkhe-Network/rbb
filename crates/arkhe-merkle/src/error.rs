#[derive(Debug)]
pub struct MerkleError;

impl std::fmt::Display for MerkleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MerkleError")
    }
}
impl std::error::Error for MerkleError {}
