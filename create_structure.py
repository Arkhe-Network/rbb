import os
import re

tree_str = """
├── 📄 README.md                                     # Documentação principal do projeto
├── 📄 LICENSE                                       # Licença MIT/Apache-2.0
├── 📄 CONTRIBUTING.md                               # Guia para contribuidores
├── 📄 CHANGELOG.md                                  # Histórico de versões
├── 📄 SECURITY.md                                   # Política de segurança e divulgação de vulnerabilidades
├── 📄 CODE_OF_CONDUCT.md                            # Código de conduta para colaboradores
├── 📄 .gitignore                                    # Arquivos e pastas ignorados pelo Git
├── 📄 .gitattributes                                # Configurações de atributos do Git
├── 📄 .editorconfig                                 # Configuração de estilo de código para editores
├── 📄 .prettierrc                                   # Configuração do Prettier para formatação de código
├── 📄 .eslintrc.js                                  # Configuração do ESLint para linting de JavaScript/TypeScript
├── 📄 .dockerignore                                 # Arquivos ignorados pelo Docker
├── 📄 .nvmrc                                        # Versão do Node.js gerenciada pelo nvm
├── 📄 .env.example                                  # Exemplo de variáveis de ambiente
├── 📄 .cargo/config.toml.template                   # Template de configuração do Cargo para cross-compilation
│
├── 📂 .github/                                      # Configurações do GitHub (CI/CD, templates)
│   ├── 📂 workflows/
│   │   ├── 📄 ci.yml                                # Pipeline CI/CD principal (build, test, lint, security)
│   │   ├── 📄 release.yml                           # Pipeline de release (gera artefatos e instaladores)
│   │   ├── 📄 security.yml                          # Pipeline de segurança (SCA, SAST, dependency scanning)
│   │   └── 📄 qa.yml                                # Pipeline de QA (cobertura de testes, mutation testing)
│   ├── 📂 ISSUE_TEMPLATE/
│   │   ├── 📄 bug_report.md                         # Template para relatório de bugs
│   │   └── 📄 feature_request.md                    # Template para solicitação de funcionalidades
│   └── 📄 PULL_REQUEST_TEMPLATE.md                  # Template para Pull Requests
│
├── 📂 .husky/                                       # Git hooks (commitlint, pre-commit)
│   ├── 📄 commit-msg                                # Hook para validar mensagens de commit
│   └── 📄 pre-commit                                # Hook para executar linting e formatação antes do commit
│
├── 📂 docs/                                         # Documentação completa do sistema
│   ├── 📂 architecture/                             # Documentação arquitetural
│   │   ├── 📄 overview.md                           # Visão geral da arquitetura
│   │   ├── 📄 uast.md                               # Especificação da UAST (Unified Abstract Syntax Tree)
│   │   ├── 📄 codesign.md                           # Documentação do co-design HW/SW
│   │   ├── 📄 marketplace.md                        # Arquitetura do marketplace de agentes
│   │   └── 📄 security.md                           # Modelo de segurança e identidade soberana
│   ├── 📂 api/                                      # Documentação das APIs
│   │   ├── 📄 openapi.yaml                          # Especificação OpenAPI 3.0 para APIs REST
│   │   └── 📄 grpc.proto                            # Definições de serviços gRPC
│   ├── 📂 guides/                                   # Guias para usuários e desenvolvedores
│   │   ├── 📄 getting-started.md                    # Guia de primeiros passos
│   │   ├── 📄 development.md                        # Guia para desenvolvimento
│   │   ├── 📄 deployment.md                         # Guia para deployment
│   │   └── 📄 troubleshooting.md                    # Guia de resolução de problemas
│   └── 📂 examples/                                 # Exemplos de projetos e casos de uso
│       └── 📂 demo-projects/
│           ├── 📂 chip-design/                      # Exemplo de design de chip
│           ├── 📂 firmware/                         # Exemplo de firmware embarcado
│           └── 📂 system-integration/               # Exemplo de integração de sistema
│
├── 📂 crates/                                       # Componentes Rust (Core do sistema)
│   │
│   ├── 📂 cathedral-core/                           # Modelos base e tipos compartilhados (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 did.rs                           # Estruturas para Identidade Descentralizada (DID)
│   │       ├── 📄 permission.rs                    # Definições de permissões e capabilities
│   │       ├── 📄 action.rs                        # Representação de ações e comandos
│   │       └── 📄 uast.rs                          # Tipos base para a UAST
│   │
│   ├── 📂 cathedral-identity/                       # 🆕 Módulo de identidade soberana (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 did.rs                           # Geração e validação de DIDs (W3C DID Core 1.0)
│   │       ├── 📄 ml_dsa.rs                         # ⬆️ ML-DSA-65 (NIST FIPS 204) com side-channel masking
│   │       ├── 📄 vc.rs                             # 🆕 W3C Verifiable Credentials 2.0 e Data Integrity Proof
│   │       ├── 📄 keypair.rs                        # Gerenciamento de pares de chaves
│   │       └── 📄 zk_proof.rs                       # Interface para provas de conhecimento zero (ZK)
│   │
│   ├── 📂 cathedral-wormgraph/                      # Ledger imutável para rastreabilidade (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 client.rs                        # Cliente para interagir com o ledger
│   │       ├── 📄 block.rs                         # Estrutura de blocos
│   │       ├── 📄 transaction.rs                   # Transações
│   │       ├── 📄 lineage.rs                       # Rastreamento de linhagem e proveniência
│   │       └── 📄 query.rs                         # Consultas ao grafo
│   │
│   ├── 📂 cathedral-zk/                             # Gateway para provas ZK (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 gateway.rs                       # Interface unificada para ZK
│   │       ├── 📄 proof.rs                         # Geração e verificação de provas
│   │       ├── 📄 compilation.rs                   # Compilação para circuitos ZK
│   │       └── 📄 equivalence.rs                   # Provas de equivalência de programas
│   │
│   ├── 📂 cathedral-uast/                           # Parser e manipulador de UAST (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 parser.rs                         # Parser baseado em tree-sitter
│   │       ├── 📄 converter.rs                      # Conversão para o formato UAST padronizado
│   │       ├── 📄 languages.rs                      # Suporte a múltiplas linguagens (Java, JS, Python, C++, Rust)
│   │       ├── 📄 semantic.rs                       # Análise semântica da UAST
│   │       ├── 📄 validator.rs                      # Validação de UAST
│   │       ├── 📄 cache.rs                         # Cache de UASTs
│   │       └── 📄 index.rs                         # Indexação para consultas rápidas
│   │
│   ├── 📂 cathedral-analysis/                       # 🆕 Motor de análise estática (YASA-Engine)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 uql.rs                           # 🆕 Unified Query Language (YASA-UQL)
│   │       ├── 📄 taint.rs                         # 🆕 Taint analysis para detecção de vulnerabilidades
│   │       ├── 📄 dataflow.rs                      # 🆕 Análise de fluxo de dados
│   │       ├── 📄 callgraph.rs                     # 🆕 Análise de grafo de chamadas de função
│   │       └── 📄 mcp.rs                           # 🆕 YASA-MCP bindings para integração com LLM
│   │
│   ├── 📂 cathedral-transpile/                      # Transpilação verificável (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 transpiler.rs                    # Motor de transpilação
│   │       ├── 📄 validator.rs                     # Validação de equivalência semântica
│   │       ├── 📄 llmlift.rs                       # Integração com LLM para sugestões de refatoração
│   │       └── 📄 equivalence.rs                   # Provas de equivalência entre códigos fonte
│   │
│   ├── 📂 cathedral-temporal/                       # Grafo temporal de versões (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 graph.rs                         # Estrutura do grafo
│   │       ├── 📄 node.rs                          # Nós do grafo (versões, artefatos)
│   │       ├── 📄 edge.rs                          # Arestas (dependências, transformações)
│   │       ├── 📄 index.rs                         # Índice para consultas
│   │       ├── 📄 rollback.rs                      # Mecanismo de rollback
│   │       ├── 📄 diff.rs                          # Comparação entre versões
│   │       └── 📄 lineage.rs                       # Rastreamento de linhagem
│   │
│   ├── 📂 cathedral-sandbox/                        # Sandbox de execução segura (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 execution.rs                     # Gerenciamento de execução
│   │       ├── 📄 policy.rs                        # Políticas de segurança
│   │       ├── 📄 backend.rs                       # Backends de execução (wasm, native)
│   │       └── 📄 security.rs                      # Medidas de segurança adicionais
│   │
│   ├── 📂 cathedral-tools/                          # Ferramentas do sistema (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 registry.rs                      # Registro de ferramentas
│   │       ├── 📄 filesystem.rs                    # Operações de sistema de arquivos
│   │       ├── 📄 bash.rs                          # Integração com Bash
│   │       ├── 📄 git.rs                           # Integração com Git
│   │       ├── 📄 search.rs                        # Busca em código-fonte
│   │       ├── 📄 web.rs                           # Cliente HTTP
│   │       ├── 📄 uast_tools.rs                    # Ferramentas específicas para UAST
│   │       ├── 📄 transpile_tools.rs               # Ferramentas para transpilação
│   │       └── 📄 temporal_tools.rs                # Ferramentas para o grafo temporal
│   │
│   ├── 📂 cathedral-permissions/                    # Sistema de permissões (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 permission.rs                    # Definição de permissões
│   │       ├── 📄 policy.rs                        # Políticas de acesso
│   │       └── 📄 evaluator.rs                     # Avaliador de políticas
│   │
│   ├── 📂 cathedral-extensibility/                  # Sistema de Skills e Plugins (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 mcp.rs                           # Implementação do MCP (Model Context Protocol)
│   │       ├── 📄 skills.rs                        # Gerenciamento de Skills
│   │       ├── 📄 plugins.rs                       # Gerenciamento de Plugins
│   │       ├── 📄 hooks.rs                         # Sistema de hooks para extensibilidade
│   │       └── 📄 config_agents.rs                 # Configuração de agentes
│   │
│   ├── 📂 cathedral-self-improve/                   # Mecanismo de auto-melhoria (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 insight.rs                       # Geração de insights a partir de dados
│   │       ├── 📄 ground_truth.rs                  # Verificação de ground truth
│   │       └── 📄 ab_testing.rs                    # Testes A/B para otimização
│   │
│   ├── 📂 cathedral-harness/                        # Harness para agentes (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 loop.rs                          # Loop principal do agente
│   │       ├── 📄 perception.rs                    # Módulo de percepção
│   │       ├── 📄 planning.rs                      # Módulo de planejamento
│   │       ├── 📄 action.rs                        # Módulo de ação
│   │       ├── 📄 reflection.rs                    # Módulo de reflexão
│   │       ├── 📄 context.rs                       # Gerenciamento de contexto
│   │       ├── 📄 compression.rs                   # Compressão de contexto
│   │       └── 📄 lineage.rs                       # Rastreamento de linhagem do agente
│   │
│   ├── 📂 cathedral-orchestrator/                   # Orquestração de agentes (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 orchestrator.rs                  # Orquestrador principal
│   │       ├── 📄 scheduler.rs                     # Agendador de tarefas
│   │       └── 📄 worker.rs                        # Worker para execução de tarefas
│   │
│   ├── 📂 cathedral-agents/                         # Agentes especializados (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 software_agent.rs                # Agente para engenharia de software
│   │       ├── 📄 hardware_agent.rs                # Agente para engenharia de hardware
│   │       ├── 📄 systems_agent.rs                 # Agente para engenharia de sistemas
│   │       └── 📄 quality_agent.rs                 # Agente para garantia de qualidade
│   │
│   ├── 📂 cathedral-codesign/                       # ⬆️ Co-design HW/SW expandido (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 engine.rs                        # Motor de co-design
│   │       ├── 📄 verifier.rs                      # Verificador cruzado HW/SW
│   │       ├── 📄 decomposition.rs                 # Decomposição de requisitos em HW e SW
│   │       ├── 📄 interface.rs                     # Definição de interface HW/SW
│   │       ├── 📄 hls.rs                           # 🆕 High-Level Synthesis (PandA-Bambu)
│   │       └── 📄 riscv_soc.rs                     # 🆕 RISC-V SoC generation (Chipyard)
│   │
│   ├── 📂 cathedral-marketplace/                    # Marketplace de agentes (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 registry.rs                      # 🆕 Registro de listings com governança
│   │       ├── 📄 listing.rs                       # Definição de listings (Skills, Plugins, Agentes)
│   │       ├── 📄 pricing.rs                       # Motor de precificação
│   │       └── 📄 reputation.rs                    # Sistema de reputação
│   │
│   ├── 📂 cathedral-prometheus/                     # Integração com Prometheus (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 client.rs                        # Cliente para API da Prometheus
│   │       ├── 📄 simulation.rs                    # Execução de simulações físicas
│   │       ├── 📄 design.rs                        # Design de hardware assistido
│   │       └── 📄 skill.rs                         # Skills que utilizam a Prometheus
│   │
│   ├── 📂 cathedral-bridge/                         # Bridge para runtimes externos (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 protocol.rs                      # Definição de protocolos de comunicação
│   │       ├── 📄 remix.rs                         # Bridge para o runtime Remix
│   │       └── 📄 opencode.rs                      # Bridge para o runtime OpenCode
│   │
│   ├── 📂 cathedral-kernel/                         # ⬆️ Microkernel expandido (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 lib.rs
│   │       ├── 📄 memory.rs                         # Gerenciamento de memória
│   │       ├── 📄 ipc.rs                            # ⬆️ Message-passing IPC (6 primitivas)【minix.rs】
│   │       ├── 📄 scheduler.rs                      # Escalonador de tarefas
│   │       ├── 📄 capability.rs                     # 🆕 Capability-based security【Sipahi/CambiOS】
│   │       ├── 📄 pmp.rs                            # 🆕 PMP isolation (RISC-V)【Sipahi】
│   │       ├── 📄 interrupts.rs                     # Gerenciamento de interrupções
│   │       └── 📄 syscall.rs                        # Chamadas de sistema
│   │
│   ├── 📂 cathedral-server/                         # Servidor HTTP (Rust)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       ├── 📄 main.rs
│   │       ├── 📄 lib.rs
│   │       └── 📂 api/
│   │           ├── 📄 mod.rs
│   │           ├── 📄 auth.rs                      # Endpoints de autenticação
│   │           ├── 📄 compile.rs                   # Endpoints de compilação
│   │           ├── 📄 transpile.rs                 # Endpoints de transpilação
│   │           ├── 📄 temporal.rs                  # Endpoints do grafo temporal
│   │           ├── 📄 sandbox.rs                   # Endpoints do sandbox
│   │           ├── 📄 agents.rs                    # Endpoints de agentes
│   │           ├── 📄 marketplace.rs               # Endpoints do marketplace
│   │           └── 📄 ledger.rs                    # Endpoints do ledger
│   │
│   └── 📂 cathedral-cli/                            # Interface de linha de comando (Rust)
│       ├── 📄 Cargo.toml
│       └── 📂 src/
│           ├── 📄 main.rs
│           └── 📂 commands/
│               ├── 📄 compile.rs                   # Comando de compilação
│               ├── 📄 transpile.rs                 # Comando de transpilação
│               ├── 📄 agent.rs                     # Comando para gerenciar agentes
│               ├── 📄 skill.rs                     # Comando para gerenciar skills
│               ├── 📄 plugin.rs                    # Comando para gerenciar plugins
│               ├── 📄 marketplace.rs               # Comando para interagir com o marketplace
│               └── 📄 ledger.rs                    # Comando para interagir com o ledger
│
├── 📂 packages/                                     # Pacotes compartilhados (TypeScript)
│   ├── 📂 trpc/                                     # 🆕 tRPC para APIs typesafe
│   │   ├── 📄 package.json
│   │   └── 📂 src/
│   │       └── 📄 index.ts
│   │
│   ├── 📂 ui/                                       # ⬆️ Expandido com Shadcn UI
│   │   ├── 📄 package.json
│   │   └── 📂 src/
│   │       └── 📂 components/
│   │           └── 📄 shadcn/                      # 🆕 Shadcn UI components
│   │
│   ├── 📂 typescript-config/                        # Configurações TypeScript compartilhadas
│   │   ├── 📄 package.json
│   │   └── 📄 base.json
│   │
│   └── 📂 eslint-config/                            # Configurações ESLint compartilhadas
│       ├── 📄 package.json
│       └── 📄 index.js
│
├── 📂 marketplace/                                  # 🆕 Serviços do Marketplace de Agentes
│   ├── 📂 registry/                                 # agentregistry core
│   │   ├── 📄 package.json
│   │   └── 📂 src/
│   │       ├── 📄 registry.ts                      # Registro centralizado de agentes e MCP servers
│   │       ├── 📄 cli.ts                           # CLI para gerenciamento do registro
│   │       └── 📂 web-ui/                          # Interface Web do registro
│   │           ├── 📄 index.html
│   │           └── 📂 src/
│   │               └── 📄 App.tsx
│   │
│   ├── 📂 discover/                                 # agent-discover MCP discovery
│   │   ├── 📄 package.json
│   │   └── 📂 src/
│   │       ├── 📄 discover.ts                      # Descoberta dinâmica de MCP servers
│   │       ├── 📄 proxy.ts                         # Proxy dinâmico para ativação de ferramentas
│   │       └── 📄 search.ts                        # Busca BM25 + semântica
│   │
│   └── 📂 gateway/                                  # MCP Gateway (enterprise-ready)
│       ├── 📄 package.json
│       └── 📂 src/
│           ├── 📄 gateway.ts                       # Gateway com OAuth e governança
│           └── 📄 auth.ts                          # Autenticação e autorização
│
├── 📂 hw/                                            # Hardware (RTL/Verilog/SystemVerilog/Chisel)
│   ├── 📂 rtl/                                      # Código RTL tradicional
│   │   ├── 📂 cpu/
│   │   │   └── 📄 core.sv                          # Núcleo do processador
│   │   ├── 📂 memory/
│   │   │   └── 📄 ram.sv                           # Módulo de memória RAM
│   │   └── 📂 peripherals/
│   │       └── 📄 uart.sv                          # Periférico UART
│   │
│   ├── 📂 chisel/                                   # 🆕 Chipyard (Chisel-based SoC)
│   │   ├── 📂 src/
│   │   │   ├── 📄 SoC.scala                        # Definição do System-on-Chip
│   │   │   ├── 📄 RocketConfig.scala               # Configuração do core Rocket
│   │   │   └── 📄 GemminiAccelerator.scala         # Acelerador Gemmini
│   │   ├── 📄 build.sbt                            # Configuração do SBT build
│   │   └── 📄 project/
│   │       └── 📄 plugins.sbt
│   │
│   ├── 📂 hls/                                      # 🆕 PandA-Bambu HLS
│   │   ├── 📂 examples/
│   │   │   └── 📄 matrix_mult.c                    # Exemplo de multiplicação de matrizes
│   │   ├── 📂 scripts/
│   │   │   └── 📄 run_hls.sh                       # Script para execução do HLS
│   │   └── 📄 README.md
│   │
│   ├── 📂 simulation/                               # Simulação com Verilator/Cocotb
│   │   ├── 📂 tests/
│   │   │   └── 📄 testbench.py                     # Testbench em Python (Cocotb)
│   │   └── 📂 models/
│   │       └── 📄 model.cpp                        # Modelos de simulação em C++
│   │
│   ├── 📂 verification/                             # Verificação formal
│   │   └── 📄 formal.sva                           # Asserções para verificação formal
│   │
│   └── 📂 specs/
│       └── 📄 architecture.md                       # Especificação da arquitetura de hardware
│
├── 📂 apps/                                         # Aplicações (TypeScript/React/React Native)
│   ├── 📂 web/                                      # Aplicação Web (Next.js)
│   │   ├── 📄 package.json
│   │   ├── 📄 tsconfig.json
│   │   ├── 📄 next.config.js
│   │   └── 📂 src/
│   │       ├── 📄 app.tsx
│   │       ├── 📂 app/
│   │       │   ├── 📂 api/                         # Rotas de API (Next.js App Router)
│   │       │   │   └── 📂 text-analysis/
│   │       │   │       └── 📄 route.ts
│   │       │   └── 📂 page.tsx                     # Página principal
│   │       └── 📂 components/
│   │           └── 📄 UASTViewer.tsx               # Componente para visualizar UAST
│   │
│   ├── 📂 native/                                   # Aplicação Nativa (Tauri v2)
│   │   ├── 📄 package.json
│   │   ├── 📄 tsconfig.json
│   │   ├── 📄 tauri.conf.json                       # Configuração do Tauri
│   │   ├── 📄 src-tauri/                           # Backend Rust do Tauri
│   │   │   ├── 📄 Cargo.toml
│   │   │   └── 📂 src/
│   │   │       ├── 📄 main.rs
│   │   │       └── 📄 lib.rs
│   │   └── 📂 src/                                 # Frontend React do Tauri
│   │       ├── 📄 main.tsx
│   │       └── 📂 components/
│   │           └── 📄 App.tsx
│   │
│   └── 📂 mobile/                                   # Aplicação Móvel (React Native/Expo)
│       ├── 📄 package.json
│       ├── 📄 tsconfig.json
│       ├── 📄 app.json                              # Configuração do Expo
│       └── 📂 src/
│           ├── 📄 App.tsx
│           └── 📂 screens/
│               ├── 📄 HomeScreen.tsx                # Tela inicial
│               └── 📄 MarketplaceScreen.tsx         # Tela do marketplace
│
├── 📂 runtimes/                                     # Runtimes externos
│   ├── 📂 remix-runtime/                           # Runtime Remix (TypeScript/Node)
│   │   ├── 📄 package.json
│   │   ├── 📄 tsconfig.json
│   │   └── 📂 src/
│   │       ├── 📄 index.ts
│   │       ├── 📄 server.ts
│   │       └── 📄 compiler.ts
│   │
│   ├── 📂 prometheus-bridge/                       # Bridge para Prometheus (TypeScript/Node)
│   │   ├── 📄 package.json
│   │   └── 📂 src/
│   │       └── 📄 index.ts
│   │
│   └── 📂 python-runtime/                          # Runtime Python
│       ├── 📄 pyproject.toml
│       ├── 📄 requirements.txt
│       └── 📂 src/
│           ├── 📄 __init__.py
│           └── 📄 simulator.py
│
├── 📂 configs/                                      # Configurações (TOML/YAML)
│   ├── 📄 server.toml                               # Configuração do servidor
│   ├── 📄 agent.toml                                # Configuração de agentes
│   ├── 📄 marketplace.toml                          # Configuração do marketplace
│   └── 📄 kernel.toml                               # Configuração do microkernel
│
├── 📂 scripts/                                      # Scripts de automação (Bash/Python)
│   ├── 📄 build.sh                                  # Build completo do sistema
│   ├── 📄 test.sh                                   # Executa todos os testes
│   ├── 📄 deploy.sh                                 # Deploy para ambientes de produção
│   ├── 📄 release.sh                                # Gera artefatos de release e instaladores
│   ├── 📄 build-wasm.sh                             # 🆕 Build para WebAssembly (mldsa-kit)
│   ├── 📄 bundle.sh                                 # 🆕 Wrapper para cargo-bundle
│   ├── 📄 run-desktop.sh                            # Executa a aplicação desktop
│   ├── 📄 build-android.sh                          # Compila a biblioteca Android JNI (.so)
│   ├── 📄 build-ios.sh                              # Compila a biblioteca estática iOS (.a)
│   └── 📂 docker/
│       ├── 📄 Dockerfile.server                     # Dockerfile para o servidor
│       ├── 📄 Dockerfile.cli                        # Dockerfile para a CLI
│       └── 📄 docker-compose.yml                    # Orquestração de serviços com Docker Compose
│
├── 📂 native/                                        # Wrappers nativos para mobile/desktop
│   ├── 📂 desktop/                                  # Desktop (wry WebView)
│   │   ├── 📄 Cargo.toml
│   │   └── 📂 src/
│   │       └── 📄 main.rs
│   ├── 📂 android/                                  # Android (WebView + Kotlin)
│   │   ├── 📄 build.gradle
│   │   └── 📂 app/
│   │       └── 📂 src/
│   │           └── 📂 main/
│   │               └── 📂 kotlin/
│   │                   └── 📄 MainActivity.kt      # Atividade principal Android
│   └── 📂 ios/                                      # iOS (WKWebView + Swift)
│       ├── 📄 ios.xcodeproj/                        # Projeto Xcode
│       └── 📂 ios/
│           └── 📂 App/
│               └── 📄 AppDelegate.swift            # Delegado da aplicação iOS
│
├── 📂 installers/                                   # Instaladores para todas as plataformas
│   ├── 📂 debian/                                   # Pacote .deb (Linux)
│   │   ├── 📄 control                               # Metadados do pacote Debian
│   │   └── 📄 postinst                              # Script pós-instalação
│   ├── 📂 rpm/                                      # Pacote .rpm (Linux)
│   │   └── 📄 cathedral-os.spec                     # Especificação do pacote RPM
│   ├── 📂 windows/                                  # Instalador .msi (Windows)
│   │   ├── 📄 installer.nsi                         # Script NSIS para o instalador
│   │   └── 📄 cathedral-os.wxs                      # Script WiX para o instalador MSI
│   ├── 📂 macos/                                    # Pacote .app / .dmg (macOS)
│   │   ├── 📄 Info.plist                            # Metadados do aplicativo macOS
│   │   └── 📄 cathedral-os.dmg                      # Imagem de disco para distribuição
│   ├── 📂 android/                                  # Pacote .apk / .aab (Android)
│   │   └── 📄 build.gradle                          # Configuração de build do Gradle
│   └── 📂 ios/                                      # Pacote .ipa (iOS)
│       └── 📄 Info.plist                            # Metadados do aplicativo iOS
│
├── 📂 tests/                                        # Testes em todos os níveis
│   ├── 📂 unit/                                     # Testes unitários (Rust)
│   │   ├── 📄 codesign_tests.rs
│   │   └── 📄 marketplace_tests.rs
│   ├── 📂 integration/                              # Testes de integração (Rust)
│   │   ├── 📄 api_tests.rs
│   │   └── 📄 hw_sw_tests.rs
│   └── 📂 e2e/                                      # Testes end-to-end (TypeScript/Playwright)
│       ├── 📄 package.json
│       └── 📂 src/
│           └── 📄 e2e.test.ts
│
├── 📂 tools/                                        # Ferramentas de desenvolvimento
│   ├── 📂 bazel/                                    # Regras Bazel (build system)
│   │   ├── 📄 BUILD
│   │   └── 📂 rules/
│   │       └── 📄 hdl.bzl                           # Regras para HDL (Verilog/SystemVerilog)
│   └── 📂 codegen/
│       └── 📄 generate_uast.py                      # Gerador de código para UAST
│
├── 📂 .cursor/                                      # Configuração do Cursor IDE
│   └── 📄 rules.mdc                                 # Regras específicas do projeto para o Cursor
│
├── 📄 Cargo.toml                                    # Workspace Rust (Raiz)
├── 📄 rust-toolchain.toml                           # MSRV 1.93.0
├── 📄 package.json                                  # Workspace Node.js (Raiz)
├── 📄 pnpm-workspace.yaml                           # Workspace pnpm
├── 📄 tsconfig.base.json                            # Configuração TypeScript base
├── 📄 commitlint.config.js                          # Configuração do commitlint
├── 📄 lint-staged.config.js                         # Configuração do lint-staged
├── 📄 turbo.json                                    # Configuração do Turborepo
├── 📄 WORKSPACE                                      # Configuração Bazel (raiz)
└── 📄 .bazelrc                                      # Configuração do Bazel
"""

