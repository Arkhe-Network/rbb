use orchestrator::{
    hashtree::storage::provider::{HashTreeStorageProvider, HashTreeConfig},
    swarm::second_self::{SecondSelfOrchestrator, LLMProvider},
    skill::manager::SkillManager,
    skill::registry::SkillRegistry,
};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    info!("🏛️ Cathedral ARKHE — Skills Demo");
    info!("═══════════════════════════════════════════════════════");

    let config = HashTreeConfig::default();
    let storage = Arc::new(HashTreeStorageProvider::new(config));
    let npub = std::env::var("NOSTR_NPUB").unwrap_or_else(|_| "npub1cathedral".to_string());
    let relays = vec![
        "wss://relay.nostr.band".to_string(),
        "wss://relay.damus.io".to_string(),
    ];

    info!("✅ Storage configurado");

    let mut skill_mgr = SkillManager::new(storage.clone(), npub.clone());

    let mut orchestrator = SecondSelfOrchestrator::new(
        storage.clone(),
        npub.clone(),
        LLMProvider::Mock,
        "CLAUDE.md",
        10,
        true,
    ).await?;

    info!("✅ Orchestrator inicializado");

    info!("📥 Carregando skills...");
    let loaded = orchestrator.load_all_skills(
        &mut skill_mgr,
        Some("./skills"),
    ).await?;

    info!("✅ Skills carregadas: {:?}", loaded);

    info!("📡 Publicando skills no registro...");
    let mut registry = SkillRegistry::new(storage.clone(), npub.clone(), relays);
    for skill_name in &loaded {
        let skill = skill_mgr.load_skill(skill_name).await.unwrap().clone();
        let _ = registry.publish_skill(&skill).await;
    }
    info!("✅ Skills publicadas via Nostr");

    info!("⚡ Executando skill 'grill-me'...");
    let result = orchestrator.execute_skill(&mut skill_mgr, "grill-me").await?;
    info!("✅ Skill executada: {} agentes, {} steps", result.agent_count, result.total_steps);

    info!("⚡ Verificando triggers de model-invoked...");
    let input = "We need to write tests for the new feature using TDD";
    let applied = orchestrator.apply_model_skills(input, &mut skill_mgr).await?;
    info!("✅ Skills aplicadas automaticamente: {:?}", applied);

    info!("📦 Exportando skill 'tdd' como OKF...");
    let okf_bundle = skill_mgr.export_as_okf("tdd").await?;
    info!("✅ OKF exportado: {} documentos", okf_bundle.documents.len());

    info!("═══════════════════════════════════════════════════════");
    info!("🎉 Skills demo concluída com sucesso!");

    Ok(())
}
