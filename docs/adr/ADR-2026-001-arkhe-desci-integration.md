---
status: Accepted
date: 2026-07-01
deciders: Arquiteto-Chefe (IC6), ITSec, DPO
consulted: DeSciOS Maintainers, Chainlink Engineering
---

# ADR 2026-001 — Integração ARKHE × DeSciOS

## Context

O DeSciOS é uma plataforma desktop containerizada para ciência descentralizada, oferecendo:

- Ambiente Linux com ferramentas científicas (Jupyter, RStudio, QGIS, UGENE, Nextflow)
- Assistente de IA local com Ollama (command-r7b, granite3.2-vision)
- Plugin System extensível via YAML/JSON
- DeSci/Web3 stack (IPFS, Syncthing, FunDeSci)

**Problema identificado:** O DeSciOS atualmente não possui:

1. **Governança de plugins** — plugins podem executar código arbitrário sem validação
2. **PII masking no assistente** — dados sensíveis podem ser enviados para LLMs
3. **Rastreabilidade de workflows** — não há cadeia causal verificável
4. **Integração cross-chain** — publicações IPFS não notificam outros sistemas

**Oportunidade:** Integrar o DeSciOS ao monorepo ARKHE para adicionar governança, segurança e rastreabilidade.

## Decisão

**Integrar o DeSciOS ao ARKHE através de um crate dedicado `arkhe-desci`** com os seguintes componentes:

| Componente | Função | Invariantes ARKHE |
|------------|--------|-------------------|
| `PluginValidator` | Valida plugins antes da instalação | OWASP-003, CNT-002, OWASP-006 |
| `DeSciAssistantGuardrails` | PII masking e validação de consultas | OWASP-002, OWASP-010 |
| `ScientificWorkflowTrace` | Cadeia causal IC16 para workflows | ARC-TRC-001, ARC-TRC-002 |
| `DeSciPublisher` | Publicação IPFS + notificação CCIP | MCP-001, MCP-006 |

### Arquitetura de Integração

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ARKHE × DeSciOS                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  ARKHE MONOREPO (crates/arkhe-desci)                                │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │   │
│  │  │ Plugin      │ │ Assistant   │ │ Workflow    │ │ Publishing  │ │   │
│  │  │ Governance  │ │ Guardrails  │ │ Traceability│ │ (IPFS+CCIP) │ │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                      │                                      │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  DEPENDÊNCIAS ARKHE                                                 │   │
│  │  • arkhe-invariants (InvariantEngine, Invariant)                   │   │
│  │  • asi-governance (PiiMasker, PolicyEngine)                        │   │
│  │  • arkhe-causal (CausalTrace, CausalLink)                         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                      │                                      │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  DeSciOS                                                            │   │
│  │  • Plugin System (YAML/JSON)                                        │   │
│  │  • Ollama Assistant                                                │   │
│  │  • Nextflow / Jupyter Workflows                                     │   │
│  │  • IPFS + FunDeSci                                                 │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Consequences

### Positivas

1. **Segurança melhorada** — plugins validados contra invariantes OWASP-003, CNT-002
2. **Privacidade reforçada** — PII masking no assistente (OWASP-002)
3. **Rastreabilidade auditável** — workflows científicos com cadeia causal IC16
4. **Interoperabilidade** — publicações IPFS notificam sistemas via CCIP
5. **Reutilização** — componentes podem ser usados por outros crates ARKHE

### Negativas

1. **Complexidade adicional** — novo crate com dependências externas
2. **Manutenção** — necessidade de acompanhar mudanças no DeSciOS
3. **Dependências** — chainlink-ccip e ipfs-http-client são opt-in

### Mitigações

1. **Features opcionais** — `ipfs` e `chainlink` são feature-gated
2. **Abstrações** — traits `IpfsClient` e `CcipClient` permitem múltiplas implementações
3. **Testes** — cobertura de testes para todos os componentes

## Alternatives Considered

### Alternativa A: Integração via API REST (rejeitada)

DeSciOS chamaria ARKHE via API REST para validação.

- **Prós:** Menos acoplamento
- **Contras:** Latência, dependência de rede, sem verificação em tempo de build

### Alternativa B: Fork do DeSciOS (rejeitada)

Criar um fork do DeSciOS com as modificações embutidas.

- **Prós:** Controle total
- **Contras:** Manutenção do fork, divergência do upstream

### Alternativa C: Plugin específico para ARKHE no DeSciOS (rejeitada)

Criar um plugin no DeSciOS que chama o ARKHE.

- **Prós:** Mais flexível
- **Contras:** Plugin seria complexo, não compilado com o restante

**Decisão:** A alternativa escolhida (`arkhe-desci` crate) oferece o melhor equilíbrio entre integração, manutenibilidade e segurança.

## Implementation Plan

| Fase | Período | Entregável | Critério de Sucesso |
|------|---------|------------|---------------------|
| 1 | Q3 2026 | PluginValidator + testes | Plugins validados antes da instalação |
| 2 | Q3 2026 | AssistantGuardrails + testes | PII masking e rate limiting funcionais |
| 3 | Q4 2026 | WorkflowTraceability + testes | Cadeia causal IC16 gerada |
| 4 | Q4 2026 | Publishing + testes | Publicação IPFS com CCIP opcional |

## Security Considerations

| Vulnerabilidade | Mitigação | Status |
|-----------------|-----------|--------|
| `curl | sh` sem checksum | Download com verificação | ✅ Corrigido |
| Senha VNC hardcoded | `--build-arg` obrigatório | ✅ Corrigido |
| `--break-system-packages` | Substituído por `--user` | ✅ Corrigido |
| `sudo` sem restrição | Removido do usuário | ✅ Corrigido |
| VNC sem TLS | Adicionar TLS no noVNC | 📋 Pendente |

## Status

**Accepted** — A implementação do crate `arkhe-desci` foi aprovada e está em desenvolvimento.

## References

- [DeSciOS Repository](https://github.com/GizmoQuest/DeSciOS)
- [ARKHE v11.0 Ontology](https://arkhe.ai/ontology/v11/)
- [OWASP LLM Top 10](https://owasp.org/www-project-top-10-for-large-language-model-applications/)
- [NIST AI RMF 1.0](https://www.nist.gov/itl/ai-risk-management-framework)
- [Chainlink CCIP](https://chain.link/cross-chain)
