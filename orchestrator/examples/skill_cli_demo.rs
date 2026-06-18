use orchestrator::{
    hashtree::storage::provider::{HashTreeStorageProvider, HashTreeConfig},
    swarm::second_self::{SecondSelfOrchestrator, LLMProvider},
    skill::manager::SkillManager,
    skill::scheduler::SkillScheduler,
    skill::persistence::SkillPersistence,
    skill::registry::SkillRegistry,
    skill::notifier::SkillNotifier,
    cli::handler::CliHandler,
    dark_relay::relay::DarkRelay,
};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    info!("🏛️ Cathedral ARKHE — Skill CLI Demo");
    info!("═══════════════════════════════════════════════════════");

    let config = HashTreeConfig::default();
    let storage = Arc::new(HashTreeStorageProvider::new(config));
    let npub = std::env::var("NOSTR_NPUB").unwrap_or_else(|_| "npub1cathedral".to_string());
    let relays = vec![
        "wss://relay.nostr.band".to_string(),
        "wss://relay.damus.io".to_string(),
    ];

    let mut skill_mgr = SkillManager::new(storage.clone(), npub.clone());

    let _ = orchestrator::skill::builtin::register_all(&mut skill_mgr).await?;
    info!("✅ Skills built-in carregadas");

    let mut orchestrator = SecondSelfOrchestrator::new(
        storage.clone(),
        npub.clone(),
        LLMProvider::Mock,
        "CLAUDE.md",
        10,
        true,
    ).await?;

    let dark_relay = Some(DarkRelay::new());

    let notifier = SkillNotifier::new(dark_relay.clone(), npub.clone());

    let persistence = SkillPersistence::new(storage.clone(), npub.clone());
    let skill_mgr_arc = Arc::new(tokio::sync::Mutex::new(skill_mgr.clone()));
    let mut scheduler = SkillScheduler::new(
        Arc::new(orchestrator.orchestrator.clone()),
        skill_mgr_arc,
        persistence,
        dark_relay.clone(),
    );
    scheduler.load_and_start().await?;

    let mut registry = SkillRegistry::new(storage.clone(), npub.clone(), relays);

    info!("🔍 Executando auto-descoberta inicial...");
    let discovered = orchestrator.discover_and_import_skills(
        &mut registry,
        &mut skill_mgr,
        None,
    ).await?;
    info!("✅ Skills descobertas: {:?}", discovered);

    info!("🖥️ CLI iniciada. Digite /help para ajuda.");
    CliHandler::run_interactive(
        &mut orchestrator,
        &mut skill_mgr,
        &mut scheduler,
        &mut registry,
        &dark_relay,
    ).await?;

    info!("👋 Encerrando...");
    Ok(())
}
