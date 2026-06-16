# Fase 7: Preparação de Infraestrutura para RL Assíncrono (ASystem-like)

A arquitetura Ling & Ring 2.6 descreve o ASystem (Asynchronous Actor-Critic System) para separar o custo da inferência pesada das atualizações de política do RL, otimizando o treinamento de agentes via PPO. No Cathedral ARKHE, preparamos o terreno com as seguintes adaptações:

## 1. Arquitetura de Filas Desacopladas
Para permitir RL assíncrono, a geração de rollouts (ação e pensamento do agente) deve ocorrer de forma desvinculada da atualização do modelo (crítico e política).
- **Inference Node (Actor)**: Os instâncias `Oracle-Instant` e `Oracle-Thinking` produzem trajetórias em massa.
- **Rollout Buffer**: Usaremos o Redis Streams existente (`acp_event_bus.rs`) para canalizar as trajetórias (Estado, Ação, LogProb, Recompensa).
- **Training Node (Learner)**: Um serviço separado que consome do Redis Streams, avalia os rewards e calcula as atualizações de gradiente.

## 2. Telemetry & Reward Collection
O `agent_loop.rs` precisa estampar em cada tick de memória ou fim de execução o "Reward Signal".
No `AgentResult`, introduzir a coleta de recompensas (e.g., feedback do usuário, asserts de segurança, execução de código com sucesso):
```rust
pub struct RolloutRecord {
    pub task_id: String,
    pub step: usize,
    pub state_hash: String,
    pub action_taken: String,
    pub log_prob: f32, // Requer backend LLM adaptado para exportar logprobs (já ativado por flag no vLLM)
    pub reward: f32,
    pub reasoning_compressed: bool,
}
```

## 3. Integração com vLLM
O `Dockerfile.llm` preparado na Fase 4 suporta as configurações de longo contexto e processamento de alto throughput exigidos pela fase de rollouts.
Para a Fase 7, habilitaremos no vLLM a flag `--enable-log-probs` e ajustaremos o cliente `llm_client` no orquestrador para capturar esse dado junto com as ações.

## 4. O "Critic" / Juiz
O `DebateEngine` atual e o `Guardian` podem ser promovidos para atuar como o "Reward Model" / Critic off-line, atribuindo scores aos rollouts depositados no Redis antes de serem consumidos pelo Learner.

Isso isolará os custos computacionais de inferência da máquina de estado de PPO, tornando o Cathedral ARKHE capaz de se auto-aprimorar passivamente ("Native Agentic Training").
