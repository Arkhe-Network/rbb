import structlog
from typing import Optional, Dict, Any
from tenacity import retry, stop_after_attempt, wait_exponential
from src.venice_client import VeniceClient
from src.bittensor_client import BittensorClient
from src.local_model import LocalModel
from src.prompt_templates import get_prompt
from src.validators import validate_and_parse_output

logger = structlog.get_logger()

class InferenceOrchestrator:
    def __init__(self, venice_api_key: str, bittensor_network: str, use_local_fallback: bool = True):
        self.venice = VeniceClient(api_key=venice_api_key)
        self.bittensor = BittensorClient(network=bittensor_network)
        self.local_model = LocalModel() if use_local_fallback else None
        self.use_local_fallback = use_local_fallback

    @retry(stop=stop_after_attempt(3), wait=wait_exponential(multiplier=1, min=2, max=10))
    def _call_venice(self, prompt: str, max_tokens: int = 4096) -> str:
        return self.venice.infer(prompt, max_tokens=max_tokens)

    @retry(stop=stop_after_attempt(2), wait=wait_exponential(multiplier=1, min=2, max=10))
    def _call_bittensor(self, prompt: str, subnet: str = "SN96") -> str:
        return self.bittensor.infer(prompt, subnet=subnet)

    def _call_local(self, prompt: str) -> str:
        if not self.local_model:
            raise RuntimeError("Local model not available")
        return self.local_model.generate(prompt)

    def generate(self, task_type: str, context: Dict[str, Any], code: Optional[str] = None,
                 language: Optional[str] = None, target_language: Optional[str] = None) -> Dict[str, Any]:
        prompt = get_prompt(task_type, context, code, language, target_language)

        # Tenta Venice
        try:
            logger.info("Calling Venice API", task_type=task_type)
            result_text = self._call_venice(prompt)
            parsed = validate_and_parse_output(result_text, task_type)
            if parsed:
                parsed["source"] = "venice"
                return parsed
        except Exception as e:
            logger.warning("Venice API failed", error=str(e))

        # Tenta Bittensor
        try:
            subnet = "SN96" if task_type in ["general", "architecture_improvement"] else "SN92"
            logger.info("Calling Bittensor subnet", subnet=subnet)
            result_text = self._call_bittensor(prompt, subnet=subnet)
            parsed = validate_and_parse_output(result_text, task_type)
            if parsed:
                parsed["source"] = f"bittensor_{subnet}"
                return parsed
        except Exception as e:
            logger.warning("Bittensor inference failed", error=str(e))

        # Fallback local
        if self.use_local_fallback and self.local_model:
            logger.info("Using local model as fallback")
            result_text = self._call_local(prompt)
            parsed = validate_and_parse_output(result_text, task_type)
            if parsed:
                parsed["source"] = "local"
                return parsed

        raise RuntimeError("All inference sources failed")

    def generate_fix(self, finding: Dict[str, Any], code: str, language: str) -> Dict[str, Any]:
        return self.generate(
            task_type="security_fix",
            context={"vulnerability": finding},
            code=code,
            language=language
        )
