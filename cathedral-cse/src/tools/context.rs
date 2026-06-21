pub struct ToolContext {
    pub workspace_dir: std::path::PathBuf,
}
impl ToolContext {
    pub fn new(dir: std::path::PathBuf) -> Self {
        Self { workspace_dir: dir }
    }
}
pub struct SessionManager {
    sessions: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, SessionData>>>,
}

#[derive(Clone)]
pub struct SessionData {
    pub history: Vec<crate::agent::AgentMessage>,
    pub tool_context: std::sync::Arc<ToolContext>,
}

impl SessionManager {
    pub fn new(_size: usize) -> Self {
        Self { sessions: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())) }
    }
    pub async fn create_session(&self, id: &str, tool_context: std::sync::Arc<ToolContext>) {
        self.sessions.lock().await.insert(id.to_string(), SessionData { history: vec![], tool_context });
    }
    pub async fn get_session(&self, id: &str) -> Option<SessionData> {
        self.sessions.lock().await.get(id).cloned()
    }
    pub async fn append_message(&self, id: &str, msg: crate::agent::AgentMessage) {
        if let Some(s) = self.sessions.lock().await.get_mut(id) {
            s.history.push(msg);
        }
    }
}
