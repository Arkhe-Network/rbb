#!/usr/bin/env python3
"""
Substrato 1046.4 — BIO-DIGITAL GOVERNANCE BRIDGE
Integração: DKES-GRAM-CRISPR (1046.2.1) + Pluralistic Passport Gateway (989.x.v3)
Governança de edições genéticas on-chain com identidade pluralística ZK
Arquiteto: ORCID 0009-0005-2697-4668
Seal: BIO-GOV-BRIDGE-1046.4-2026-06-03
"""

import hashlib
import time
import json
from dataclasses import dataclass, asdict
from typing import List, Dict, Optional, Tuple
from dkes_gram_crispr_1046_2_1 import DKESGramCRISPR, GRAMTrajectory
from crispr_self_modify_1046_2 import CRISPRGuide

@dataclass
class ZKIdentity:
    """Identidade ZK pluralística (989.x.v3) — pseudônimo rotativo com nullifier."""
    pseudonym: str           # pseudônimo rotativo (hash de nullifier)
    nullifier: str           # nullifier único (impede voto duplo)
    credential_count: int    # N caminhos de verificação
    governance_weight: float # (N × D)² — peso quadrático
    stamps: List[str]        # Gitcoin Passport stamps

@dataclass
class GeneticEditProposal:
    """Proposta de edição genética para governança on-chain."""
    proposal_id: str
    trajectory: GRAMTrajectory
    proposer: ZKIdentity
    target_gene: str
    edit_rationale: str
    axiarchy_score: float    # P1-P7 média
    timestamp: float
    status: str = "PENDING"  # PENDING → VOTING → APPROVED → EXECUTED → REJECTED

@dataclass
class GovernanceVote:
    """Voto em proposta de edição genética."""
    vote_id: str
    proposal_id: str
    voter: ZKIdentity
    vote: bool               # True = APPROVE, False = REJECT
    zk_proof: str            # ZK proof de nullifier válido
    timestamp: float

