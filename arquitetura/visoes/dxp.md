# DXP — Digital Experience Platform
## Arquitetura Completa v1.0.0
## Nubank AI-First Platform Ecosystem

### 1. Visão Executiva
O DXP (Digital Experience Platform) é a plataforma AI-first da Nubank projetada para comprimir radicalmente o workflow de desenvolvimento de produto — do conceito à produção.

**O Problema: Creation Gap + Workflow Gap**

| Métrica | Status Atual (BDC Stack) |
|---|---|
| Tempo médio | ~49 dias úteis |
| Steps com handoff | ~24 steps |
| Qualidade | Pós-hoc, manual |
| Escalabilidade | Linear com headcount |

**A Solução: AI-Native por Default**
O DXP transforma a criação de experiências "Magic App" em um processo AI-native, onde agentes inteligentes orquestram o ciclo completo: design → especificação → código → qualidade → deploy.

### 2. Arquitetura de Produtos (4 Pilares)
```text
┌─────────────────────────────────────────────────────────────────┐
│                    DXP PLATFORM ECOSYSTEM                       │
├─────────────┬─────────────┬─────────────┬───────────────────────┤
│  DXP Studio │ DXP Dictionary│ DXP Spec   │    DXP Workflow      │
│  (Create)   │  (Know)      │ Protocol   │   (Orchestrate)      │
├─────────────┼─────────────┼─────────────┼───────────────────────┤
│ Figma → AI  │ NuDS, Tokens│ Vendor     │ Human + AI Agent      │
│ → BDC Code  │ Patterns    │ Agnostic   │ Teams Lifecycle       │
└─────────────┴─────────────┴─────────────┴───────────────────────┘
         │              │              │              │
         └──────────────┴──────────────┴──────────────┘
                          │
                    ┌─────┴─────┐
                    │  OUTPUT   │
                    │ Production│
                    │  BDC Code │
                    └───────────┘
```

### 3. DXP Studio — AI-Powered Creation Environment
#### 3.1 Pipeline Figma → DXP Spec → BDC
```text
┌──────────┐    ┌─────────────┐    ┌──────────────┐    ┌──────────┐
│  FIGMA   │───→│  AI Parser  │───→│ DXP Spec     │───→│  BDC     │
│  Design  │    │  (Vision)   │    │ Generator    │    │  Code    │
└──────────┘    └─────────────┘    └──────────────┘    └──────────┘
      │               │                  │                  │
      │         ┌─────┴─────┐      ┌────┴────┐       ┌────┴────┐
      │         │  Layout   │      │ NuDS    │       │ Golden  │
      │         │  Analysis │      │ Mapping │       │ Tests   │
      │         └───────────┘      └─────────┘       └─────────┘
```

#### 3.2 Componentes Arquiteturais

| Componente | Responsabilidade | Tecnologia |
|---|---|---|
| Figma Plugin | Exporta design tokens + estrutura visual | Figma REST API + Plugin SDK |
| AI Vision Parser | Converte design em estrutura semântica | GPT-4V / Claude 3 Opus |
| Layout Engine | Análise de constraints, grids, responsividade | Custom + Yoga Layout |
| NuDS Mapper | Mapeia elementos para componentes canônicos | RAG + DXP Dictionary |
| Spec Generator | Emite DXP Spec Protocol (vendor-agnostic) | Template engine + LLM |
| BDC Codegen | Gera código BDC production-ready | Custom codegen + LLM |
| Quality Gate | Validação automática pré-geração | Rule engine + AI reviewer |

#### 3.3 DXP Spec Protocol (Vendor-Agnostic)
```typescript
// DXP Spec Protocol v1.0
interface DXPExperience {
  id: string;
  version: string;
  metadata: {
    name: string;
    description: string;
    targetPlatform: "mobile" | "web" | "both";
    accessibility: WCAGLevel;
  };
  designTokens: {
    colors: DesignToken[];
    typography: DesignToken[];
    spacing: DesignToken[];
    shadows: DesignToken[];
  };
  layout: {
    type: "screen" | "component" | "widget";
    root: LayoutNode;
    constraints: Constraint[];
  };
  components: {
    nuDS: NuDSComponent[];      // Componentes canônicos
    custom: CustomComponent[];  // Componentes custom
  };
  interactions: Interaction[];
  dataBindings: DataBinding[];
  analytics: AnalyticsConfig;
}

interface NuDSComponent {
  componentId: string;           // e.g., "nuds/button-primary"
  variant: string;             // e.g., "size=large,state=default"
  props: Record<string, any>;
  children?: NuDSComponent[];
}
```

