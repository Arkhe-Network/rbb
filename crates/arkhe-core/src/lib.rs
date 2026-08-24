pub mod string_safe;

#[derive(Debug)]
pub struct ArkheError;

impl std::fmt::Display for ArkheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArkheError")
    }
}

impl std::error::Error for ArkheError {}

pub mod delta;
pub mod hash;
pub mod invariants;
pub mod types;

#[cfg(test)]
pub mod tests;
