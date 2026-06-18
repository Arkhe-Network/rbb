use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use crate::ast::{Language, Node};

#[async_trait]
pub trait Parser: Send + Sync {
    fn language(&self) -> Language;
    async fn parse(&self, code: &str) -> Result<Node>;
    async fn parse_file(&self, path: &str) -> Result<Node> {
        let code = tokio::fs::read_to_string(path).await?;
        self.parse(&code).await
    }
}

pub struct ParserFactory {
    parsers: Vec<Arc<dyn Parser>>,
}

impl ParserFactory {
    pub fn new() -> Self {
        let mut factory = Self { parsers: Vec::new() };
        factory.register_parser(Arc::new(rust::RustParser::new()));
        factory
    }

    pub fn register_parser(&mut self, parser: Arc<dyn Parser>) {
        self.parsers.push(parser);
    }

    pub fn get_parser(&self, language: Language) -> Option<Arc<dyn Parser>> {
        self.parsers.iter().find(|p| p.language() == language).cloned()
    }

    pub fn detect_language(&self, code: &str) -> Option<Language> {
        // Heurísticas simples
        if code.contains("fn ") && code.contains("->") && code.contains("let ") {
            return Some(Language::Rust);
        }
        None
    }

    pub async fn parse_auto(&self, code: &str) -> Result<(Language, Node)> {
        let lang = self.detect_language(code).ok_or_else(|| anyhow::anyhow!("Could not detect language"))?;
        let parser = self.get_parser(lang.clone()).ok_or_else(|| anyhow::anyhow!("No parser for {:?}", lang))?;
        let node = parser.parse(code).await?;
        Ok((lang, node))
    }
}

pub mod rust;
