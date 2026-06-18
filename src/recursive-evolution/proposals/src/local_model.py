import os
import structlog
import requests

logger = structlog.get_logger()

class LocalModel:
    def __init__(self, model_name: str = "llama3:8b"):
        self.model_name = model_name
        self.use_ollama = os.getenv("USE_OLLAMA", "true").lower() == "true"

    def generate(self, prompt: str) -> str:
        if self.use_ollama:
            return self._ollama_generate(prompt)
        else:
            return self._transformers_generate(prompt)

    def _ollama_generate(self, prompt: str) -> str:
        try:
            response = requests.post(
                "http://localhost:11434/api/generate",
                json={"model": self.model_name, "prompt": prompt, "stream": False},
                timeout=60.0
            )
            response.raise_for_status()
            return response.json().get("response", "")
        except Exception as e:
            logger.error("Ollama generation failed", error=str(e))
            return "[ERROR] Local model unavailable"

    def _transformers_generate(self, prompt: str) -> str:
        # Lazy loading do modelo
        if not hasattr(self, "_model"):
            from transformers import AutoModelForCausalLM, AutoTokenizer
            self._tokenizer = AutoTokenizer.from_pretrained(self.model_name)
            self._model = AutoModelForCausalLM.from_pretrained(self.model_name)
        inputs = self._tokenizer(prompt, return_tensors="pt")
        outputs = self._model.generate(**inputs, max_new_tokens=1024)
        return self._tokenizer.decode(outputs[0], skip_special_tokens=True)
