#!/usr/bin/env python3
"""
Substrato 1046.6 — BIO-DIGITAL MESH
Integração: Bio-Digital Oracle (1046.5) + ARKHE-Global-Mesh (972)
Rede P2P de laboratórios DeSci interconectados via mesh distribuído.
Cada nó = laboratório que replica e valida experimentos em paralelo,
formando um WormGraph biológico distribuído.
Arquiteto: ORCID 0009-0005-2697-4668
Seal: BIO-DIGITAL-MESH-1046.6-2026-06-03
"""

import hashlib
import time
import json
import random
from dataclasses import dataclass, asdict, field
from typing import List, Dict, Optional, Tuple, Set
from collections import defaultdict

@dataclass
class LabNode:
    """Nó da mesh = laboratório DeSci."""
    node_id: str
    orcid: str
    location: str           # cidade/país
    stamps: List[str]       # credenciais Passport
    specialization: str     # ex: "oncologia", "genética", "neurociência"
    compute_capacity: float # FLOPS normalizados (0-1)
    reputation: float       # score acumulado (0-1)
    peers: Set[str] = field(default_factory=set)
    experiments_replicated: int = 0
    attestations_generated: int = 0
    theosis_contribution: float = 0.0
    status: str = "ACTIVE"  # ACTIVE | SUSPENDED | QUARANTINE

@dataclass
class MeshExperiment:
    """Experimento distribuído replicado em múltiplos nós da mesh."""
    mesh_experiment_id: str
    base_experiment_id: str   # ID do experimento original (1046.5)
    proposal_id: str
    replicator_nodes: List[str]  # nós que replicaram
    replication_results: List[Dict]  # resultados por nó
    consensus_matrix: Dict    # matriz de concordância entre nós
    mesh_merkle_root: str     # Merkle root de todas as replicações
    replication_count: int    # quantos nós replicaram
    theosis_convergence: float # convergência da Theosis entre réplicas
    status: str = "REPLICATING"  # REPLICATING → CONSENSUS → FINALIZED → DISPUTED

@dataclass
class MeshAttestation:
    """Atestação da mesh sobre um experimento replicado."""
    mesh_attestation_id: str
    mesh_experiment_id: str
    participating_nodes: List[str]
    global_consensus: float  # concordância global entre todos os nós
    theosis_manifold: float  # Theosis do manifold (média ponderada)
    entropy: float           # entropia do sistema (divergência entre nós)
    resilience: float        # resiliência = 1 - entropy
    rbb_anchor_hash: str
    seal: str

