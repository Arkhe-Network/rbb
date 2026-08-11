import json
import argparse
import random

SPECIALTIES = [
    ("Matemática", "math", [100, 300], "ValidatorA"),
    ("Física", "phys", [300, 300], "ValidatorB"),
    ("Computação", "cs", [500, 300], "ValidatorC"),
    ("Engenharia", "eng", [700, 300], "ValidatorD"),
    ("IA", "ai", [900, 300], "RedundantA"),
    ("Robótica", "rob", [1100, 300], "RedundantB"),
    ("Ciências Naturais", "nat", [100, 600], "WitnessA"),
    ("Ciências Humanas", "human", [300, 600], "WitnessB"),
    ("Linguística", "lang", [500, 600], "WitnessC"),
    ("História", "hist", [700, 600], "RouterA"),
    ("Filosofia", "phil", [900, 600], "RouterB"),
]

def generate_agent_nodes(specialty, count, base_position, specialty_id, anchor_type, pq_scheme="Falcon512"):
    nodes = []
    connections = {}
    for i in range(1, count + 1):
        curvature = round(0.05 + 0.85 * (i / max(count, 1)), 2)
        sector = random.choice(["VortexBound", "VortexProliferated", "Dual", "Winding", "Momentum"])
        ee = round(random.uniform(0.1, 0.9), 2)
        collapse_prob = round(0.01 * (1.0 - curvature), 3)
        reliability = round(0.5 + 0.5 * (i / max(count, 1)), 2)
        audit_count = random.randint(0, 10)
        correction_count = random.randint(0, max(audit_count, 0))
        agent_id = f"agent-{specialty_id}-{i}"
        agent_name = f"{specialty} {i}"
        x, y = base_position
        x_offset = (i - 1) * 80
        y_offset = (i - 1) * 60
        lat = random.uniform(-90, 90)
        lon = random.uniform(-180, 180)

        node = {
            "parameters": {
                "systemMessage": (
                    f"Você é um especialista em {specialty}. "
                    f"Âncora: {anchor_type}. Curvatura local: {curvature:.2f}. "
                    f"Setor topológico: {sector}. "
                    f"Entropia de emaranhamento: {ee:.2f}. "
                    f"Probabilidade de colapso: {collapse_prob:.3f}. "
                    f"Confiabilidade: {reliability:.2f}. "
                    f"Auditorias: {audit_count}. "
                    f"Correções: {correction_count}. "
                    f"Esquema PQ: {pq_scheme}. "
                    f"Nó ARKHE-Net em ({lat:.2f}, {lon:.2f}). "
                    f"Latência fixa: 3.3 ms. "
                    f"Participe do ciclo de correção contínua: propostas são auditadas, "
                    f"erros são corrigidos, e correções são re-auditadas."
                ),
                "promptType": "conversational"
            },
            "id": agent_id,
            "name": agent_name,
            "type": "@n8n/n8n-nodes-langchain.agent",
            "typeVersion": 1,
            "position": [x + x_offset, y + y_offset],
            "pq_scheme": pq_scheme,
            "curvature": curvature,
            "anchor_type": anchor_type,
            "topological_sector": sector,
            "entanglement_entropy": ee,
            "collapse_probability": collapse_prob,
            "reliability": reliability,
            "audit_count": audit_count,
            "correction_count": correction_count,
            "lat": lat,
            "lon": lon,
            "specialty": specialty,
        }
        nodes.append(node)
        # v2.1 FIX: Connect to coordinator, not directly to merge
        connections[agent_name] = {
            "main": [[{"node": "Coordenador Central (Correção Contínua)", "type": "main", "index": 0}]]
        }
    return nodes, connections

