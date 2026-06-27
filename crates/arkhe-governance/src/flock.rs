use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FlockConfig {
    pub flock_bin: Option<PathBuf>,
    pub hash_function: String,
    pub steps: u64,
}

impl Default for FlockConfig {
    fn default() -> Self {
        Self {
            flock_bin: None,
            hash_function: "blake3".into(),
            steps: 256,
        }
    }
}
