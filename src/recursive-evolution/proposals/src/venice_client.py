import requests
import structlog

logger = structlog.get_logger()

class VeniceClient:
    def __init__(self, api_key: str, base_url: str = "https://api.venice.ai/v1"):
        self.api_key = api_key
        self.base_url = base_url
        self.session = requests.Session()
        self.session.headers.update({
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json"
        })

    def infer(self, prompt: str, max_tokens: int = 4096, temperature: float = 0.7) -> str:
        payload = {
            "model": "claude-3.5-sonnet",
            "prompt": prompt,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "stream": False
        }
        try:
            response = self.session.post(
                f"{self.base_url}/completions",
                json=payload,
                timeout=30.0
            )
            response.raise_for_status()
            data = response.json()
            return data.get("choices", [{}])[0].get("text", "")
        except Exception as e:
            logger.error("Venice API request failed", error=str(e))
            raise
