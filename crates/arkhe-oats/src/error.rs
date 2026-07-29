#[derive(Debug)]
pub struct OatsError;

impl std::fmt::Display for OatsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OatsError")
    }
}
impl std::error::Error for OatsError {}
