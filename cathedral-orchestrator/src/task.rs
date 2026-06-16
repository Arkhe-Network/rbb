#[derive(Debug, Clone)]
pub enum CognitiveTask {
    TranslateMultimodal(String),
    HumanDialog(String),
    SymbolicReasoning(String),
    ExecuteAction(String),
}
