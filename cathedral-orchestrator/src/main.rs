pub mod llm_api;
pub mod orchestrator;
pub mod task;

use crate::orchestrator::NeuroSymbolicOrchestrator;
use crate::task::CognitiveTask;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), String> {
    println!("🏛️ Cathedral ARKHE v28 - Neuro-Symbolic Multi-Model Orchestrator Initialized");

    // Simulate PREFER_LOCAL_LLM=true environment variable
    let prefer_local_llm = true;
    let orchestrator = NeuroSymbolicOrchestrator::new(prefer_local_llm);

    // Scenario 1: Translate Multimodal Input
    let task1 = CognitiveTask::TranslateMultimodal("Sensory Input: Image of Low Performance Hub".to_string());
    let res1 = orchestrator.process_task(task1).await?;
    println!("Response 1: {}", res1);

    // Scenario 2: Human-AI Dialog
    let task2 = CognitiveTask::HumanDialog("Why did the agent stop recommending high-risk DeFi hubs?".to_string());
    let res2 = orchestrator.process_task(task2).await?;
    println!("Response 2: {}", res2);

    // Scenario 3: Symbolic Reasoning (e.g. checking constraints against Ontology v28)
    let task3 = CognitiveTask::SymbolicReasoning("Validate ConstraintViolation: LowPerformanceHub for 'DeFi-Yield-HighRisk' with acceptanceRate 0.35".to_string());
    let res3 = orchestrator.process_task(task3).await?;
    println!("Response 3: {}", res3);

    // Generate Provenance Record logic (Simulation)
    println!("Generating Provenance Record...");
    let timestamp = Utc::now();
    println!("Provenance Record: mutation triggered. Rationale: 'acceptanceRate < 0.4'. Timestamp: {}", timestamp.to_rfc3339());

    // Scenario 4: Execute Action (e.g. adjust parameter)
    let task4 = CognitiveTask::ExecuteAction("Disable hub 'DeFi-Yield-HighRisk'".to_string());
    let res4 = orchestrator.process_task(task4).await?;
    println!("Response 4: {}", res4);

    Ok(())
}
