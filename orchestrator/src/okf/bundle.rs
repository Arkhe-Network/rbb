pub struct OkfBundle {
    pub documents: Vec<String>,
}

impl OkfBundle {
    pub fn new(_name: &str, _desc: &str) -> Self {
        Self { documents: vec![] }
    }
    pub fn add_document(&mut self, path: String, _meta: crate::okf::types::OkfMetadata, _content: String) {
        self.documents.push(path);
    }
    pub fn add_log_entry(&mut self, _action: &str, _msg: &str) {
    }
}