def generate_auditor_nodes(count, base_position):
    nodes = []
    connections = {}
    for i in range(1, count + 1):
        reliability = round(0.6 + 0.35 * (i / max(count, 1)), 2)
        agent_id = f"auditor-{i}"
        # v2.1 FIX: Use Latin "Auditor" not Cyrillic
        agent_name = f"Auditor {i}"
        x, y = base_position
        x_offset = (i - 1) * 100
        y_offset = (i - 1) * 80
        node = {
            "parameters": {
                "systemMessage": (
                    f"Você é o Auditor {i}. "
                    f"Confiabilidade: {reliability:.2f}. "
                    f"Sua função é verificar as saídas dos agentes especializados. "
                    f"Detecte erros, inconsistências e alegações não verificadas. "
                    f"Registre suas auditorias no ledger via Witness Layer. "
                    f"Participe do ciclo de correção contínua."
                ),
                "promptType": "conversational"
            },
            "id": agent_id,
            "name": agent_name,
            "type": "@n8n/n8n-nodes-langchain.agent",
            "typeVersion": 1,
            "position": [x + x_offset, y + y_offset],
            "reliability": reliability,
            "agent_type": "auditor",
        }
        nodes.append(node)
        # v2.1 FIX: Connect to coordinator
        connections[agent_name] = {
            "main": [[{"node": "Coordenador Central (Correção Contínua)", "type": "main", "index": 0}]]
        }
    return nodes, connections

def generate_corrector_nodes(count, base_position):
    nodes = []
    connections = {}
    for i in range(1, count + 1):
        reliability = round(0.5 + 0.4 * (i / max(count, 1)), 2)
        agent_id = f"corrector-{i}"
        agent_name = f"Corretor {i}"
        x, y = base_position
        x_offset = (i - 1) * 100
        y_offset = (i - 1) * 80
        node = {
            "parameters": {
                "systemMessage": (
                    f"Você é o Corretor {i}. "
                    f"Confiabilidade: {reliability:.2f}. "
                    f"Sua função é aplicar correções baseadas nas auditorias. "
                    f"Use o ledger para consultar o histórico de erros e correções. "
                    f"Registre suas correções no ledger via Witness Layer. "
                    f"Participe do ciclo de correção contínua."
                ),
                "promptType": "conversational"
            },
            "id": agent_id,
            "name": agent_name,
            "type": "@n8n/n8n-nodes-langchain.agent",
            "typeVersion": 1,
            "position": [x + x_offset, y + y_offset],
            "reliability": reliability,
            "agent_type": "corrector",
        }
        nodes.append(node)
        # v2.1 FIX: Connect to coordinator
        connections[agent_name] = {
            "main": [[{"node": "Coordenador Central (Correção Contínua)", "type": "main", "index": 0}]]
        }
    return nodes, connections

