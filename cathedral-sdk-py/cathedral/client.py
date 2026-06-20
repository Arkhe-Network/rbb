# cathedral-sdk-py/cathedral/client.py
# SDK Python para agentes Prometheus — leve, assíncrono, com batching e governança.

import asyncio
import json
import uuid
from dataclasses import dataclass, asdict
from enum import Enum
from typing import Any, Dict, List, Optional, Callable, Awaitable
import aiohttp
import logging

logger = logging.getLogger("cathedral-sdk")

# ============================================================
# TIPOS DE EVENTO
# ============================================================

class EventType(str, Enum):
    DESIGN_PROPOSED = "design_proposed"
    SIMULATION_COMPLETED = "simulation_completed"
    DESIGN_OPTIMIZED = "design_optimized"
    FABRICATION_PLANNED = "fabrication_planned"
    FABRICATION_COMPLETED = "fabrication_completed"
    TEST_RESULT = "test_result"
    HUMAN_REVIEW = "human_review"
    AGENT_MUTATION = "agent_mutation"

class HumanVerdict(str, Enum):
    APPROVED = "approved"
    CONDITIONAL = "conditional"
    REJECTED = "rejected"

class GovernanceMode(str, Enum):
    HUMAN_IN_THE_LOOP = "human_in_the_loop"
    AUTONOMOUS_WITH_CIRCUIT_BREAKER = "autonomous_with_circuit_breaker"
    AUTONOMOUS_FULL = "autonomous_full"

@dataclass
class DesignProposedEvent:
    design_hash: str
    parent_hashes: List[str]
    parameters: Dict[str, float]
    rationale: str
    agent_id: str

@dataclass
class SimulationCompletedEvent:
    design_hash: str
    simulator: str
    metrics: Dict[str, float]
    convergence: bool
    compute_cost_usd: float

@dataclass
class AgentMutationEvent:
    mutation_description: str
    previous_agent_hash: str
    substrate_version: str

@dataclass
class GovernanceResponse:
    verdict: str  # approved, rejected, requires_human, conditional, timeout
    rationale: str
    conditions: Optional[List[str]] = None

# ============================================================
# CLIENTE PRINCIPAL
# ============================================================