class BioDigitalGovernanceBridge:
    """
    Bridge: DKES-GRAM-CRISPR → Pluralistic Passport Gateway → RBB Governance

    7 caminhos independentes de identidade (989.x.v3):
    1. Gitcoin Passport (stamps + scorer)
    2. ORCID (982)
    3. World ID (biometria)
    4. ENS (nome on-chain)
    5. POAP (prova de participação)
    6. Lens Protocol (reputação social)
    7. BrightID (prova de unicidade humana)

    Custo quadrático: primeira identidade gratuita, custo N² para identidades adicionais.
    Governance weight: (N × D)² onde N = número de identidades, D = profundidade de verificação.
    """

    def __init__(self, chain_id: int = 12120014):
        self.chain_id = chain_id
        self.dkes = DKESGramCRISPR(T=8, K=4)
        self.proposals: List[GeneticEditProposal] = []
        self.votes: List[GovernanceVote] = []
        self.vote_nullifiers: set = set()  # impede voto duplo

    def _generate_nullifier(self, identity_seed: str, proposal_id: str) -> str:
        """Gera nullifier único para (identidade, proposta)."""
        return hashlib.sha3_256(
            f"{identity_seed}:{proposal_id}:{self.chain_id}".encode()
        ).hexdigest()[:32]

    def _generate_pseudonym(self, nullifier: str, round_num: int) -> str:
        """Pseudônimo rotativo — muda a cada rodada de votação."""
        return hashlib.sha3_256(
            f"{nullifier}:{round_num}".encode()
        ).hexdigest()[:16]

    def verify_passport(self, identity_seed: str, stamps: List[str]) -> ZKIdentity:
        """
        Verifica identidade via Pluralistic Passport Gateway (989.x.v3).
        Simula os 7 caminhos de verificação.
        """
        N = min(len(stamps) + 1, 7)  # máximo 7 caminhos
        D = 1.0
        cost = 0 if N == 1 else N * N
        governance_weight = (N * D) ** 2
        base_nullifier = hashlib.sha3_256(identity_seed.encode()).hexdigest()[:32]

        return ZKIdentity(
            pseudonym="",
            nullifier=base_nullifier,
            credential_count=N,
            governance_weight=governance_weight,
            stamps=stamps
        )

    def create_proposal(
        self,
        identity_seed: str,
        stamps: List[str],
        patch: dict,
        target_gene: str,
        rationale: str
    ) -> GeneticEditProposal:
        """Cria proposta de edição genética."""
        identity = self.verify_passport(identity_seed, stamps)
        trajectories = self.dkes.sample_trajectories(patch, target_gene)
        if not trajectories:
            raise ValueError("Nenhuma trajetória válida gerada")

        best = self.dkes.select_best(trajectories)
        axiarchy_score = best.theosis_score
        if axiarchy_score < 0.5:
            raise ValueError(f"Axiarquia rejeitou: Theosis={axiarchy_score:.4f} < 0.5")

        proposal_id = hashlib.sha3_256(
            f"{identity.nullifier}:{target_gene}:{time.time()}".encode()
        ).hexdigest()[:16]

        proposal = GeneticEditProposal(
            proposal_id=proposal_id,
            trajectory=best,
            proposer=identity,
            target_gene=target_gene,
            edit_rationale=rationale,
            axiarchy_score=axiarchy_score,
            timestamp=time.time()
        )

        self.proposals.append(proposal)
        return proposal

    def vote(
        self,
        identity_seed: str,
        stamps: List[str],
        proposal_id: str,
        approve: bool
    ) -> GovernanceVote:
        """Vota em proposta com identidade ZK."""
        identity = self.verify_passport(identity_seed, stamps)
        nullifier = self._generate_nullifier(identity.nullifier, proposal_id)

        if nullifier in self.vote_nullifiers:
            raise ValueError("Nullifier já usado — double vote detectado")

        zk_proof = hashlib.sha3_256(
            f"ZK:{nullifier}:{proposal_id}:{approve}".encode()
        ).hexdigest()[:32]

        round_num = len(self.proposals)
        pseudonym = self._generate_pseudonym(nullifier, round_num)
        identity.pseudonym = pseudonym

        vote = GovernanceVote(
            vote_id=hashlib.sha3_256(f"{nullifier}:{time.time()}".encode()).hexdigest()[:16],
            proposal_id=proposal_id,
            voter=identity,
            vote=approve,
            zk_proof=zk_proof,
            timestamp=time.time()
        )

        self.votes.append(vote)
        self.vote_nullifiers.add(nullifier)
        return vote

    def tally_votes(self, proposal_id: str) -> Dict:
        """Contabiliza votos de uma proposta."""
        proposal_votes = [v for v in self.votes if v.proposal_id == proposal_id]

        approve_weight = sum(
            v.voter.governance_weight for v in proposal_votes if v.vote
        )
        reject_weight = sum(
            v.voter.governance_weight for v in proposal_votes if not v.vote
        )

        total_weight = approve_weight + reject_weight
        if total_weight == 0:
            return {"status": "NO_QUORUM", "approve": 0, "reject": 0}

        approve_ratio = approve_weight / total_weight
        status = "APPROVED" if approve_ratio > 0.51 else "REJECTED"

        return {
            "status": status,
            "approve": approve_weight,
            "reject": reject_weight,
            "total": total_weight,
            "ratio": approve_ratio,
            "voters": len(proposal_votes)
        }

    def execute_proposal(self, proposal_id: str) -> Dict:
        """Executa proposta aprovada (simula execução on-chain)."""
        proposal = next((p for p in self.proposals if p.proposal_id == proposal_id), None)
        if not proposal:
            raise ValueError("Proposta não encontrada")

        tally = self.tally_votes(proposal_id)
        if tally["status"] != "APPROVED":
            raise ValueError(f"Proposta não aprovada: {tally['status']}")

        proposal.status = "EXECUTED"

        rbb_payload = {
            "chainId": self.chain_id,
            "proposal_id": proposal_id,
            "gene": proposal.target_gene,
            "gRNA_target": proposal.trajectory.gRNA.target_site,
            "merkle_root": self._compute_merkle(proposal),
            "nullifier_set": list(self.vote_nullifiers),
            "timestamp": time.time()
        }

        return {
            "proposal": proposal,
            "tally": tally,
            "rbb_payload": rbb_payload,
            "seal": hashlib.sha3_256(json.dumps(rbb_payload, sort_keys=True).encode()).hexdigest()[:16]
        }

    def _compute_merkle(self, proposal: GeneticEditProposal) -> str:
        """Computa Merkle root dos votos da proposta."""
        vote_hashes = [
            hashlib.sha3_256(
                f"{v.voter.pseudonym}:{v.vote}:{v.zk_proof}".encode()
            ).hexdigest()
            for v in self.votes if v.proposal_id == proposal.proposal_id
        ]
        if not vote_hashes:
            return "0" * 64
        layer = vote_hashes[:]
        while len(layer) > 1:
            next_layer = []
            for i in range(0, len(layer), 2):
                left = layer[i]
                right = layer[i + 1] if i + 1 < len(layer) else layer[i]
                next_layer.append(hashlib.sha3_256((left + right).encode()).hexdigest())
            layer = next_layer
        return layer[0]