### 4. DXP Dictionary — Knowledge Base Nu-Aware
#### 4.1 Arquitetura de Conhecimento
```text
┌─────────────────────────────────────────────────────────────┐
│                  DXP DICTIONARY KNOWLEDGE GRAPH              │
├──────────────┬──────────────┬──────────────┬──────────────┤
│   NuDS Specs │   Tokens     │   Patterns   │  Domain      │
│   (Components│   (Design    │   (Canonical │  Knowledge   │
│    Catalog)   │    System)   │    BDC)      │  (NuBank)    │
├──────────────┼──────────────┼──────────────┼──────────────┤
│ • Button     │ • Colors     │ • Auth flow  │ • Products   │
│ • Card       │ • Typography │ • Onboarding │ • Compliance │
│ • Input      │ • Spacing    │ • KYC        │ • Regulations│
│ • List       │ • Elevation  │ • Payments   │ • User types │
└──────────────┴──────────────┴──────────────┴──────────────┘
         │              │              │              │
         └──────────────┴──────────────┴──────────────┘
                          │
                    ┌─────┴─────┐
                    │  RAG +    │
                    │  Vector   │
                    │  Store    │
                    └───────────┘
```

#### 4.2 Ingestão e Estruturação

| Fonte | Formato | Pipeline | Frequência |
|---|---|---|---|
| NuDS Figma | JSON + imagens | AI parser → structured | On commit |
| NuDS Code | TypeScript/Flutter | AST extractor | On PR merge |
| BDC Patterns | YAML/JSON | Schema validator | On release |
| Domain Docs | Markdown/PDF | LLM summarizer + chunking | Weekly |
| Analytics | Events/Metrics | Feature extractor | Real-time |

#### 4.3 Consumo por Agentes
```text
Agent Request → Context Assembly → RAG Retrieval →
→ Knowledge Fusion → Agent Action → Feedback Loop →
→ Dictionary Update
```

