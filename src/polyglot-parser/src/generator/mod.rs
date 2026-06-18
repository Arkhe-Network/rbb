use anyhow::Result;
use crate::ast::{Node, Language};

pub trait CodeGenerator {
    fn generate(&self, node: &Node) -> Result<String>;
    fn language(&self) -> Language;
}

pub struct GeneratorFactory {
    generators: Vec<Box<dyn CodeGenerator>>,
}

impl GeneratorFactory {
    pub fn new() -> Self {
        let mut factory = Self { generators: Vec::new() };
        factory
    }

    pub fn register_generator(&mut self, generator: Box<dyn CodeGenerator>) {
        self.generators.push(generator);
    }

    pub fn get_generator(&self, language: Language) -> Option<&dyn CodeGenerator> {
        self.generators.iter().find(|g| g.language() == language).map(|g| g.as_ref())
    }

    pub fn generate(&self, node: &Node, target_language: Language) -> Result<String> {
        let generator = self.get_generator(target_language.clone())
            .ok_or_else(|| anyhow::anyhow!("No generator for {:?}", target_language))?;
        generator.generate(node)
    }
}
