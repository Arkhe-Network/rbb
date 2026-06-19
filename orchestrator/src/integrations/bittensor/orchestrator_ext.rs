use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use crate::swarm::second_self::SecondSelfOrchestrator;
use crate::integrations::bittensor::{
    BittensorClient, BittensorConfig,
    sn96_verathos::VerathosClient,
    sn64_chutes::ChutesClient,
    sn60_bitsec::{BitsecClient, BitsecVulnerability},
    sn61_redteam::RedTeamClient,
    sn1_apex::{ApexClient, ApexSolutionResult},
    sn62_ridges::RidgesClient,
    sn31_recall::RecallClient,
    sn4_targon::TargonClient,
    stubs::{SecurityAnalysisReport, FastBrain, WormGraphIndexer, Vulnerability, Severity},
};

impl SecondSelfOrchestrator {
    pub async fn security_analysis_pipeline(
        &mut self,
        code: &str,
        language: &str,
    ) -> Result<SecurityAnalysisReport> {
        let bittensor = Arc::new(BittensorClient::new(BittensorConfig::default())?);

        // 1. Análise de código com SN60 (Bitsec)
        let bitsec = BitsecClient::new(bittensor.clone());
        let bitsec_result = bitsec.analyze_code(code, language, true).await?;

        // 2. Testes de penetração com SN61 (RedTeam) - se for código web/contrato
        let mut redteam_findings = Vec::new();
        if language == "javascript" || language == "rust" {
            let redteam = RedTeamClient::new(bittensor.clone());
            // Simula um alvo (para POC)
            let redteam_result = redteam.run_pentest("localhost:8080", "web", false).await?;
            redteam_findings = redteam_result.findings;
        }

        // 3. Correção de código com SN62 (Ridges)
        let ridges = RidgesClient::new(bittensor.clone());
        let mut fixes = Vec::new();
        for vuln in &bitsec_result.vulnerabilities {
            if vuln.severity == "critical" || vuln.severity == "high" {
                let fix = ridges.fix_code(code, language, &vuln.description).await?;
                fixes.push((self.convert_to_cathedral_vuln(vuln), fix.fixed_code));
            }
        }

        // 4. Armazena resultados no WormGraph + SN31 (Recall)
        let recall = RecallClient::new(bittensor.clone());
        let mut wormgraph_indexer = WormGraphIndexer;
        for vuln in &bitsec_result.vulnerabilities {
            // Converte para o formato da Cathedral
            let cathedral_vuln = self.convert_to_cathedral_vuln(vuln);
            wormgraph_indexer.index_with_recall(&cathedral_vuln, "bittensor-sn60").await?;
        }

        // 5. Gera provas ZK para vulnerabilidades críticas usando SN4 (Targon)
        let mut zk_proofs = Vec::new();
        for (vuln, _) in &fixes {
            if let Severity::Critical = vuln.severity {
                let targon = TargonClient::new(bittensor.clone());
                let proof = targon.generate_cathedral_proof(
                    vuln,
                    code,
                ).await?;
                zk_proofs.push(proof);
            }
        }

        // 6. Report final
        Ok(SecurityAnalysisReport {
            vulnerabilities: bitsec_result,
            pentest_findings: redteam_findings,
            suggested_fixes: fixes,
            zk_proofs,
        })
    }

    pub async fn run_agent_on_apex(
        &mut self,
        challenge_type: Option<&str>,
    ) -> Result<Vec<ApexSolutionResult>> {
        let bittensor = Arc::new(BittensorClient::new(BittensorConfig::default())?);
        let apex = ApexClient::new(bittensor);
        let fast_brain = FastBrain;

        // 1. Obtém desafios
        let challenges = apex.get_challenges(challenge_type).await?;

        // 2. Para cada desafio, o agent (FastBrain) resolve
        let mut results = Vec::new();
        for challenge in challenges {
            info!("🧠 Agent atacando desafio: {}", challenge.title);

            // Pula desafios muito fáceis ou muito difíceis
            if challenge.difficulty == "easy" || challenge.difficulty == "hard" {
                continue;
            }

            // Usa o FastBrain (que usa SN96) para gerar solução
            let solution = fast_brain
                .infer_with_verathos(
                    &format!("Resolva o desafio: {}", challenge.description),
                    false,
                )
                .await?;

            // Submete a solução
            let result = apex.submit_solution(&challenge.id, &solution).await?;
            results.push(result);
        }

        Ok(results)
    }

    pub fn convert_to_cathedral_vuln(&self, v: &BitsecVulnerability) -> Vulnerability {
        Vulnerability {
            id: v.id.clone(),
            title: v.title.clone(),
            description: v.description.clone(),
            severity: match v.severity.as_str() {
                "critical" => Severity::Critical,
                "high" => Severity::High,
                "medium" => Severity::Medium,
                "low" => Severity::Low,
                _ => Severity::Info,
            },
            location: v.location.clone(),
            cwe_id: v.cwe_id.clone(),
            verified: false,
            exploitation_details: None,
            remediation: v.remediation.clone(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}
