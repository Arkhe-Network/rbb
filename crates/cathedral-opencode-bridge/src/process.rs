use std::process::{Command, Child};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

pub struct OpenCodeRuntime {
    process: Arc<Mutex<Option<Child>>>,
    port: u16,
}

impl OpenCodeRuntime {
    pub fn new(port: u16) -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            port,
        }
    }

    pub async fn start(&self, workspace: &str) -> Result<(), String> {
        let cmd = Command::new("bun")
            .arg("run")
            .arg("src/index.ts")
            .arg("--port")
            .arg(self.port.to_string())
            .arg("--workspace")
            .arg(workspace)
            .current_dir("./opencode-runtime")
            .spawn()
            .map_err(|e| format!("Failed to start OpenCode runtime: {}", e))?;

        *self.process.lock().await = Some(cmd);
        info!("OpenCode runtime started on port {}", self.port);

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        if let Some(mut child) = self.process.lock().await.take() {
            child.kill().map_err(|e| format!("Failed to stop OpenCode: {}", e))?;
            child.wait().map_err(|e| format!("Failed to wait for OpenCode: {}", e))?;
            info!("OpenCode runtime stopped");
        }
        Ok(())
    }
}