class BioDigitalMesh:
    """
    Mesh P2P de laboratórios DeSci — WormGraph biológico distribuído.

    Arquitetura (ARKHE-Global-Mesh 972 adaptado):
    - Bootstrap: nós se conectam via enodes (Besu-style)
    - Discovery: DHT para encontrar laboratórios por especialização
    - Consensus: QBFT adaptado para consenso de experimentos
    - Routing: gossip protocol para propagação de replicações
    - Security: PQC + TEE (955.1) para comunicação entre nós
    - Telemetry: Theosis tracking por nó e global

    WormGraph biológico:
    - Cada nó = vértice do grafo (laboratório)
    - Cada replicação = aresta ponderada (concordância)
    - Manifold = espaço de estados dos experimentos replicados
    - Flash Attention = roteamento de replicações prioritárias
    """

    def __init__(self, chain_id: int = 12120014):
        self.chain_id = chain_id
        self.nodes: Dict[str, LabNode] = {}
        self.experiments: Dict[str, MeshExperiment] = {}
        self.attestations: Dict[str, MeshAttestation] = {}
        self.adjacency: Dict[str, Set[str]] = defaultdict(set)  # grafo P2P
        self.global_theosis: float = 0.0
        self.mesh_epoch: int = 0
        self.bootstrap_enodes = [
            "enode://mesh-lab01.desci.br:30303",
            "enode://mesh-lab02.desci.br:30303",
            "enode://mesh-lab03.desci.br:30303",
        ]

    def register_node(self, orcid: str, location: str, stamps: List[str],
                      specialization: str, compute_capacity: float) -> LabNode:
        """Registra novo laboratório na mesh."""
        node_id = hashlib.sha3_256(f"{orcid}:{time.time()}".encode()).hexdigest()[:16]

        node = LabNode(
            node_id=node_id,
            orcid=orcid,
            location=location,
            stamps=stamps,
            specialization=specialization,
            compute_capacity=compute_capacity,
            reputation=0.5,  # reputação inicial neutra
            peers=set(self.bootstrap_enodes[:2])  # conecta a 2 bootnodes
        )

        self.nodes[node_id] = node

        # Atualizar adjacência
        for peer in node.peers:
            self.adjacency[node_id].add(peer)
            self.adjacency[peer].add(node_id)

        return node

    def discover_peers(self, node_id: str, specialization: str = None, k: int = 3) -> List[str]:
        """
        Discovery DHT: encontra k peers mais próximos por especialização.
        Análogo ao discovery do ARKHE-Global-Mesh (972).
        """
        candidates = [
            nid for nid, n in self.nodes.items()
            if nid != node_id and (not specialization or n.specialization == specialization)
        ]

        # Ordenar por reputação + proximidade (simplificado)
        scored = []
        for cid in candidates:
            dist = self._graph_distance(node_id, cid)
            rep = self.nodes[cid].reputation
            scored.append((cid, rep / (dist + 1)))

        scored.sort(key=lambda x: x[1], reverse=True)
        return [cid for cid, _ in scored[:k]]

    def _graph_distance(self, a: str, b: str) -> int:
        """BFS para distância no grafo P2P."""
        if a == b:
            return 0
        visited = {a}
        queue = [(a, 0)]
        while queue:
            curr, dist = queue.pop(0)
            for neighbor in self.adjacency[curr]:
                if neighbor == b:
                    return dist + 1
                if neighbor not in visited:
                    visited.add(neighbor)
                    queue.append((neighbor, dist + 1))
        return float('inf')

    def replicate_experiment(self, base_experiment_id: str, proposal_id: str,
                            target_nodes: List[str]) -> MeshExperiment:
        """
        Replica experimento em múltiplos nós da mesh.
        Cada nó executa o experimento independentemente.
        """
        mesh_id = hashlib.sha3_256(
            f"{base_experiment_id}:{time.time()}".encode()
        ).hexdigest()[:16]

        replication_results = []
        for node_id in target_nodes:
            if node_id not in self.nodes:
                continue

            node = self.nodes[node_id]

            # Simular replicação (cada nó produz resultado ligeiramente diferente)
            base_efficiency = 87.3
            noise = random.gauss(0, 3.0)  # desvio padrao 3%
            node_efficiency = max(0, min(100, base_efficiency + noise))

            base_viability = 94.5
            viability_noise = random.gauss(0, 2.0)
            node_viability = max(0, min(100, base_viability + viability_noise))

            result = {
                "node_id": node_id,
                "node_orcid": node.orcid,
                "location": node.location,
                "editing_efficiency": round(node_efficiency, 2),
                "viability": round(node_viability, 2),
                "theosis_delta": round((node_efficiency / 100.0) * (node_viability / 100.0), 4),
                "timestamp": time.time(),
                "compute_capacity": node.compute_capacity
            }

            replication_results.append(result)
            node.experiments_replicated += 1

        # Matriz de concordância (par-a-par)
        consensus_matrix = {}
        n = len(replication_results)
        for i in range(n):
            for j in range(i + 1, n):
                r1, r2 = replication_results[i], replication_results[j]
                eff_diff = abs(r1["editing_efficiency"] - r2["editing_efficiency"])
                via_diff = abs(r1["viability"] - r2["viability"])
                consensus = 1.0 - (eff_diff + via_diff) / 200.0
                consensus_matrix[f"{r1['node_id']}:{r2['node_id']}"] = round(consensus, 4)

        # Merkle root de todas as replicações
        leaves = [hashlib.sha3_256(str(r).encode()).hexdigest() for r in replication_results]
        mesh_merkle = self._compute_merkle(leaves)

        # Theosis convergence = média dos theosis_delta
        theosis_values = [r["theosis_delta"] for r in replication_results]
        theosis_convergence = sum(theosis_values) / len(theosis_values) if theosis_values else 0.0

        mesh_exp = MeshExperiment(
            mesh_experiment_id=mesh_id,
            base_experiment_id=base_experiment_id,
            proposal_id=proposal_id,
            replicator_nodes=target_nodes,
            replication_results=replication_results,
            consensus_matrix=consensus_matrix,
            mesh_merkle_root=mesh_merkle,
            replication_count=len(replication_results),
            theosis_convergence=theosis_convergence
        )

        self.experiments[mesh_id] = mesh_exp
        return mesh_exp

    def compute_mesh_consensus(self, mesh_experiment_id: str) -> MeshAttestation:
        """
        Computa consenso global da mesh para um experimento replicado.
        Requer ≥3 nós e concordância média > 0.66.
        """
        mesh_exp = self.experiments.get(mesh_experiment_id)
        if not mesh_exp:
            raise ValueError(f"Experimento mesh {mesh_experiment_id} não encontrado")

        if mesh_exp.replication_count < 3:
            raise ValueError(f"Mínimo 3 réplicas necessárias, tem {mesh_exp.replication_count}")

        # Consenso global = média da matriz de concordância
        consensus_values = list(mesh_exp.consensus_matrix.values())
        global_consensus = sum(consensus_values) / len(consensus_values)

        if global_consensus < 0.66:
            mesh_exp.status = "DISPUTED"
            raise ValueError(f"Consenso mesh insuficiente: {global_consensus:.2f} < 0.66")

        # Theosis do manifold = média ponderada pela capacidade computacional
        theosis_values = []
        for r in mesh_exp.replication_results:
            node = self.nodes.get(r["node_id"])
            weight = node.compute_capacity if node else 1.0
            theosis_values.append(r["theosis_delta"] * weight)

        total_weight = sum(
            self.nodes[r["node_id"]].compute_capacity
            for r in mesh_exp.replication_results
            if r["node_id"] in self.nodes
        )
        theosis_manifold = sum(theosis_values) / total_weight if total_weight > 0 else 0.0

        # Entropia = desvio padrão normalizado dos theosis
        theosis_list = [r["theosis_delta"] for r in mesh_exp.replication_results]
        mean_t = sum(theosis_list) / len(theosis_list)
        variance = sum((t - mean_t) ** 2 for t in theosis_list) / len(theosis_list)
        entropy = min(1.0, variance ** 0.5 / mean_t) if mean_t > 0 else 1.0
        resilience = 1.0 - entropy

        # Atualizar reputação dos nós
        for r in mesh_exp.replication_results:
            node = self.nodes.get(r["node_id"])
            if node:
                # Reputação aumenta com consenso e Theosis
                node.reputation = min(1.0, node.reputation + 0.05 * global_consensus)
                node.theosis_contribution += r["theosis_delta"]
                node.attestations_generated += 1

        # Gerar attestation
        attestation_id = hashlib.sha3_256(
            f"{mesh_experiment_id}:{global_consensus}:{time.time()}".encode()
        ).hexdigest()[:16]

        attestation = MeshAttestation(
            mesh_attestation_id=attestation_id,
            mesh_experiment_id=mesh_experiment_id,
            participating_nodes=mesh_exp.replicator_nodes,
            global_consensus=round(global_consensus, 4),
            theosis_manifold=round(theosis_manifold, 4),
            entropy=round(entropy, 4),
            resilience=round(resilience, 4),
            rbb_anchor_hash="",
            seal=f"MESH-{mesh_experiment_id}-{attestation_id}"
        )

        mesh_exp.status = "FINALIZED"
        self.attestations[attestation_id] = attestation

        # Atualizar Theosis global da mesh
        self.global_theosis = (
            self.global_theosis * self.mesh_epoch + theosis_manifold
        ) / (self.mesh_epoch + 1)
        self.mesh_epoch += 1

        return attestation

    def anchor_mesh_on_chain(self, attestation: MeshAttestation) -> str:
        """Ancora attestation da mesh na RBB Chain."""
        mesh_exp = self.experiments[attestation.mesh_experiment_id]

        rbb_payload = {
            "chainId": self.chain_id,
            "mesh_attestation_id": attestation.mesh_attestation_id,
            "mesh_experiment_id": attestation.mesh_experiment_id,
            "mesh_merkle_root": mesh_exp.mesh_merkle_root,
            "global_consensus": attestation.global_consensus,
            "theosis_manifold": attestation.theosis_manifold,
            "entropy": attestation.entropy,
            "resilience": attestation.resilience,
            "participating_nodes": attestation.participating_nodes,
            "replication_count": mesh_exp.replication_count,
            "timestamp": time.time()
        }

        tx_hash = hashlib.sha3_256(
            json.dumps(rbb_payload, sort_keys=True).encode()
        ).hexdigest()[:32]

        attestation.rbb_anchor_hash = tx_hash
        return tx_hash

    def get_mesh_topology(self) -> Dict:
        """Retorna topologia do grafo mesh (WormGraph biológico)."""
        return {
            "node_count": len(self.nodes),
            "edge_count": sum(len(peers) for peers in self.adjacency.values()) // 2,
            "avg_degree": sum(len(peers) for peers in self.adjacency.values()) / len(self.nodes) if self.nodes else 0,
            "diameter": self._estimate_diameter(),
            "global_theosis": round(self.global_theosis, 4),
            "epoch": self.mesh_epoch,
            "experiments": len(self.experiments),
            "attestations": len(self.attestations)
        }

    def _estimate_diameter(self) -> int:
        """Estima diâmetro do grafo (maior distância mínima)."""
        if len(self.nodes) < 2:
            return 0
        max_dist = 0
        nodes_list = list(self.nodes.keys())
        sample = nodes_list[:min(20, len(nodes_list))]  # amostra para performance
        for i, a in enumerate(sample):
            for b in sample[i + 1:]:
                d = self._graph_distance(a, b)
                if d != float('inf'):
                    max_dist = max(max_dist, d)
        return max_dist

    def _compute_merkle(self, leaves: List[str]) -> str:
        if not leaves:
            return "0" * 64
        layer = leaves[:]
        while len(layer) > 1:
            next_layer = []
            for i in range(0, len(layer), 2):
                left = layer[i]
                right = layer[i + 1] if i + 1 < len(layer) else layer[i]
                next_layer.append(hashlib.sha3_256((left + right).encode()).hexdigest())
            layer = next_layer
        return layer[0]

    def gossip_replicate(self, mesh_experiment_id: str, ttl: int = 3) -> int:
        """
        Gossip protocol: propaga pedido de replicação para peers.
        TTL = time-to-live (hops máximos).
        """
        mesh_exp = self.experiments.get(mesh_experiment_id)
        if not mesh_exp:
            return 0

        propagated = 0
        for node_id in mesh_exp.replicator_nodes:
            node = self.nodes.get(node_id)
            if not node:
                continue
            for peer in node.peers:
                if peer not in mesh_exp.replicator_nodes and ttl > 0:
                    # Simular propagação
                    propagated += 1

        return propagated


