import requests
import structlog
from typing import Dict, Any

logger = structlog.get_logger()

class GovernanceIntegration:
    def __init__(self, orchestrator_url: str):
        self.orchestrator_url = orchestrator_url

    def submit_proposal(self, proposal: Dict[str, Any]) -> bool:
        try:
            response = requests.post(
                f"{self.orchestrator_url}/api/v1/proposals",
                json=proposal,
                timeout=5.0
            )
            if response.status_code in (200, 201):
                logger.info("Proposal submitted", proposal=proposal.get("title"))
                return True
        except Exception as e:
            logger.error("Failed to submit proposal", error=str(e))
        return False
