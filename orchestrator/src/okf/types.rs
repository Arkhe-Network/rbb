#[derive(Default)]
pub struct OkfMetadata {
    pub doc_type: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub author: String,
    pub version: Option<String>,
}
