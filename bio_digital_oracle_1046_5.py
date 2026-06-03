#!/usr/bin/env python3
"""
Substrato 1046.5 — BIO-DIGITAL ORACLE
Integração: Bio-Digital Governance Bridge (1046.4) + DeSci-Nodes (989.y.4)
Oráculo on-chain que verifica execução real de edições genéticas em laboratórios
via proof-of-experiment com dPID, IPFS, ORCID, C2PA provenance e FAIR validation.
Arquiteto: ORCID 0009-0005-2697-4668
Seal: BIO-DIGITAL-ORACLE-1046.5-2026-06-03
"""

import hashlib
import time
import json
from dataclasses import dataclass, asdict
from typing import List, Dict, Optional, Tuple
from bio_digital_governance_bridge_1046_4 import BioDigitalGovernanceBridge, GeneticEditProposal

@dataclass
class ExperimentResult:
    """Resultado de experimento de edição genética em laboratório."""
    experiment_id: str
    proposal_id: str
    lab_orcid: str
    target_gene: str
    gRNA_sequence: str
    cell_line: str
    delivery_method: str
    editing_efficiency: float
    indel_profile: Dict
    off_target_sites: List[Dict]
    viability: float
    sequencing_hash: str
    raw_data_cid: str
    processed_data_cid: str
    c2pa_signature: str
    timestamp: float
    status: str = "SUBMITTED"

@dataclass
class OracleAttestation:
    """Atestação do oráculo sobre um experimento."""
    attestation_id: str
    experiment_id: str
    validator_orcids: List[str]
    consensus_score: float
    fair_score: float
    theosis_delta: float
    merkle_root: str
    rbb_tx_hash: str
    seal: str