### 5. DXP Workflow — Orchestration Platform
#### 5.1 Arquitetura de Agentes
```text
┌─────────────────────────────────────────────────────────────────┐
│                    DXP WORKFLOW ORCHESTRATOR                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │ Design   │  │ Codegen  │  │   QA     │  │ Deploy   │        │
│  │ Agent    │  │ Agent    │  │  Agent   │  │  Agent   │        │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘        │
│       │             │             │             │               │
│       └─────────────┴─────────────┴─────────────┘               │
│                     │                                           │
│              ┌──────┴──────┐                                    │
│              │  Supervisor │                                    │
│              │   Agent     │                                    │
│              └──────┬──────┘                                    │
│                     │                                           │
│              ┌──────┴──────┐                                    │
│              │   Human     │                                    │
│              │  Reviewer   │                                    │
│              └─────────────┘                                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 5.2 Agent Types & Responsabilidades

| Agente | Função | LLM/Tooling | MCP Tools |
|---|---|---|---|
| Design Mapper | Figma → DXP Spec | GPT-4V / Claude 3 | Figma API, NuDS Catalog |
| Codegen Agent | DXP Spec → BDC Code | Claude 3.5 Sonnet | BDC SDK, NuDS Components |
| QA Agent | Validação automática | Custom + GPT-4 | Test runner, A11y checker |
| Review Agent | Code review AI | Claude 3 Opus | GitHub API, Lint rules |
| Deploy Agent | Progressive rollout | Custom | Feature flags, Analytics |
| Supervisor | Orquestração + fallback | Claude 3 Opus | All MCP tools |

#### 5.3 Context Assembly Strategy
```text
┌─────────────────────────────────────────┐
│         CONTEXT ASSEMBLY PIPELINE        │
├─────────────────────────────────────────┤
│ 1. User Intent (prompt + design file)   │
│ 2. DXP Dictionary Retrieval (RAG)       │
│ 3. NuDS Component Context (live catalog)│
│ 4. BDC Pattern Context (canonical)      │
│ 5. Historical Context (similar experiences│
│ 6. Quality Rules (accessibility, tokens)│
│ 7. Assembled Context Window → LLM       │
└─────────────────────────────────────────┘
```

### 6. Integração Cross-Platform
#### 6.1 Mapa de Integrações
```text
                    ┌─────────────┐
                    │    DXP      │
                    │  Platform   │
                    └──────┬──────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
   ┌────┴────┐       ┌────┴────┐       ┌────┴────┐
   │  NuDS   │       │  BDC/   │       │   HPP   │
   │Catalog  │       │  Core   │       │ Widgets │
   │& Tokens │       │         │       │         │
   └────┬────┘       └────┬────┘       └────┬────┘
        │                  │                  │
   ┌────┴────┐       ┌────┴────┐       ┌────┴────┐
   │ Figma   │       │Backend  │       │Homepage │
   │Library  │       │Services │       │Infra    │
   └─────────┘       └─────────┘       └─────────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           │
                    ┌──────┴──────┐
                    │ App Excellence│
                    │(Observability)│
                    └─────────────┘
```

#### 6.2 Contratos de Integração

| Sistema | Contrato | Formato | Frequência |
|---|---|---|---|
| NuDS | Component catalog + tokens | JSON Schema + Figma | Real-time webhook |
| BDC/Core | Service patterns + API specs | OpenAPI + BDC YAML | On release |
| HPP | Widget schema + layout rules | JSON Schema | On change |
| App Excellence | Quality metrics + golden tests | Metrics API | Real-time |

### 7. Quality-at-Generation-Time
#### 7.1 Quality Gates Automatizados
```text
┌─────────────────────────────────────────────────────────────┐
│              QUALITY-AT-GENERATION PIPELINE                 │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │  NuDS       │  │   Token     │  │ Accessibility│         │
│  │ Compliance  │  │ Correctness │  │   (a11y)     │         │
│  │  Gate       │  │   Gate      │  │    Gate      │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │                │
│         └────────────────┼────────────────┘                │
│                          │                                 │
│                   ┌──────┴──────┐                          │
│                   │  Structural │                          │
│                   │  Integrity  │                          │
│                   │   Gate      │                          │
│                   └──────┬──────┘                          │
│                          │                                 │
│                   ┌──────┴──────┐                          │
│                   │  Golden   │                          │
│                   │  Tests    │                          │
│                   │  Gate     │                          │
│                   └──────┬──────┘                          │
│                          │                                 │
│                   ┌──────┴──────┐                          │
│                   │   PASS    │  → Production Deploy      │
│                   │   /FAIL   │  → Human Review            │
│                   └───────────┘                            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

#### 7.2 Regras de Qualidade

| Categoria | Regras | Enforcement |
|---|---|---|
| NuDS Compliance | Component canônico, variant correta, props válidos | AST + rule engine |
| Token Correctness | Cores do DS, typography scale, spacing grid | Token resolver |
| Accessibility | WCAG 2.1 AA, screen reader labels, contrast | A11y checker + AI |
| Structural Integrity | Layout constraints, responsive breakpoints | Layout engine |
| Golden Tests | Pixel-perfect match, interaction flows | Visual regression |

### 8. Arquitetura Técnica — Stack & Infra
#### 8.1 Diagrama de Componentes
```text
┌─────────────────────────────────────────────────────────────────┐
│                         CLIENT LAYER                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │ Figma Plugin│  │ DXP Studio  │  │ DXP Workflow Dashboard  │ │
│  │  (Plugin)   │  │   (Web)     │  │       (Web)             │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                         API GATEWAY                              │
│              (Auth, Rate Limiting, Routing)                     │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                      SERVICE LAYER                               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────┐ │
│  │  Studio  │ │Dictionary│ │  Spec    │ │ Workflow │ │Quality │ │
│  │ Service  │ │ Service  │ │ Protocol │ │ Service  │ │ Service│ │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                      AI/LLM LAYER                                │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────────────┐ │
│  │  LLM     │ │  RAG     │ │  Agent   │ │  MCP Tool Registry │ │
│  │ Router   │ │  Engine  │ │ Runtime  │ │                    │ │
│  │(Multi-  │ │(Vector  │ │(CrewAI/ │ │(Figma, GitHub,    │ │
│  │ provider)│ │ Store)  │ │ LangGraph│ │ BDC, NuDS, etc)   │ │
│  └──────────┘ └──────────┘ └──────────┘ └────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                      DATA LAYER                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────────────┐ │
│  │  Spec    │ │ Knowledge│ │  Vector  │ │  Analytics/        │ │
│  │  Store   │ │  Graph   │ │  DB      │ │  Metrics DB       │ │
│  │(PostgreSQL│ │(Neo4j)  │ │(PGVector)│ │(ClickHouse)      │ │
│  └──────────┘ └──────────┘ └──────────┘ └────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

#### 8.2 Stack Tecnológico

| Camada | Tecnologia | Justificativa |
|---|---|---|
| Frontend | React + TypeScript | Familiaridade Nubank, ecosystem |
| Backend | Kotlin / Go | Performance, type safety |
| AI/LLM | Claude 3.5 Sonnet, GPT-4V | Capacidade multimodal, reasoning |
| Agent Framework | CrewAI / LangGraph | Orquestração multi-agente |
| RAG | PGVector + LangChain | Vector search, context assembly |
| Knowledge Graph | Neo4j | Relacionamentos complexos NuDS |
| MCP | Custom MCP servers | Tool abstraction para agentes |
| Infra | Kubernetes + AWS/GCP | Escalabilidade, resiliência |
| Observability | Datadog / Grafana | Tracing, metrics, logs |

### 9. Roadmap Estratégico (6–18 meses)

**Fase 1: Foundation (Meses 1–3)**
- [ ] DXP Spec Protocol v1.0 estabilizado
- [ ] DXP Dictionary MVP com NuDS catalog
- [ ] DXP Studio POC: Figma → BDC para 3 templates
- [ ] Primeiro agente codegen operacional
- [ ] Quality gates básicos (NuDS + tokens)

**Fase 2: Integration (Meses 4–8)**
- [ ] Integração NuDS live catalog
- [ ] DXP Workflow orquestrando 4+ agentes
- [ ] MCP tool registry completo
- [ ] Quality-at-generation para 80% dos casos
- [ ] Primeiro time usando DXP em produção

**Fase 3: Scale (Meses 9–18)**
- [ ] Rollout company-wide (todos os squads)
- [ ] DXP Studio self-service para designers
- [ ] Agent farm com 10+ agentes especializados
- [ ] Analytics e feedback loop fechado
- [ ] Open-source de DXP Spec Protocol

### 10. Métricas de Sucesso

| Métrica | Baseline | Target Fase 1 | Target Fase 3 |
|---|---|---|---|
| Tempo design → code | 49 dias | 14 dias | 3 dias |
| Handoffs manuais | 24 | 8 | 2 |
| NuDS compliance | 60% (pós-hoc) | 85% (geração) | 99% (geração) |
| Experiências geradas/mês | ~10 | ~50 | ~500 |
| Engenheiros usando DXP | 0 | 20 | 1000+ |
| Tempo economizado/eng | 0% | 30% | 70% |

### 11. Riscos e Mitigações

| Risco | Probabilidade | Impacto | Mitigação |
|---|---|---|---|
| LLM hallucination | Alta | Alto | Quality gates + human review + golden tests |
| NuDS drift | Média | Alto | Webhook sync + versioned specs |
| Adoption resistance | Média | Alto | POCs com early adopters + métricas claras |
| Vendor lock-in | Baixa | Médio | DXP Spec Protocol vendor-agnostic |
| Performance AI | Média | Médio | Caching + model routing + async processing |

---
**Documento arquitetural DXP v1.0.0 — Nubank, 2026**
**Arquiteto:** Staff Software Engineer – DXP