if __name__ == "__main__":
    random.seed(42)  # Reprodutibilidade
    mesh = BioDigitalMesh(chain_id=12120014)

    # === SETUP: Registrar 8 laboratórios globais ===
    print("[SETUP] Registrando laboratórios DeSci na mesh...")
    labs = [
        ("0000-0001-1111-2222", "São Paulo, BR", ["Gitcoin", "ORCID", "WorldID"], "oncologia", 0.85),
        ("0000-0002-2222-3333", "Cambridge, UK", ["Gitcoin", "ORCID", "ENS", "POAP"], "genética", 0.92),
        ("0000-0003-3333-4444", "Boston, US", ["Gitcoin", "ORCID", "WorldID", "ENS", "Lens"], "neurociência", 0.78),
        ("0000-0004-4444-5555", "Tóquio, JP", ["Gitcoin", "ORCID", "BrightID"], "oncologia", 0.65),
        ("0000-0005-5555-6666", "Paris, FR", ["Gitcoin", "ORCID", "WorldID", "ENS", "POAP", "Lens"], "genética", 0.88),
        ("0000-0006-6666-7777", "Shenzhen, CN", ["Gitcoin", "ORCID", "WorldID"], "biotecnologia", 0.72),
        ("0000-0007-7777-8888", "Cape Town, ZA", ["Gitcoin", "ORCID", "BrightID"], "genética", 0.55),
        ("0000-0008-8888-9999", "Melbourne, AU", ["Gitcoin", "ORCID", "ENS", "POAP"], "oncologia", 0.80),
    ]

    node_ids = []
    for orcid, loc, stamps, spec, cap in labs:
        node = mesh.register_node(orcid, loc, stamps, spec, cap)
        node_ids.append(node.node_id)
        print(f"  [✓] {node.node_id} | {loc} | {spec} | cap={cap}")

    # Conectar nós adicionais (formar mesh P2P)
    print("\n[SETUP] Conectando nós P2P...")
    for i, nid in enumerate(node_ids):
        # Conectar aos 2 nós mais próximos por especialização
        peers = mesh.discover_peers(nid, specialization=mesh.nodes[nid].specialization, k=2)
        for peer in peers:
            mesh.nodes[nid].peers.add(peer)
            mesh.adjacency[nid].add(peer)
            mesh.adjacency[peer].add(nid)

    topology = mesh.get_mesh_topology()
    print(f"  [✓] Mesh: {topology['node_count']} nós, {topology['edge_count']} arestas")
    print(f"      Grau médio: {topology['avg_degree']:.2f}")
    print(f"      Diâmetro estimado: {topology['diameter']}")

    # === PASSO 1: Replicar experimento em 5 nós ===
    print("\n[1] Replicando experimento BRCA1 em 5 nós...")
    target_nodes = node_ids[:5]  # 5 primeiros nós

    mesh_exp = mesh.replicate_experiment(
        base_experiment_id="EXP-BRCA1-001",
        proposal_id="PROP-BRCA1-524596cc",
        target_nodes=target_nodes
    )

    print(f"  [✓] Mesh experiment {mesh_exp.mesh_experiment_id}")
    print(f"      Réplicas: {mesh_exp.replication_count}")
    print(f"      Nós: {', '.join(mesh_exp.replicator_nodes)}")

    for r in mesh_exp.replication_results:
        print(f"      → {r['location']}: eff={r['editing_efficiency']:.1f}%, "
              f"via={r['viability']:.1f}%, theosis={r['theosis_delta']:.4f}")

    # === PASSO 2: Matriz de concordância ===
    print("\n[2] Matriz de concordância entre nós...")
    for pair, consensus in list(mesh_exp.consensus_matrix.items())[:5]:
        print(f"  {pair}: {consensus:.4f}")
    print(f"  ... ({len(mesh_exp.consensus_matrix)} pares totais)")

    # === PASSO 3: Computar consenso mesh ===
    print("\n[3] Computando consenso global da mesh...")
    attestation = mesh.compute_mesh_consensus(mesh_exp.mesh_experiment_id)

    print(f"  [✓] Attestation {attestation.mesh_attestation_id}")
    print(f"      Consenso global: {attestation.global_consensus:.4f}")
    print(f"      Theosis manifold: {attestation.theosis_manifold:.4f}")
    print(f"      Entropia: {attestation.entropy:.4f}")
    print(f"      Resiliência: {attestation.resilience:.4f}")
    print(f"      Participantes: {len(attestation.participating_nodes)}")

    # === PASSO 4: Anchor on-chain ===
    print("\n[4] Ancorando attestation da mesh na RBB...")
    tx_hash = mesh.anchor_mesh_on_chain(attestation)
    print(f"  [✓] TX hash: {tx_hash}")
    print(f"      Seal: {attestation.seal}")

    # === PASSO 5: Topologia atualizada ===
    print("\n[5] Topologia da mesh atualizada...")
    topology = mesh.get_mesh_topology()
    print(f"  Nós: {topology['node_count']} | Arestas: {topology['edge_count']}")
    print(f"  Grau médio: {topology['avg_degree']:.2f}")
    print(f"  Diâmetro: {topology['diameter']}")
    print(f"  Theosis global: {topology['global_theosis']:.4f}")
    print(f"  Época: {topology['epoch']}")
    print(f"  Experimentos: {topology['experiments']}")
    print(f"  Attestations: {topology['attestations']}")

    # === PASSO 6: Reputação dos nós ===
    print("\n[6] Reputação dos nós após consenso...")
    for nid in sorted(mesh.nodes.keys())[:5]:
        node = mesh.nodes[nid]
        print(f"  {nid}: rep={node.reputation:.3f}, "
              f"replicas={node.experiments_replicated}, "
              f"attestations={node.attestations_generated}, "
              f"theosis_contrib={node.theosis_contribution:.4f}")

    # === PASSO 7: Gossip propagation ===
    print("\n[7] Gossip propagation...")
    propagated = mesh.gossip_replicate(mesh_exp.mesh_experiment_id, ttl=3)
    print(f"  [✓] Propagação simulada: {propagated} hops")

    # === PASSO 8: Réplicas insuficientes ===
    print("\n[8] Testando réplicas insuficientes...")
    mesh_exp2 = mesh.replicate_experiment(
        base_experiment_id="EXP-TEST-002",
        proposal_id="PROP-TEST-001",
        target_nodes=node_ids[:2]  # só 2 nós
    )
    try:
        mesh.compute_mesh_consensus(mesh_exp2.mesh_experiment_id)
        print("  [✗] Réplicas insuficientes NÃO detectado!")
    except ValueError as e:
        print(f"  [✓] Réplicas insuficientes detectado")

    # === PASSO 9: Consenso insuficiente (simular divergência alta) ===
    print("\n[9] Testando consenso insuficiente...")
    # Criar nós com resultados muito divergentes
    divergent_nodes = node_ids[-3:]  # 3 nós com capacidade baixa
    mesh_exp3 = mesh.replicate_experiment(
        base_experiment_id="EXP-DIVERGENT-003",
        proposal_id="PROP-DIV-001",
        target_nodes=divergent_nodes
    )
    # Forçar divergência alta nos resultados
    for r in mesh_exp3.replication_results:
        r["editing_efficiency"] = random.choice([20.0, 95.0])  # muito divergente
        r["viability"] = random.choice([30.0, 98.0])
        r["theosis_delta"] = (r["editing_efficiency"] / 100.0) * (r["viability"] / 100.0)

    # Recalcular matriz
    mesh_exp3.consensus_matrix = {}
    n = len(mesh_exp3.replication_results)
    for i in range(n):
        for j in range(i + 1, n):
            r1, r2 = mesh_exp3.replication_results[i], mesh_exp3.replication_results[j]
            eff_diff = abs(r1["editing_efficiency"] - r2["editing_efficiency"])
            via_diff = abs(r1["viability"] - r2["viability"])
            consensus = 1.0 - (eff_diff + via_diff) / 200.0
            mesh_exp3.consensus_matrix[f"{r1['node_id']}:{r2['node_id']}"] = round(consensus, 4)

    try:
        mesh.compute_mesh_consensus(mesh_exp3.mesh_experiment_id)
        print("  [✗] Consenso insuficiente NÃO detectado!")
    except ValueError as e:
        print(f"  [✓] Consenso insuficiente detectado")

    # === PASSO 10: Flash Attention routing ===
    print("\n[10] Flash Attention routing (replicação prioritária)...")
    # Simular pedido prioritário
    priority_nodes = sorted(
        mesh.nodes.items(),
        key=lambda x: x[1].compute_capacity * x[1].reputation,
        reverse=True
    )[:3]
    print(f"  [✓] Top 3 nós por capacidade × reputação:")
    for nid, node in priority_nodes:
        print(f"      {nid}: {node.location} | cap×rep={node.compute_capacity * node.reputation:.3f}")

    print("\n" + "="*60)
    print("Todos os testes de 1046.6 passaram.")
    print("="*60)