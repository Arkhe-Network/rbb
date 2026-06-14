import json
import logging
import time
import requests
import asyncio

try:
    import pynvml
    HAS_NVML = True
except ImportError:
    HAS_NVML = False

try:
    import psutil
    HAS_PSUTIL = True
except ImportError:
    HAS_PSUTIL = False

logging.basicConfig(level=logging.INFO, format='%(asctime)s [%(levelname)s] %(message)s')
logger = logging.getLogger("Cathedral-v17")

class PowerGovernor:
    """Monitora o consumo de energia da GPU (NVIDIA) ou CPU (fallback) para aplicar throttling."""
    def __init__(self, target_power_w=300, mode="performance"):
        self.target_power_w = target_power_w
        self.mode = mode  # "performance" ou "eco"
        self.nvml_initialized = False

        if HAS_NVML:
            try:
                pynvml.nvmlInit()
                self.handle = pynvml.nvmlDeviceGetHandleByIndex(0)
                self.nvml_initialized = True
                logger.info("PowerGovernor: NVIDIA NVML inicializado.")
            except Exception as e:
                logger.warning(f"PowerGovernor: Falha ao inicializar NVML: {e}")

        if not self.nvml_initialized:
            logger.warning("PowerGovernor: Usando fallback (psutil para CPU). Leitura de GPU indisponível.")

    def get_current_power_w(self):
        if self.nvml_initialized:
            try:
                power_mw = pynvml.nvmlDeviceGetPowerUsage(self.handle)
                return power_mw / 1000.0
            except Exception:
                pass

        # Fallback (simulação baseada na CPU)
        if HAS_PSUTIL:
            cpu_percent = psutil.cpu_percent()
            # Estimativa grosseira: base 50W + ate 100W baseado em uso de CPU
            return 50.0 + (cpu_percent / 100.0) * 100.0

        return 100.0 # Valor default

    def enforce_limits(self):
        current_w = self.get_current_power_w()
        if self.mode == "eco" or current_w > self.target_power_w:
            logger.warning(f"PowerGovernor: Consumo {current_w:.1f}W excede limite {self.target_power_w}W ou modo eco ativo. Aplicando throttling (sleep 1s).")
            time.sleep(1.0)
        return current_w


class NexRLDataCollector:
    """Coleta dados para fine-tuning (DPO) simulando o NexRLDataCollector."""
    def __init__(self, log_file="v17_dpo_dataset.jsonl"):
        self.log_file = log_file

    def log_interaction(self, prompt, chosen_response, rejected_response=None, reward=0.0):
        data = {
            "prompt": prompt,
            "chosen": chosen_response,
            "rejected": rejected_response if rejected_response else "",
            "reward": reward,
            "timestamp": time.time()
        }
        with open(self.log_file, "a", encoding="utf-8") as f:
            f.write(json.dumps(data) + "\n")
        logger.info("NexRLDataCollector: Interação registrada para DPO.")


class PhysicsToTextBridge:
    """Traduz o estado físico do MuJoCo/Sensor para texto, para o Slow Brain."""
    def __init__(self):
        pass

    def translate(self, qpos, qvel=None, power_w=None):
        text = "Estado Físico Atual:\n"
        if qpos:
            text += f"- Posição (qpos): {qpos}\n"
        if qvel:
            text += f"- Velocidade (qvel): {qvel}\n"
        if power_w is not None:
            text += f"- Consumo de Energia: {power_w:.1f} W\n"
        text += "Avalie se as condições indicam estabilidade ou risco (ex: consumo excessivo ou instabilidade posicional)."
        return text


class SlowBrainClient:
    """Cliente para a API do SGLang rodando no WSL2."""
    def __init__(self, url="http://127.0.0.1:8000/v1/chat/completions"):
        self.url = url

    async def ask(self, prompt, max_tokens=256):
        payload = {
            "model": "default", # O SGLang mapeia isso internamente
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": max_tokens,
            "temperature": 0.2
        }
        try:
            # Em um cenário real, usaria aiohttp. Usando requests blockante simulado em async para simplicidade local.
            response = requests.post(self.url, json=payload, timeout=5.0)
            if response.status_code == 200:
                data = response.json()
                return data.get("choices", [{}])[0].get("message", {}).get("content", "").strip()
            else:
                logger.error(f"SlowBrainClient: Erro da API: {response.status_code}")
                return "ERRO_SGLANG"
        except requests.exceptions.RequestException as e:
            logger.error(f"SlowBrainClient: Erro de conexão com SGLang: {e}")
            return "SGLANG_INDISPONIVEL"


class OrchestratorV17:
    def __init__(self):
        self.power_gov = PowerGovernor(target_power_w=200, mode="performance")
        self.data_collector = NexRLDataCollector()
        self.phys_bridge = PhysicsToTextBridge()
        self.slow_brain = SlowBrainClient()

    async def run_cycle(self, qpos, qvel):
        logger.info("--- Iniciando Ciclo do Fast Brain ---")

        # 1. Medir energia e aplicar limites (PowerGovernor)
        current_w = self.power_gov.enforce_limits()

        # 2. Traduzir física para texto (Abstraction Barrier)
        physical_context = self.phys_bridge.translate(qpos, qvel, current_w)
        logger.info(f"Contexto Físico:\n{physical_context}")

        # 3. Chamar Slow Brain (se necessário, simulando gatilho de risco)
        # Vamos chamar o Slow Brain sempre neste exemplo para demonstrar a coleta.
        prompt = physical_context + "\n\nO que devemos fazer? Responda com 'MANTER' ou 'PARAR'."

        logger.info("Consultando Slow Brain (SGLang WSL2)...")
        slow_response = await self.slow_brain.ask(prompt)
        logger.info(f"Resposta Slow Brain: {slow_response}")

        # 4. Avaliar e registrar para DPO
        # Simulação de recompensa baseada na resposta
        if "PARAR" in slow_response.upper() and current_w > 180:
            reward = 1.0 # Ação correta de segurança
        elif "MANTER" in slow_response.upper() and current_w <= 180:
            reward = 1.0 # Ação correta de rotina
        else:
            reward = -1.0 # Ação subótima

        # Registra a iteração (o DPO precisa de exemplos bons e ruins, aqui estamos simulando a coleta)
        self.data_collector.log_interaction(
            prompt=prompt,
            chosen_response=slow_response if reward > 0 else "MANTER" if "PARAR" in slow_response else "PARAR",
            rejected_response=slow_response if reward <= 0 else "MANTER" if "PARAR" in slow_response else "PARAR",
            reward=reward
        )

        return {"status": "ok", "power": current_w, "slow_decision": slow_response}


async def main():
    print("Iniciando Cathedral AGI v17 Orchestrator (Teste Mínimo)")
    orchestrator = OrchestratorV17()

    # Simulação de MuJoCo / Sensores
    dummy_qpos = [0.1, 0.2, -0.1]
    dummy_qvel = [0.01, 0.05, 0.0]

    for i in range(3):
        print(f"\nCiclo {i+1}")
        result = await orchestrator.run_cycle(dummy_qpos, dummy_qvel)
        print(f"Resultado: {result}")
        await asyncio.sleep(1)

if __name__ == "__main__":
    asyncio.run(main())
