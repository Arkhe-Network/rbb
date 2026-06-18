use orchestrator::{
    hashtree::storage::provider::{HashTreeStorageProvider, HashTreeConfig},
    swarm::second_self::{SecondSelfOrchestrator, LLMProvider},
    skill::manager::SkillManager,
    skill::background_scheduler::BackgroundSkillScheduler,
    skill::registry::SkillRegistry,
};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    info!("🏛️ Cathedral ARKHE — Background Skill + Registry Demo");
    info!("═══════════════════════════════════════════════════════");

    let config = HashTreeConfig::default();
    let storage = Arc::new(HashTreeStorageProvider::new(config));
    let npub = std::env::var("NOSTR_NPUB").unwrap_or_else(|_| "npub1cathedral".to_string());
    let relays = vec![
        "wss://relay.nostr.band".to_string(),
        "wss://relay.damus.io".to_string(),
    ];

    let mut skill_mgr = SkillManager::new(storage.clone(), npub.clone());

    let builtin = orchestrator::skill::builtin::register_all(&mut skill_mgr).await?;
    info!("✅ Skills built-in carregadas: {:?}", builtin);

    let mut orchestrator = SecondSelfOrchestrator::new(
        storage.clone(),
        npub.clone(),
        LLMProvider::Mock,
        "CLAUDE.md",
        10,
        true,
    ).await?;

    let orchestrator_arc = Arc::new(orchestrator.orchestrator.clone());
    let skill_mgr_arc = Arc::new(tokio::sync::Mutex::new(skill_mgr.clone()));

    let mut scheduler = BackgroundSkillScheduler::new(
        orchestrator_arc,
        skill_mgr_arc.clone(),
        storage.clone(),
        npub.clone(),
    );

    scheduler.add_schedule("improve-codebase-architecture", "*/5 * * * *").await?;
    info!("✅ Skill agendada: improve-codebase-architecture a cada 5 minutos");

    let mut registry = SkillRegistry::new(storage.clone(), npub.clone(), relays);

    info!("🔍 Buscando skill 'diagnose' do registro...");
    let fetch_result = registry.fetch_skill("diagnose").await;
    if let Some(skill) = fetch_result {
        info!("✅ Skill encontrada: {} (v{})", skill.name, skill.version);
        let _ = registry.import_skill("diagnose", &mut skill_mgr).await?;
        info!("✅ Skill importada localmente");
    }

    info!("🔄 Iniciando BackgroundSkillScheduler...");
    scheduler.start().await;

    info!("⏳ Aguardando execuções agendadas (30 segundos)...");
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    info!("═══════════════════════════════════════════════════════");
    info!("🎉 Demo concluída!");

    Ok(())
}
