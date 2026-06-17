pub mod llm_api;
pub mod orchestrator;
pub mod task;

use crate::orchestrator::NeuroSymbolicOrchestrator;
use crate::task::CognitiveTask;
use chrono::Utc;
use std::sync::Arc;
use tracing::error;

use ::orchestrator::attestation::manager::{AttestationManager, CathedralComputeProvider};
use ::orchestrator::identity_attestation::DummyIdentityProvider;
use ::orchestrator::voice::VoiceCore;
use ::orchestrator::mcp::server::start_mcp_server;

#[tokio::main]
async fn main() -> Result<(), String> {
    println!("🏛️ Cathedral ARKHE v28 - Neuro-Symbolic Multi-Model Orchestrator Initialized");

    let mcp_enabled = std::env::var("ENABLE_MCP_SERVER")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    if mcp_enabled {
        let mcp_port = std::env::var("MCP_PORT")
            .unwrap_or_else(|_| "3032".to_string())
            .parse::<u16>()
            .unwrap_or(3032);

        let attestation_manager = Arc::new(AttestationManager::new());
        let identity_provider = Arc::new(DummyIdentityProvider::new());
        let execution_provider = Arc::new(CathedralComputeProvider::new());
        let voice_core = Arc::new(VoiceCore::new());

        let attestation_manager_clone = attestation_manager.clone();
        let identity_provider_clone = identity_provider.clone();
        let execution_provider_clone = execution_provider.clone();
        let voice_core_clone = Some(voice_core.clone());

        tokio::spawn(async move {
            if let Err(e) = start_mcp_server(
                attestation_manager_clone,
                identity_provider_clone,
                execution_provider_clone,
                None, // architect_verifier
                voice_core_clone,
                mcp_port,
            )
            .await
            {
                error!("❌ MCP Server falhou: {}", e);
            }
        });

        println!("🧠 MCP Server iniciado na porta {}", mcp_port);
    }

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