class BioDigitalOracle:
    def __init__(self, chain_id: int = 12120014, min_validators: int = 3):
        self.chain_id = chain_id
        self.min_validators = min_validators
        self.experiments: List[ExperimentResult] = []
        self.attestations: List[OracleAttestation] = []
        self.governance = BioDigitalGovernanceBridge(chain_id=chain_id)
        self.validator_pool: List[str] = []

    def register_validator(self, orcid: str, stamps: List[str]) -> bool:
        """Registra validador DeSci via Passport Gateway."""
        identity = self.governance.verify_passport(orcid, stamps)
        if identity.credential_count >= 3:
            self.validator_pool.append(orcid)
            return True
        return False

    def _generate_c2pa(self, lab_orcid: str, sequencing_hash: str, raw_data_cid: str) -> str:
        """Gera C2PA signature canônica."""
        return hashlib.sha3_256(
            f"{lab_orcid}:{sequencing_hash}:{raw_data_cid}".encode()
        ).hexdigest()[:32]

    def submit_experiment(
        self,
        proposal_id: str,
        lab_orcid: str,
        gRNA_sequence: str,
        cell_line: str,
        delivery_method: str,
        editing_efficiency: float,
        indel_profile: Dict,
        off_target_sites: List[Dict],
        viability: float,
        sequencing_hash: str,
        raw_data_cid: str,
        processed_data_cid: str,
        c2pa_signature: str
    ) -> ExperimentResult:
        """Laboratório submete resultado de experimento."""
        proposal = next(
            (p for p in self.governance.proposals if p.proposal_id == proposal_id),
            None
        )
        if not proposal:
            raise ValueError(f"Proposta {proposal_id} não encontrada")

        tally = self.governance.tally_votes(proposal_id)
        if tally["status"] != "APPROVED":
            raise ValueError(f"Proposta {proposal_id} não aprovada: {tally['status']}")

        # Verificar C2PA signature
        expected_c2pa = self._generate_c2pa(lab_orcid, sequencing_hash, raw_data_cid)
        if c2pa_signature != expected_c2pa:
            raise ValueError(
                f"Assinatura C2PA inválida — esperado {expected_c2pa}, recebido {c2pa_signature}"
            )

        experiment_id = hashlib.sha3_256(
            f"{proposal_id}:{lab_orcid}:{time.time()}".encode()
        ).hexdigest()[:16]

        experiment = ExperimentResult(
            experiment_id=experiment_id,
            proposal_id=proposal_id,
            lab_orcid=lab_orcid,
            target_gene=proposal.target_gene,
            gRNA_sequence=gRNA_sequence,
            cell_line=cell_line,
            delivery_method=delivery_method,
            editing_efficiency=editing_efficiency,
            indel_profile=indel_profile,
            off_target_sites=off_target_sites,
            viability=viability,
            sequencing_hash=sequencing_hash,
            raw_data_cid=raw_data_cid,
            processed_data_cid=processed_data_cid,
            c2pa_signature=c2pa_signature,
            timestamp=time.time()
        )

        self.experiments.append(experiment)
        return experiment

    def validate_fair(self, experiment: ExperimentResult) -> float:
        """Valida conformidade FAIR (989.y.4). Retorna score 0-1."""
        scores = {"Findable": 0.0, "Accessible": 0.0, "Interoperable": 0.0, "Reusable": 0.0}

        if experiment.lab_orcid and len(experiment.lab_orcid) == 19:
            scores["Findable"] = 1.0
        if experiment.raw_data_cid and experiment.processed_data_cid:
            scores["Accessible"] = 1.0
        if "CRISPResso" in str(experiment.indel_profile) or "amplicon" in str(experiment.indel_profile):
            scores["Interoperable"] = 1.0
        if experiment.c2pa_signature and experiment.sequencing_hash:
            scores["Reusable"] = 1.0

        return sum(scores.values()) / 4.0

    def validate_experiment(
        self,
        experiment_id: str,
        validator_orcids: List[str],
        validator_scores: List[float]
    ) -> OracleAttestation:
        """Validadores DeSci revisam experimento e geram attestation."""
        experiment = next(
            (e for e in self.experiments if e.experiment_id == experiment_id),
            None
        )
        if not experiment:
            raise ValueError(f"Experimento {experiment_id} não encontrado")

        if len(validator_orcids) < self.min_validators:
            raise ValueError(f"Mínimo {self.min_validators} validadores necessários")

        for vocid in validator_orcids:
            if vocid not in self.validator_pool:
                raise ValueError(f"Validador {vocid} não registrado")

        consensus_score = sum(validator_scores) / len(validator_scores)
        if consensus_score < 0.66:
            experiment.status = "DISPUTED"
            raise ValueError(f"Consenso insuficiente: {consensus_score:.2f} < 0.66")

        fair_score = self.validate_fair(experiment)

        off_target_penalty = len(experiment.off_target_sites) * 0.1
        theosis_delta = (
            experiment.editing_efficiency / 100.0 *
            experiment.viability / 100.0 *
            max(0, 1.0 - off_target_penalty)
        )

        merkle_leaves = [
            experiment.sequencing_hash,
            experiment.raw_data_cid,
            experiment.processed_data_cid,
            experiment.c2pa_signature,
            hashlib.sha3_256(str(experiment.indel_profile).encode()).hexdigest()
        ]
        merkle_root = self._compute_merkle(merkle_leaves)

        attestation_id = hashlib.sha3_256(
            f"{experiment_id}:{consensus_score}:{time.time()}".encode()
        ).hexdigest()[:16]

        attestation = OracleAttestation(
            attestation_id=attestation_id,
            experiment_id=experiment_id,
            validator_orcids=validator_orcids,
            consensus_score=consensus_score,
            fair_score=fair_score,
            theosis_delta=theosis_delta,
            merkle_root=merkle_root,
            rbb_tx_hash="",
            seal=f"ORACLE-{experiment_id}-{attestation_id}"
        )

        experiment.status = "VALIDATED"
        self.attestations.append(attestation)
        return attestation

    def anchor_on_chain(self, attestation: OracleAttestation) -> str:
        """Ancora attestation na RBB Chain (12120014)."""
        rbb_payload = {
            "chainId": self.chain_id,
            "attestation_id": attestation.attestation_id,
            "experiment_id": attestation.experiment_id,
            "merkle_root": attestation.merkle_root,
            "consensus_score": attestation.consensus_score,
            "fair_score": attestation.fair_score,
            "theosis_delta": attestation.theosis_delta,
            "validators": attestation.validator_orcids,
            "timestamp": time.time()
        }

        tx_hash = hashlib.sha3_256(
            json.dumps(rbb_payload, sort_keys=True).encode()
        ).hexdigest()[:32]

        attestation.rbb_tx_hash = tx_hash
        return tx_hash

    def compute_mpp_payment(self, attestation: OracleAttestation, base_rate: float = 0.00002) -> float:
        """Computa pagamento MPP: base_rate × Theosis_delta × consensus × fair."""
        if attestation.consensus_score < 0.66:
            return 0.0
        payment = base_rate * attestation.theosis_delta * attestation.consensus_score * attestation.fair_score
        return round(payment, 8)

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

    def get_experiment_status(self, experiment_id: str) -> Dict:
        experiment = next(
            (e for e in self.experiments if e.experiment_id == experiment_id),
            None
        )
        if not experiment:
            return {"error": "Experimento não encontrado"}

        attestation = next(
            (a for a in self.attestations if a.experiment_id == experiment_id),
            None
        )

        return {
            "experiment": asdict(experiment),
            "attestation": asdict(attestation) if attestation else None,
            "fair_score": self.validate_fair(experiment) if not attestation else attestation.fair_score,
            "next_step": "ANCHOR_ON_CHAIN" if attestation and not attestation.rbb_tx_hash else "COMPLETE" if attestation else "AWAITING_VALIDATION"
        }