lines = tree_str.strip().split('\n')
current_path = []
for line in lines:
    line = line.split('#')[0]
    line = line.rstrip()
    if not line: continue

    # Identify depth based on '│' or spaces before '├──' or '└──'
    match = re.match(r'^((?:│\   |\s{4})*)(├──|└──)?\s*(?:📂|📄)?\s*(.+?)\s*$', line)
    if not match: continue

    prefix, _, name = match.groups()
    if name == "cathedral-os/" or name == "│":
        continue

    depth = len(prefix) // 4

    current_path = current_path[:depth]

    # remove trailing slash from directory name
    is_dir = name.endswith('/')
    name = name.rstrip('/')

    full_path = os.path.join(*current_path, name) if current_path else name

    # Only touch files that don't exist
    if is_dir:
        os.makedirs(full_path, exist_ok=True)
        current_path.append(name)
    else:
        # Create directory if missing
        os.makedirs(os.path.dirname(full_path) or '.', exist_ok=True)
        # Create empty file only if it doesn't exist
        if not os.path.exists(full_path):
            with open(full_path, 'w') as f:
                if full_path.endswith('Cargo.toml') and 'crates/' in full_path:
                    package_name = os.path.basename(os.path.dirname(full_path))
                    f.write(f'[package]\nname = "{package_name}"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\n')
                elif full_path.endswith('Cargo.toml') and 'desktop' in full_path:
                    f.write(f'[package]\nname = "desktop"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\n')
                elif full_path.endswith('package.json') and ('packages/' in full_path or 'apps/' in full_path or 'marketplace/' in full_path or 'runtimes/' in full_path or 'tests/' in full_path):
                    package_name = os.path.basename(os.path.dirname(full_path))
                    f.write(f'{{\n  "name": "{package_name}",\n  "version": "0.1.0"\n}}\n')
                else:
                    pass
