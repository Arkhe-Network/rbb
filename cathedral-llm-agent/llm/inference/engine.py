try:
    from vllm import LLM, SamplingParams
    VLLM_AVAILABLE = True
except ImportError:
    VLLM_AVAILABLE = False

class CathedralLLMEngine:
    def __init__(self, model_path: str = None):
        if VLLM_AVAILABLE and model_path:
            self.llm = LLM(model=model_path, tensor_parallel_size=1)
            self.use_vllm = True
        else:
            self.llm = None
            self.use_vllm = False

    async def chat(self, messages, temperature=0.7, max_tokens=1024):
        if self.use_vllm:
            prompt = "\n".join([f"{m['role']}: {m['content']}" for m in messages])
            sampling = SamplingParams(temperature=temperature, max_tokens=max_tokens)
            outputs = self.llm.generate([prompt], sampling)
            return outputs[0].outputs[0].text
        else:
            # Stub
            return "Thought: I should use a tool.\nAction: picoads\nAction Input: {\"user_context_hash\": \"test\"}"