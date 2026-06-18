use orchestrator::{
    hashtree::storage::provider::{HashTreeStorageProvider, HashTreeConfig},
    swarm::second_self::{SecondSelfOrchestrator, LLMProvider},
    skill::manager::SkillManager,
    skill::scheduler::SkillScheduler,
    skill::persistence::SkillPersistence,
    skill::registry::SkillRegistry,
    dark_relay::relay::DarkRelay,
    cli::skill_commands::SkillCommand,
};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    info!("🏛️ Cathedral ARKHE — Full Skill System");
    info!("═══════════════════════════════════════════════════════");

    let config = HashTreeConfig::default();
    let storage = Arc::new(HashTreeStorageProvider::new(config));
    let npub = std::env::var("NOSTR_NPUB").unwrap_or_else(|_| "npub1cathedral".to_string());
    let relays = vec![
        "wss://relay.nostr.band".to_string(),
        "wss://relay.damus.io".to_string(),
    ];

    let mut skill_mgr = SkillManager::new(storage.clone(), npub.clone());

    let mut orchestrator = SecondSelfOrchestrator::new(
        storage.clone(),
        npub.clone(),
        LLMProvider::Mock,
        "CLAUDE.md",
        10,
        true,
    ).await?;

    let loaded = orchestrator.load_all_skills(&mut skill_mgr, None).await?;
    info!("✅ Skills carregadas: {:?}", loaded);

    let mut registry = SkillRegistry::new(storage.clone(), npub.clone(), relays);
    info!("✅ Skills prontas para registro");

    let dark_relay = Some(DarkRelay::new());

    let persistence = SkillPersistence::new(storage.clone(), npub.clone());
    let mut scheduler = SkillScheduler::new(
        Arc::new(orchestrator.orchestrator.clone()),
        Arc::new(tokio::sync::Mutex::new(skill_mgr.clone())),
        persistence,
        dark_relay.clone(),
    );

    scheduler.load_and_start().await?;

    let schedules = scheduler.list_schedules();
    if !schedules.iter().any(|s| s.skill_name == "improve-codebase-architecture") {
        scheduler.add_schedule("improve-codebase-architecture", "0 0 * * 0").await?;
        info!("📅 Skill 'improve-codebase-architecture' agendada semanalmente");
    }

    let cmd = SkillCommand::ListSchedules;
    let output = cmd.execute(&mut orchestrator, &mut skill_mgr, &mut scheduler, &mut registry, &dark_relay).await?;
    println!("{}\n", output);

    let cmd = SkillCommand::Run { skill_name: "grill-me".to_string() };
    let output = cmd.execute(&mut orchestrator, &mut skill_mgr, &mut scheduler, &mut registry, &dark_relay).await?;
    println!("{}\n", output);

    info!("═══════════════════════════════════════════════════════");
    info!("🎉 Sistema completo rodando!");

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