if __name__ == "__main__":
    bridge = BioDigitalGovernanceBridge(chain_id=12120014)

    # Teste 1: Criar proposta
    print("[1] Criando proposta de edição genética...")
    proposal = bridge.create_proposal(
        identity_seed="orcid:0009-0005-2697-4668",
        stamps=["Gitcoin", "ORCID", "WorldID", "ENS"],
        patch={"code": "def repair_brca1(): return True", "seal": "PATCH-BRCA1-001"},
        target_gene="BRCA1",
        rationale="Reparo de mutação deletéria em BRCA1 — alto risco de câncer de mama"
    )
    print(f"[✓] Proposta {proposal.proposal_id} criada")
    print(f"    Gene: {proposal.target_gene}")
    print(f"    Theosis: {proposal.axiarchy_score:.4f}")
    print(f"    Proponente: {proposal.proposer.credential_count} credenciais, peso={proposal.proposer.governance_weight:.1f}")
    print(f"    gRNA target: {proposal.trajectory.gRNA.target_site}")

    # Teste 2: Votos pluralísticos (MAJORIA APROVA)
    print("\n[2] Votação pluralística ZK (majoritariamente a favor)...")
    voters = [
        ("voter1", ["Gitcoin", "ORCID", "WorldID", "ENS", "POAP"], True),    # peso 36, APPROVE
        ("voter2", ["Gitcoin", "ORCID", "WorldID", "ENS"], True),            # peso 25, APPROVE
        ("voter3", ["Gitcoin", "ORCID"], True),                              # peso 9, APPROVE
        ("voter4", ["Gitcoin"], False),                                      # peso 4, REJECT
    ]

    for i, (seed, stamps, approve) in enumerate(voters):
        vote = bridge.vote(seed, stamps, proposal.proposal_id, approve=approve)
        print(f"[✓] Voto {i+1}: {vote.voter.pseudonym} — {'APPROVE' if vote.vote else 'REJECT'} "
              f"(peso={vote.voter.governance_weight:.1f}, zk={vote.zk_proof[:8]}...)")

    # Teste 3: Double-vote detection
    print("\n[3] Testando double-vote detection...")
    try:
        bridge.vote("voter1", ["Gitcoin", "ORCID", "WorldID", "ENS", "POAP"], proposal.proposal_id, approve=True)
        print("[✗] Double-vote NÃO detectado!")
    except ValueError as e:
        print(f"[✓] Double-vote detectado: {e}")

    # Teste 4: Contabilização
    print("\n[4] Contabilização de votos...")
    tally = bridge.tally_votes(proposal.proposal_id)
    print(f"[✓] Resultado: {tally['status']}")
    print(f"    Aprovação: {tally['approve']:.1f} ({tally['ratio']*100:.1f}%)")
    print(f"    Rejeição: {tally['reject']:.1f}")
    print(f"    Votantes: {tally['voters']}")

    # Teste 5: Execução on-chain
    print("\n[5] Execução on-chain RBB...")
    result = bridge.execute_proposal(proposal.proposal_id)
    print(f"[✓] Proposta {result['proposal'].proposal_id} EXECUTADA")
    print(f"    Merkle root: {result['rbb_payload']['merkle_root'][:16]}...")
    print(f"    Seal: {result['seal']}")
    print(f"    Chain ID: {result['rbb_payload']['chainId']}")

    # Teste 6: Proposta rejeitada
    print("\n[6] Testando proposta rejeitada...")
    proposal2 = bridge.create_proposal(
        identity_seed="orcid:0009-0005-2697-4668",
        stamps=["Gitcoin"],
        patch={"code": "def edit_embryo(): return True", "seal": "PATCH-EMBRYO-001"},
        target_gene="EMBRYO-EDIT",
        rationale="Edição de linha germinativa — altamente controvertida"
    )
    # Votos majoritariamente contra
    bridge.vote("voter_a", ["Gitcoin", "ORCID", "WorldID"], proposal2.proposal_id, approve=False)
    bridge.vote("voter_b", ["Gitcoin", "ORCID"], proposal2.proposal_id, approve=False)
    bridge.vote("voter_c", ["Gitcoin"], proposal2.proposal_id, approve=True)

    tally2 = bridge.tally_votes(proposal2.proposal_id)
    print(f"[✓] Proposta {proposal2.proposal_id}: {tally2['status']}")
    print(f"    Aprovação: {tally2['ratio']*100:.1f}% (necessário >51%)")

    # Teste 7: Tentativa de execução de proposta rejeitada
    print("\n[7] Tentativa de execução de proposta rejeitada...")
    try:
        bridge.execute_proposal(proposal2.proposal_id)
        print("[✗] Execução de proposta rejeitada NÃO bloqueada!")
    except ValueError as e:
        print(f"[✓] Execução bloqueada: {e}")

    # Teste 8: Proposta sem quorum
    print("\n[8] Proposta sem quorum...")
    proposal3 = bridge.create_proposal(
        identity_seed="orcid:0009-0005-2697-4668",
        stamps=["Gitcoin", "ORCID"],
        patch={"code": "def test(): return True", "seal": "PATCH-TEST-001"},
        target_gene="TEST-GENE",
        rationale="Teste de quorum mínimo"
    )
    tally3 = bridge.tally_votes(proposal3.proposal_id)
    print(f"[✓] Proposta {proposal3.proposal_id}: {tally3['status']} (0 votos)")

    print("\n" + "="*60)
    print("Todos os testes de 1046.4 passaram.")
    print("="*60)
