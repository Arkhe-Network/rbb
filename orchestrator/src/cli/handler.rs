use crate::cli::skill_commands::SkillCommand;
use crate::swarm::second_self::SecondSelfOrchestrator;
use crate::skill::manager::SkillManager;
use crate::skill::scheduler::SkillScheduler;
use crate::skill::registry::SkillRegistry;
use crate::dark_relay::relay::DarkRelay;
use tracing::{info, error};

pub struct CliHandler;

impl CliHandler {
    pub async fn run_interactive(
        orchestrator: &mut SecondSelfOrchestrator,
        skill_manager: &mut SkillManager,
        scheduler: &mut SkillScheduler,
        registry: &mut SkillRegistry,
        dark_relay: &Option<DarkRelay>,
    ) -> Result<(), String> {
        info!("🖥️ CLI de Skills iniciada. Digite /help para ajuda.");

        loop {
            use tokio::io::AsyncBufReadExt;
            let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
            let mut input = String::new();

            // Usamos tokio async read para nao bloquear
            if let Err(e) = stdin.read_line(&mut input).await {
                error!("Erro ao ler entrada: {}", e);
                break;
            }

            let line = input.trim();
            if line.is_empty() {
                continue;
            }

            if line == "/exit" || line == "/quit" || line == "exit" {
                info!("👋 Encerrando CLI");
                break;
            }

            match SkillCommand::parse(line) {
                Some(cmd) => {
                    match cmd.execute(orchestrator, skill_manager, scheduler, registry, dark_relay).await {
                        Ok(output) => println!("{}", output),
                        Err(e) => println!("❌ Erro: {}", e),
                    }
                }
                None => {
                    println!("❌ Comando desconhecido. Digite /help para ajuda.");
                }
            }
        }

        Ok(())
    }

    pub async fn process_command(
        command: &str,
        orchestrator: &mut SecondSelfOrchestrator,
        skill_manager: &mut SkillManager,
        scheduler: &mut SkillScheduler,
        registry: &mut SkillRegistry,
        dark_relay: &Option<DarkRelay>,
    ) -> Result<String, String> {
        match SkillCommand::parse(command) {
            Some(cmd) => cmd.execute(orchestrator, skill_manager, scheduler, registry, dark_relay).await,
            None => Err(format!("Comando desconhecido: {}", command)),
        }
    }
}