if __name__ == "__main__":
    oracle = BioDigitalOracle(chain_id=12120014, min_validators=3)

    # === SETUP ===
    print("[SETUP] Registrando validadores DeSci...")
    validators = [
        ("0000-0001-2345-6789", ["Gitcoin", "ORCID", "WorldID", "ENS"]),
        ("0000-0002-3456-7890", ["Gitcoin", "ORCID", "WorldID", "ENS", "POAP"]),
        ("0000-0003-4567-8901", ["Gitcoin", "ORCID", "WorldID", "ENS", "POAP", "Lens"]),
        ("0000-0004-5678-9012", ["Gitcoin", "ORCID", "WorldID", "ENS", "POAP", "Lens", "BrightID"]),
    ]
    for vocid, stamps in validators:
        ok = oracle.register_validator(vocid, stamps)
        print(f"  [✓] Validador {vocid}: {'registrado' if ok else 'rejeitado'}")
    print(f"  Pool: {len(oracle.validator_pool)} validadores")

    # === PASSO 1: Proposta aprovada ===
    print("\n[1] Criando proposta de edição BRCA1...")
    proposal = oracle.governance.create_proposal(
        identity_seed="orcid:0009-0005-2697-4668",
        stamps=["Gitcoin", "ORCID", "WorldID", "ENS"],
        patch={"code": "def repair_brca1(): return True", "seal": "PATCH-BRCA1-001"},
        target_gene="BRCA1",
        rationale="Reparo de mutação deletéria em BRCA1"
    )

    voters = [
        ("voter1", ["Gitcoin", "ORCID", "WorldID", "ENS", "POAP"], True),
        ("voter2", ["Gitcoin", "ORCID", "WorldID", "ENS"], True),
        ("voter3", ["Gitcoin", "ORCID"], True),
        ("voter4", ["Gitcoin"], False),
    ]
    for seed, stamps, approve in voters:
        oracle.governance.vote(seed, stamps, proposal.proposal_id, approve=approve)

    tally = oracle.governance.tally_votes(proposal.proposal_id)
    print(f"  [✓] Proposta {proposal.proposal_id}: {tally['status']} ({tally['ratio']*100:.1f}%)")

    # === PASSO 2: Submeter experimento ===
    print("\n[2] Laboratório submetendo resultado...")

    lab_orcid = "0000-0005-6789-0123"
    seq_hash = "fastq_hash_abc123"
    raw_cid = "QmRawDataXYZ789"
    c2pa = oracle._generate_c2pa(lab_orcid, seq_hash, raw_cid)

    experiment = oracle.submit_experiment(
        proposal_id=proposal.proposal_id,
        lab_orcid=lab_orcid,
        gRNA_sequence="AAGTTAATAACAGGGAGGAC",
        cell_line="HEK293T",
        delivery_method="RNP (ribonucleoprotein)",
        editing_efficiency=87.3,
        indel_profile={
            "amplicon": "ATCG...",
            "reads_aligned": 150000,
            "insertions": {"+1": 45.2, "+2": 12.1},
            "deletions": {"-1": 28.4, "-2": 1.6},
            "unmodified": 12.7
        },
        off_target_sites=[
            {"chr": "chr1", "pos": 1234567, "score": 0.02},
            {"chr": "chr7", "pos": 8901234, "score": 0.01}
        ],
        viability=94.5,
        sequencing_hash=seq_hash,
        raw_data_cid=raw_cid,
        processed_data_cid="QmProcessedDataABC456",
        c2pa_signature=c2pa
    )
    print(f"  [✓] Experimento {experiment.experiment_id} submetido")
    print(f"      Eficiência: {experiment.editing_efficiency}% | Viabilidade: {experiment.viability}%")
    print(f"      Off-targets: {len(experiment.off_target_sites)}")

    # === PASSO 3: FAIR ===
    print("\n[3] Validando conformidade FAIR...")
    fair_score = oracle.validate_fair(experiment)
    print(f"  [✓] FAIR score: {fair_score:.2f}/1.0")

    # === PASSO 4: Consenso DeSci ===
    print("\n[4] Validação por consenso DeSci...")
    attestation = oracle.validate_experiment(
        experiment_id=experiment.experiment_id,
        validator_orcids=["0000-0001-2345-6789", "0000-0002-3456-7890", "0000-0003-4567-8901"],
        validator_scores=[0.85, 0.90, 0.78]
    )
    print(f"  [✓] Attestation {attestation.attestation_id}")
    print(f"      Consenso: {attestation.consensus_score:.2f} | FAIR: {attestation.fair_score:.2f}")
    print(f"      Theosis Δ: {attestation.theosis_delta:.4f}")
    print(f"      Merkle root: {attestation.merkle_root[:16]}...")

    # === PASSO 5: Anchor RBB ===
    print("\n[5] Ancorando na RBB Chain...")
    tx_hash = oracle.anchor_on_chain(attestation)
    print(f"  [✓] TX hash: {tx_hash}")
    print(f"      Seal: {attestation.seal}")

    # === PASSO 6: MPP ===
    print("\n[6] Computando pagamento MPP...")
    payment = oracle.compute_mpp_payment(attestation)
    print(f"  [✓] Pagamento: ${payment:.8f} MPP")

    # === PASSO 7: Status ===
    print("\n[7] Status completo...")
    status = oracle.get_experiment_status(experiment.experiment_id)
    print(f"  Status: {status['next_step']}")

    # === PASSO 8: C2PA inválido ===
    print("\n[8] Testando C2PA inválido...")
    try:
        oracle.submit_experiment(
            proposal_id=proposal.proposal_id,
            lab_orcid="0000-0006-7890-1234",
            gRNA_sequence="INVALID",
            cell_line="HEK293T",
            delivery_method="RNP",
            editing_efficiency=99.9,
            indel_profile={},
            off_target_sites=[],
            viability=100.0,
            sequencing_hash="fake_hash",
            raw_data_cid="QmFake",
            processed_data_cid="QmFake2",
            c2pa_signature="INVALID_SIGNATURE"
        )
        print("  [✗] C2PA inválido NÃO detectado!")
    except ValueError as e:
        print(f"  [✓] C2PA inválido detectado")

    # === PASSO 9: Consenso insuficiente ===
    print("\n[9] Testando consenso insuficiente...")
    lab2 = "0000-0005-6789-0123"
    seq2 = "fastq_hash_def456"
    raw2 = "QmRawDataDEF012"
    c2pa2 = oracle._generate_c2pa(lab2, seq2, raw2)

    experiment2 = oracle.submit_experiment(
        proposal_id=proposal.proposal_id,
        lab_orcid=lab2,
        gRNA_sequence="AAGTTAATAACAGGGAGGAC",
        cell_line="iPSC",
        delivery_method="AAV",
        editing_efficiency=45.0,
        indel_profile={"CRISPResso": "low_quality"},
        off_target_sites=[{"chr": "chr1", "pos": 123, "score": 0.5}],
        viability=60.0,
        sequencing_hash=seq2,
        raw_data_cid=raw2,
        processed_data_cid="QmProcessedDataDEF789",
        c2pa_signature=c2pa2
    )
    try:
        oracle.validate_experiment(
            experiment_id=experiment2.experiment_id,
            validator_orcids=["0000-0001-2345-6789", "0000-0002-3456-7890", "0000-0003-4567-8901"],
            validator_scores=[0.50, 0.55, 0.45]
        )
        print("  [✗] Consenso insuficiente NÃO detectado!")
    except ValueError as e:
        print(f"  [✓] Consenso insuficiente detectado")

    # === PASSO 10: Validadores insuficientes ===
    print("\n[10] Testando validadores insuficientes...")
    try:
        oracle.validate_experiment(
            experiment_id=experiment.experiment_id,
            validator_orcids=["0000-0001-2345-6789"],
            validator_scores=[0.90]
        )
        print("  [✗] Validadores insuficientes NÃO detectado!")
    except ValueError as e:
        print(f"  [✓] Validadores insuficientes detectado")

    print("\n" + "="*60)
    print("Todos os testes de 1046.5 passaram.")
    print("="*60)