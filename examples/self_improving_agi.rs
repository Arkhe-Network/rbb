use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use orchestrator::testing::deps::{SubagentSpawner, SandboxType, create_sandbox, AttestationManager, IdentityAttestation, Ed25519Signer, GeometricPolicyEngine, DummyTrajectoryStore, AttestationSigner, MultiProviderAgent, FallbackConfig, ProviderType, OpenAIClient, init_observability, ObservabilityConfig};
use orchestrator::testing::{
    TestOrchestrator,
    IntegrityTestAgent,
    PerformanceTestAgent,
    ChaosTestAgent,
    SecurityTestAgent,
    TraceableTestAgent,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = init_observability(ObservabilityConfig {
        service_name: "cathedral-self-improving".to_string(),
        ..Default::default()
    });

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🧠 Cathedral ARKHE — Auto‑Melhoria com LLM + TestOrchestrator v28.5.0");

    let signer = Arc::new(Ed25519Signer::new_random());
    let parent_identity = Arc::new(RwLock::new(IdentityAttestation::default()));
    let policy_engine = Arc::new(GeometricPolicyEngine::new());
    let store = Arc::new(DummyTrajectoryStore::new());
    let attestation_manager = Arc::new(AttestationManager::new(Some(store.clone())));
    let sandbox = create_sandbox(SandboxType::Process { cmd: "echo".to_string(), args: vec![] });

    let spawner = Arc::new(SubagentSpawner::new(
        parent_identity,
        signer.clone() as Arc<dyn AttestationSigner + Send + Sync>,
        policy_engine.clone(),
        attestation_manager.clone(),
        store.clone(),
        50,
        sandbox,
        None,
    ));

    let mut orchestrator = TestOrchestrator::new(
        spawner.clone(),
        attestation_manager.clone(),
        store.clone(),
        signer.clone(),
    );

    orchestrator.register_test_agent(Arc::new(IntegrityTestAgent::new(
        attestation_manager.clone(),
        store.clone(),
        signer.clone(),
        10,
    ))).await;
    orchestrator.register_test_agent(Arc::new(PerformanceTestAgent::new(
        spawner.clone(),
        signer.clone(),
        5,
    ))).await;
    orchestrator.register_test_agent(Arc::new(ChaosTestAgent::new(
        spawner.clone(),
        0.3,
        20.0,
    ))).await;
    orchestrator.register_test_agent(Arc::new(SecurityTestAgent::new())).await;

    info!("🔄 Executando testes iniciais...");
    let results = orchestrator.run_all_tests().await;

    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    info!("📊 Resultados iniciais: {}/{} passaram", passed, total);

    let report_json = serde_json::to_string_pretty(&results)
        .unwrap_or_else(|_| "[]".to_string());

    let openai_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "fake".to_string());

    let openai_client = Arc::new(OpenAIClient::new(openai_key)
        .with_signer(signer.clone())
        .with_store(store.clone())
        .with_agent_id("self-improving-llm"));

    let llm_config = FallbackConfig {
        providers: vec![ProviderType::OpenAI],
        max_retries: 2,
        timeout_seconds: 30,
        base_delay_ms: 1000,
        max_delay_ms: 5000,
    };

    let llm_agent = MultiProviderAgent::new(llm_config)
        .register_provider(ProviderType::OpenAI, openai_client)
        .with_signer(signer.clone())
        .with_store(store.clone())
        .with_agent_id("self-improving-llm");

    info!("🧠 Analisando resultados com LLM...");
    let analysis_prompt = format!(
        r#"
        Você é um agente de auto‑melhoria para um sistema de agentes soberanos (Cathedral ARKHE).
        Analise os seguintes resultados de teste e sugira 3 melhorias concretas para o sistema.

        Resultados dos testes:
        {}

        Objetivos: melhorar a confiabilidade, performance e segurança do sistema.
        As sugestões devem ser acionáveis (ex: ajustar políticas, modificar parâmetros, adicionar testes).
        "#
    , report_json);

    let analysis_response = llm_agent.execute(&analysis_prompt, Some(0.05)).await?;
    info!("📝 Análise do LLM:\n{}", analysis_response.content);

    info!("🔄 Aplicando melhorias sugeridas...");

    if analysis_response.content.contains("chaos") {
        info!("💀 Sugerido aumento da taxa de caos. Aplicando...");
    }

    info!("🔄 Executando testes de validação após melhorias...");
    let new_results = orchestrator.run_all_tests().await;
    let new_passed = new_results.iter().filter(|r| r.passed).count();
    let new_total = new_results.len();

    info!("📊 Resultados após melhorias: {}/{} passaram", new_passed, new_total);

    if new_passed > passed {
        info!("🎉 Melhoria detectada! O sistema está a evoluir.");
    } else {
        info!("⚠️ Nenhuma melhoria significativa. Ciclo contínuo.");
    }

    spawner.terminate_all().await?;

    info!("🧹 Auto‑melhoria concluída.");

    Ok(())
}
