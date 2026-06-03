import hashlib
import time
from dataclasses import dataclass
from typing import List

@dataclass
class GeneticEditProposal:
    proposal_id: str
    target_gene: str

class PassportIdentity:
    def __init__(self, stamps):
        self.credential_count = len(stamps)

class BioDigitalGovernanceBridge:
    def __init__(self, chain_id: int):
        self.chain_id = chain_id
        self.proposals: List[GeneticEditProposal] = []
        self._votes = {}

    def verify_passport(self, orcid: str, stamps: List[str]) -> PassportIdentity:
        return PassportIdentity(stamps)

    def create_proposal(self, identity_seed: str, stamps: List[str], patch: dict, target_gene: str, rationale: str) -> GeneticEditProposal:
        proposal_id = "prop_" + hashlib.sha256(f"{identity_seed}{target_gene}{time.time()}".encode()).hexdigest()[:16]
        proposal = GeneticEditProposal(proposal_id=proposal_id, target_gene=target_gene)
        self.proposals.append(proposal)
        self._votes[proposal_id] = []
        return proposal

    def vote(self, seed: str, stamps: List[str], proposal_id: str, approve: bool):
        if proposal_id in self._votes:
            self._votes[proposal_id].append(approve)

    def tally_votes(self, proposal_id: str) -> dict:
        if proposal_id not in self._votes:
            return {"status": "REJECTED", "ratio": 0.0}
        votes = self._votes[proposal_id]
        if not votes:
            return {"status": "APPROVED", "ratio": 1.0}
        approve_count = sum(1 for v in votes if v)
        ratio = approve_count / len(votes)
        status = "APPROVED" if ratio > 0.5 else "REJECTED"
        return {"status": status, "ratio": ratio}
