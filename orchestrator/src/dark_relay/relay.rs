#[derive(Clone)]
pub struct DarkRelay;

impl DarkRelay {
    pub fn new() -> Self {
        Self
    }
    pub async fn send_private(&self, target: &str, msg: &str) -> Result<(), String> {
        Ok(())
    }
}