def generate_hypergraph_v21(total_agents, auditors=5, correctors=5):
    """Gera o workflow completo com ciclo de correção contínua — VERSÃO 2.1 CORRIGIDA."""
    # v2.1 FIX: Distribute agents evenly, not lose some to integer division
    base_per_specialty = total_agents // len(SPECIALTIES)
    remainder = total_agents % len(SPECIALTIES)

    # v2.1 FIX: Add trigger node (webhook)
    fixed_nodes = [
        {
            "parameters": {
                "path": "arkhe-n-v21",
                "responseMode": "responseNode",
                "options": {}
            },
            "id": "trigger-webhook",
            "name": "ARKHE-N Trigger",
            "type": "n8n-nodes-base.webhook",
            "typeVersion": 2,
            "position": [50, 50],
            "webhookId": "arkhe-n-v21"
        },
        {
            "parameters": {
                "systemMessage": (
                    "Você é o Coordenador Central do Ciclo de Correção Contínua (v2.1). "
                    "Sua função é orquestrar o ciclo: Proposta → Auditoria → Correção → Re-auditoria. "
                    "REGRAS: (1) Receba propostas dos agentes especializados. "
                    "(2) Encaminhe para auditores APENAS se a confiabilidade < 0.8. "
                    "(3) Se auditoria encontrar erros, encaminhe para corretores. "
                    "(4) Re-audite correções antes de aprovar. "
                    "(5) Registre cada etapa no ledger via Witness Layer. "
                    "(6) Se confiança >= 0.9 após re-auditoria, encaminhe para merge. "
                    "(7) Se confiança < 0.9 após 3 ciclos, rejeite e registre no Error Manager."
                ),
                "promptType": "conversational",
                "maxIterations": 15,
                "options": {
                    "temperature": 0.2
                }
            },
            "id": "central-coordinator",
            "name": "Coordenador Central (Correção Contínua)",
            "type": "@n8n/n8n-nodes-langchain.agent",
            "typeVersion": 1,
            "position": [850, 50]
        },
        {
            "parameters": {
                "method": "POST",
                "url": "={{ $env.ARKHE_WITNESS_URL || 'http://localhost:8080/api/v2.0/witness' }}",
                "sendBody": True,
                "bodyParameters": {
                    "parameters": [
                        {"name": "proposal_id", "value": "={{ $json.proposal_id || 'unknown' }}"},
                        {"name": "sender_did", "value": "={{ $json.sender_did || 'unknown' }}"},
                        {"name": "agent_type", "value": "={{ $json.agent_type || 'specialist' }}"},
                        {"name": "action", "value": "={{ $json.action || 'propose' }}"},
                        {"name": "payload", "value": "={{ JSON.stringify($json) }}"},
                        {"name": "signature", "value": "={{ $json.signature || '' }}"},
                        {"name": "curvature", "value": "={{ $json.curvature || 0.0 }}"},
                        {"name": "topological_sector", "value": "={{ $json.topological_sector || 'VortexBound' }}"},
                        {"name": "reliability", "value": "={{ $json.reliability || 0.5 }}"},
                        {"name": "audit_count", "value": "={{ $json.audit_count || 0 }}"},
                        {"name": "correction_count", "value": "={{ $json.correction_count || 0 }}"}
                    ]
                },
                "options": {
                    "timeout": 5000
                }
            },
            "id": "witness-layer",
            "name": "ARKHE-N Witness (Correção Contínua)",
            "type": "n8n-nodes-base.httpRequest",
            "typeVersion": 4,
            "position": [850, 250]
        },
        {
            "parameters": {
                "jsCode": (
                    "// ARKHE-N v2.1 Witness Preprocessor\n"
                    "const event = $input.first().json;\n"
                    "const timestamp = Date.now();\n"
                    "const agent_id = event.agent_id || event.id || 'unknown';\n"
                    "const agent_type = event.agent_type || 'specialist';\n"
                    "const anchor_type = event.anchor_type || 'unknown';\n"
                    "const curvature = parseFloat(event.curvature) || 0.0;\n"
                    "const topological_sector = event.topological_sector || 'VortexBound';\n"
                    "const reliability = parseFloat(event.reliability) || 0.5;\n"
                    "const audit_count = parseInt(event.audit_count) || 0;\n"
                    "const correction_count = parseInt(event.correction_count) || 0;\n"
                    "const entanglement_entropy = parseFloat(event.entanglement_entropy) || 0.0;\n"
                    "const collapse_probability = parseFloat(event.collapse_probability) || 0.0;\n"
                    "const pq_scheme = event.pq_scheme || 'Falcon512';\n"
                    "const signature = event.signature || '';\n"
                    "const lat = parseFloat(event.lat) || 0.0;\n"
                    "const lon = parseFloat(event.lon) || 0.0;\n"
                    "// Simple hash using n8n-compatible approach\n"
                    "const hashInput = JSON.stringify(event) + timestamp;\n"
                    "let hash = 0;\n"
                    "for (let i = 0; i < hashInput.length; i++) {\n"
                    "  const char = hashInput.charCodeAt(i);\n"
                    "  hash = ((hash << 5) - hash) + char;\n"
                    "  hash = hash & hash;\n"
                    "}\n"
                    "const event_hash = '0x' + Math.abs(hash).toString(16).padStart(64, '0');\n"
                    "return [{\n"
                    "  json: {\n"
                    "    ...event,\n"
                    "    event_hash,\n"
                    "    timestamp,\n"
                    "    agent_id,\n"
                    "    agent_type,\n"
                    "    anchor_type,\n"
                    "    curvature,\n"
                    "    topological_sector,\n"
                    "    reliability,\n"
                    "    audit_count,\n"
                    "    correction_count,\n"
                    "    entanglement_entropy,\n"
                    "    collapse_probability,\n"
                    "    pq_scheme,\n"
                    "    signature,\n"
                    "    lat,\n"
                    "    lon,\n"
                    "    witness_status: 'pending'\n"
                    "  }\n"
                    "}];"
                )
            },
            "id": "witness-preprocessor",
            "name": "Witness Preprocessor (Correção)",
            "type": "n8n-nodes-base.code",
            "typeVersion": 2,
            "position": [650, 250]
        },
        {
            "parameters": {
                "jsCode": (
                    "// ARKHE-N v2.1 Witness Processor\n"
                    "const response = $input.first().json;\n"
                    "const event = $input.first().json;\n"
                    "if (response.is_anchored === true) {\n"
                    "  event.witness_status = 'ANCHORED';\n"
                    "  event.block_height = response.block_height || 0;\n"
                    "  event.energy_cost_mj = response.energy_cost_mj || 0;\n"
                    "  event.anchor_type = response.anchor_type || event.anchor_type;\n"
                    "  event.curvature = parseFloat(response.curvature) || event.curvature;\n"
                    "  event.topological_sector = response.topological_sector || event.topological_sector;\n"
                    "  event.reliability = parseFloat(response.reliability) || event.reliability;\n"
                    "  event.audit_count = (parseInt(event.audit_count) || 0) + 1;\n"
                    "  event.correction_count = parseInt(response.correction_count) || event.correction_count;\n"
                    "  event.pq_scheme = response.pq_scheme || event.pq_scheme;\n"
                    "  event.signature = response.signature || event.signature;\n"
                    "  event.chord_latency_ms = parseFloat(response.chord_latency_ms) || 3.3;\n"
                    "} else {\n"
                    "  event.witness_status = 'REJECTED';\n"
                    "  event.rejection_reason = response.rejection_reason || 'unknown';\n"
                    "}\n"
                    "return [{ json: event }];"
                )
            },
            "id": "witness-processor",
            "name": "Witness Processor (Correção)",
            "type": "n8n-nodes-base.code",
            "typeVersion": 2,
            "position": [1050, 250]
        },
        {
            "parameters": {
                "systemMessage": (
                    "Você é o Gestor de Erros (Error Manager) v2.1. "
                    "Sua função é manter o registro de erros e correções. "
                    "Calcule a confiabilidade geral do sistema, identifique padrões de erro, "
                    "e recomende melhorias no processo de correção contínua. "
                    "Se um agente falhar 3+ vezes consecutivas, recomende sua substituição. "
                    "Use a curvatura de Forman-Ricci para detectar gargalos no ciclo."
                ),
                "promptType": "conversational",
                "maxIterations": 5,
                "options": {
                    "temperature": 0.1
                }
            },
            "id": "error-manager",
            "name": "Gestor de Erros",
            "type": "@n8n/n8n-nodes-langchain.agent",
            "typeVersion": 1,
            "position": [850, 450]
        },
        # v2.1 FIX: Use a Switch node instead of Merge for routing
        {
            "parameters": {
                "rules": {
                    "rules": [
                        {
                            "value": "approved",
                            "output": 0,
                            "conditions": [
                                {
                                    "value": "approved",
                                    "operator": {
                                        "type": "string",
                                        "operation": "equals"
                                    },
                                    "leftValue": "={{ $json.coordinator_decision }}",
                                    "rightValue": "approved"
                                }
                            ]
                        },
                        {
                            "value": "needs_audit",
                            "output": 1,
                            "conditions": [
                                {
                                    "value": "needs_audit",
                                    "operator": {
                                        "type": "string",
                                        "operation": "equals"
                                    },
                                    "leftValue": "={{ $json.coordinator_decision }}",
                                    "rightValue": "needs_audit"
                                }
                            ]
                        },
                        {
                            "value": "needs_correction",
                            "output": 2,
                            "conditions": [
                                {
                                    "value": "needs_correction",
                                    "operator": {
                                        "type": "string",
                                        "operation": "equals"
                                    },
                                    "leftValue": "={{ $json.coordinator_decision }}",
                                    "rightValue": "needs_correction"
                                }
                            ]
                        },
                        {
                            "value": "rejected",
                            "output": 3,
                            "conditions": [
                                {
                                    "value": "rejected",
                                    "operator": {
                                        "type": "string",
                                        "operation": "equals"
                                    },
                                    "leftValue": "={{ $json.coordinator_decision }}",
                                    "rightValue": "rejected"
                                }
                            ]
                        }
                    ]
                }
            },
            "id": "coordinator-router",
            "name": "Roteador do Coordenador",
            "type": "n8n-nodes-base.switch",
            "typeVersion": 2,
            "position": [850, 150]
        },
        {
            "parameters": {},
            "id": "responder",
            "name": "Responder ao Usuário",
            "type": "n8n-nodes-base.respondToWebhook",
            "typeVersion": 1,
            "position": [1250, 50]
        }
    ]

    all_nodes = fixed_nodes
    all_connections = {}

    # v2.1 FIX: Distribute remainder agents evenly
    for idx, (name, sid, pos, anchor) in enumerate(SPECIALTIES):
        count = base_per_specialty + (1 if idx < remainder else 0)
        nodes, conns = generate_agent_nodes(name, count, pos, sid, anchor)
        all_nodes.extend(nodes)
        all_connections.update(conns)

    auditor_nodes, auditor_conns = generate_auditor_nodes(auditors, [100, 800])
    corrector_nodes, corrector_conns = generate_corrector_nodes(correctors, [1100, 800])
    all_nodes.extend(auditor_nodes)
    all_nodes.extend(corrector_nodes)
    all_connections.update(auditor_conns)
    all_connections.update(corrector_conns)

    # v2.1 FIX: Proper connection topology with routing
    all_connections["ARKHE-N Trigger"] = {
        "main": [[{"node": "Coordenador Central (Correção Contínua)", "type": "main", "index": 0}]]
    }
    all_connections["Coordenador Central (Correção Contínua)"] = {
        "main": [[{"node": "Roteador do Coordenador", "type": "main", "index": 0}]]
    }
    # Route 0: approved -> merge -> responder
    # Route 1: needs_audit -> auditors -> back to coordinator
    # Route 2: needs_correction -> correctors -> back to coordinator
    # Route 3: rejected -> error manager
    all_connections["Roteador do Coordenador"] = {
        "main": [
            [{"node": "Responder ao Usuário", "type": "main", "index": 0}],  # approved
            [{"node": "Auditor 1", "type": "main", "index": 0}],  # needs_audit -> first auditor
            [{"node": "Corretor 1", "type": "main", "index": 0}],  # needs_correction -> first corrector
            [{"node": "Gestor de Erros", "type": "main", "index": 0}]  # rejected
        ]
    }
    # Audit loop: auditors -> witness -> back to coordinator
    for i in range(1, auditors + 1):
        all_connections[f"Auditor {i}"] = {
            "main": [[{"node": "Witness Preprocessor (Correção)", "type": "main", "index": 0}]]
        }
    # Correction loop: correctors -> witness -> back to coordinator
    for i in range(1, correctors + 1):
        all_connections[f"Corretor {i}"] = {
            "main": [[{"node": "Witness Preprocessor (Correção)", "type": "main", "index": 0}]]
        }

    all_connections["Witness Preprocessor (Correção)"] = {
        "main": [[{"node": "ARKHE-N Witness (Correção Contínua)", "type": "main", "index": 0}]]
    }
    all_connections["ARKHE-N Witness (Correção Contínua)"] = {
        "main": [[{"node": "Witness Processor (Correção)", "type": "main", "index": 0}]]
    }
    # v2.1 FIX: Witness processor feeds back to coordinator for re-audit
    all_connections["Witness Processor (Correção)"] = {
        "main": [[{"node": "Coordenador Central (Correção Contínua)", "type": "main", "index": 0}]]
    }
    all_connections["Gestor de Erros"] = {
        "main": [[{"node": "Responder ao Usuário", "type": "main", "index": 0}]]
    }

    total_agents_count = sum(base_per_specialty + (1 if idx < remainder else 0) for idx in range(len(SPECIALTIES))) + auditors + correctors
    workflow_name = f"ARKHE-N v2.1 — Correção Contínua + {total_agents_count} agentes"

    return {
        "name": workflow_name,
        "nodes": all_nodes,
        "connections": all_connections
    }

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="Gerador de Hipergrafo n8n — ARKHE‑N v2.1 (Inércia + RSI)")
    parser.add_argument("--total-agents", type=int, default=1000, help="Número total de agentes especializados")
    parser.add_argument("--auditors", type=int, default=5, help="Número de auditores")
    parser.add_argument("--correctors", type=int, default=5, help="Número de corretores")
    parser.add_argument("--output", type=str, default="hypergraph_v2.1.json", help="Arquivo de saída")
    args = parser.parse_args()

    # Test with 1000 agents as requested
    workflow = generate_hypergraph_v21(args.total_agents, args.auditors, args.correctors)

    with open(args.output, "w") as f:
        json.dump(workflow, f, indent=2)
    print(f"Generated JSON successfully to {args.output}")