class CathedralClient:
    def __init__(
        self,
        bridge_endpoint: str = "http://localhost:8080",
        project_id: str = "default",
        agent_id: str = "default-agent",
        batch_size: int = 50,
        flush_interval_ms: int = 5000,
        governance_mode: GovernanceMode = GovernanceMode.AUTONOMOUS_WITH_CIRCUIT_BREAKER,
        loop: Optional[asyncio.AbstractEventLoop] = None,
    ):
        self.bridge_endpoint = bridge_endpoint
        self.project_id = project_id
        self.agent_id = agent_id
        self.batch_size = batch_size
        self.flush_interval_ms = flush_interval_ms
        self.governance_mode = governance_mode
        self.loop = loop or asyncio.get_event_loop()

        self._event_queue: List[Dict[str, Any]] = []
        self._flush_task: Optional[asyncio.Task] = None
        self._session: Optional[aiohttp.ClientSession] = None
        self._running = False

    async def __aenter__(self):
        self._session = aiohttp.ClientSession()
        self._running = True
        self._flush_task = asyncio.create_task(self._background_flusher())
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        self._running = False
        if self._flush_task:
            await self._flush_task
        if self._session:
            await self._session.close()

    # ============================================================
    # EMISSÃO DE EVENTOS (PÚBLICO)
    # ============================================================

    async def emit_design_proposed(
        self,
        design_hash: str,
        parent_hashes: List[str],
        parameters: Dict[str, float],
        rationale: str,
    ) -> None:
        event = DesignProposedEvent(
            design_hash=design_hash,
            parent_hashes=parent_hashes,
            parameters=parameters,
            rationale=rationale,
            agent_id=self.agent_id,
        )
        await self._emit_event(EventType.DESIGN_PROPOSED, asdict(event))

    async def emit_simulation_completed(
        self,
        design_hash: str,
        simulator: str,
        metrics: Dict[str, float],
        convergence: bool,
        compute_cost_usd: float,
    ) -> None:
        event = SimulationCompletedEvent(
            design_hash=design_hash,
            simulator=simulator,
            metrics=metrics,
            convergence=convergence,
            compute_cost_usd=compute_cost_usd,
        )
        await self._emit_event(EventType.SIMULATION_COMPLETED, asdict(event))

    async def emit_agent_mutation(
        self,
        mutation_description: str,
        previous_agent_hash: str,
    ) -> None:
        event = AgentMutationEvent(
            mutation_description=mutation_description,
            previous_agent_hash=previous_agent_hash,
            substrate_version="5003.8",
        )
        # Mutação de agente requer governança síncrona
        response = await self.request_governance(EventType.AGENT_MUTATION, asdict(event))
        if response.verdict == "rejected":
            raise RuntimeError(f"Agent mutation rejected: {response.rationale}")
        if response.verdict == "requires_human":
            logger.warning(f"Agent mutation requires human review: {response.rationale}")
        if response.verdict == "conditional":
            logger.info(f"Agent mutation approved with conditions: {response.conditions}")
        # Emite o evento mesmo assim (já que foi aprovado condicionalmente)
        await self._emit_event(EventType.AGENT_MUTATION, asdict(event))

    # ============================================================
    # GOVERNANÇA SÍNCRONA
    # ============================================================

    async def request_governance(self, event_type: EventType, payload: Dict[str, Any]) -> GovernanceResponse:
        if self.governance_mode == GovernanceMode.AUTONOMOUS_FULL:
            return GovernanceResponse(verdict="approved", rationale="Autonomous full mode")

        if self.governance_mode == GovernanceMode.AUTONOMOUS_WITH_CIRCUIT_BREAKER:
            risk = self._estimate_risk(event_type)
            if risk < 0.5:
                return GovernanceResponse(verdict="approved", rationale="Low risk decision")

        # Chamada síncrona ao serviço de governança
        if not self._session:
            self._session = aiohttp.ClientSession()

        request_id = str(uuid.uuid4())
        req_body = {
            "request_id": request_id,
            "project_id": self.project_id,
            "agent_id": self.agent_id,
            "event_type": event_type.value,
            "payload": payload,
            "timestamp": int(asyncio.get_event_loop().time() * 1e9),
        }

        try:
            async with self._session.post(
                f"{self.bridge_endpoint}/governance",
                json=req_body,
                timeout=aiohttp.ClientTimeout(total=5.0),
            ) as resp:
                if resp.status != 200:
                    text = await resp.text()
                    return GovernanceResponse(
                        verdict="rejected",
                        rationale=f"Governance service error: {text}",
                    )
                data = await resp.json()
                return GovernanceResponse(
                    verdict=data.get("verdict", "rejected"),
                    rationale=data.get("rationale", "No rationale provided"),
                    conditions=data.get("conditions"),
                )
        except asyncio.TimeoutError:
            return GovernanceResponse(
                verdict="timeout",
                rationale="Governance request timed out",
            )
        except Exception as e:
            logger.exception("Governance request failed")
            return GovernanceResponse(
                verdict="rejected",
                rationale=f"Governance request failed: {e}",
            )

    # ============================================================
    # INTERNO
    # ============================================================

    def _estimate_risk(self, event_type: EventType) -> float:
        risk_map = {
            EventType.AGENT_MUTATION: 0.85,
            EventType.FABRICATION_PLANNED: 0.70,
            EventType.SIMULATION_COMPLETED: 0.30,
            EventType.DESIGN_PROPOSED: 0.20,
        }
        return risk_map.get(event_type, 0.10)

    async def _emit_event(self, event_type: EventType, payload: Dict[str, Any]) -> None:
        entry = {
            "event_type": event_type.value,
            "timestamp_ns": int(asyncio.get_event_loop().time() * 1e9),
            "payload": payload,
        }
        self._event_queue.append(entry)
        if len(self._event_queue) >= self.batch_size:
            await self._flush()

    async def _flush(self) -> None:
        if not self._event_queue or not self._session:
            return

        batch = self._event_queue[:]
        self._event_queue.clear()

        body = {
            "project_id": self.project_id,
            "agent_id": self.agent_id,
            "events": batch,
        }

        try:
            async with self._session.post(
                f"{self.bridge_endpoint}/ingest",
                json=body,
                timeout=aiohttp.ClientTimeout(total=5.0),
            ) as resp:
                if resp.status == 200:
                    logger.debug(f"Batch of {len(batch)} events sent successfully")
                else:
                    logger.error(f"Failed to send batch: {resp.status}")
        except Exception as e:
            logger.exception("Failed to send batch")
            # Em produção, poderíamos colocar de volta na fila ou persistir localmente

    async def _background_flusher(self) -> None:
        while self._running:
            await asyncio.sleep(self.flush_interval_ms / 1000.0)
            if self._event_queue:
                await self._flush()

# ============================================================
# EXEMPLO DE USO
# ============================================================

async def example_usage():
    async with CathedralClient(
        bridge_endpoint="http://localhost:8080",
        project_id="jet-engine-v4",
        agent_id="prometheus-agent-007",
        governance_mode=GovernanceMode.AUTONOMOUS_WITH_CIRCUIT_BREAKER,
    ) as client:
        await client.emit_design_proposed(
            design_hash="abc123",
            parent_hashes=["parent1", "parent2"],
            parameters={"lattice_constant": 360.0, "thickness": 2.5},
            rationale="Literature suggests 360nm for optimal Q-factor",
        )
        await client.emit_simulation_completed(
            design_hash="abc123",
            simulator="FDTD-Meep",
            metrics={"Q_factor": 12000.0, "bandgap": 0.065},
            convergence=True,
            compute_cost_usd=45.0,
        )
        # Mutação de agente (requer governança)
        try:
            await client.emit_agent_mutation(
                mutation_description="Swap optimizer from Bayesian to Genetic",
                previous_agent_hash="agent-v1.2",
            )
        except RuntimeError as e:
            print(f"Mutation blocked: {e}")
