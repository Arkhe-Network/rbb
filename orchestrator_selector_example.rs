//! Exemplo de como o orquestrador seleciona dinamicamente entre Oracle-Instant e Oracle-Thinking
//! com base na complexidade da tarefa.
//!
//! Selo: CATHEDRAL-ARKHE-v28.3-DYNAMIC-SELECTOR-2026-06-16


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskComplexity {
    Low,    // Respostas factuais, cálculos diretos, validações simples.
    Medium, // Análise leve, cruzamento de poucos dados.
    High,   // Planejamento longo, pesquisa profunda, código complexo, dilemas éticos.
}

/// Heurística simples para avaliar a complexidade da tarefa
pub fn evaluate_task_complexity(task_description: &str, required_tools: &[String]) -> TaskComplexity {
    let text = task_description.to_lowercase();

    // Indicadores de alta complexidade
    let high_complexity_keywords = ["analisar profundo", "arquitetura", "refatorar", "dilema", "estratégia de longo prazo", "árvore de pensamento"];
    if high_complexity_keywords.iter().any(|&kw| text.contains(kw)) || required_tools.len() > 3 {
        return TaskComplexity::High;
    }

    // Indicadores de média complexidade
    let medium_complexity_keywords = ["comparar", "pesquisar e resumir", "revisar"];
    if medium_complexity_keywords.iter().any(|&kw| text.contains(kw)) || required_tools.len() >= 2 {
        return TaskComplexity::Medium;
    }

    // Por padrão (e.g. "Qual o status do sistema?", "Formatar data")
    TaskComplexity::Low
}

pub struct TaskRouter;

impl TaskRouter {
    /// Roteia a tarefa para o perfil de Oracle adequado com base na complexidade avaliada.
    pub fn route_task(task_description: &str, required_tools: &[String]) -> &'static str {
        let complexity = evaluate_task_complexity(task_description, required_tools);

        match complexity {
            // Tarefas de baixa complexidade ou média complexidade se beneficiam de baixa latência e Evo-CoT (Instant)
            TaskComplexity::Low | TaskComplexity::Medium => {
                println!("🧠 [Router] Tarefa avaliada como {:?}. Redirecionando para Oracle-Instant...", complexity);
                "cathedral-oracle-instant-v28.3"
            },
            // Tarefas complexas exigem reflexão profunda e manutenção do rastro de raciocínio (Thinking)
            TaskComplexity::High => {
                println!("🧠 [Router] Tarefa avaliada como {:?}. Redirecionando para Oracle-Thinking...", complexity);
                "cathedral-oracle-thinking-v28.3"
            }
        }
    }
}

pub fn main() {
    let task_1 = "Qual é o saldo atual do endereço 0x123... ?";
    let tools_1 = vec!["web3_call".to_string()];

    let task_2 = "Analisar profundo os contratos do protocolo X e propor uma arquitetura segura de integração.";
    let tools_2 = vec!["code_analysis".to_string(), "web_search".to_string(), "cathedral_policy".to_string(), "read_file".to_string()];

    let profile_1 = TaskRouter::route_task(task_1, &tools_1);
    println!("-> Perfil selecionado para Task 1: {}\n", profile_1);

    let profile_2 = TaskRouter::route_task(task_2, &tools_2);
    println!("-> Perfil selecionado para Task 2: {}\n", profile_2);
}
