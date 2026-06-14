import time
import json
import logging
import statistics
from cathedral_arkhe_v12_1_2_agi import CathedralOrchestratorV12_1_2

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
logger = logging.getLogger("sovereign_benchmark")

class SovereignBenchmarkIntegrated:
    def __init__(self, num_iterations=10):
        self.num_iterations = num_iterations
        self.results = {
            "federation": {"latency_ms": [], "corte_count": 0, "telemetry_logs": []}
        }
        self.orch = CathedralOrchestratorV12_1_2()

    def simulate_federation_tasks(self):
        prompts = [
            "What is the mathematical proof of the Riemann Hypothesis?",
            "Explique as nuances legais do Protocolo de Corte 294 no ecossistema Cathedral.",
            "Write a Python script that integrates with WormGraph and OpenTelemetry.",
            "Descreva as camadas da arquitetura do Cathedral AGI Omega.",
            "Generate a summary of the Federation's governance model."
        ]

        logger.info(f"Iniciando Sovereign Benchmark na Federação. {self.num_iterations} ciclos...")

        for i in range(self.num_iterations):
            prompt = prompts[i % len(prompts)]
            start_time = time.time()

            try:
                res = self.orch.infer(prompt)
            except Exception as e:
                logger.error(f"Inference error: {e}")
                continue

            latency = (time.time() - start_time) * 1000
            self.results["federation"]["latency_ms"].append(latency)

            telemetry = self.orch.get_telemetry()
            self.results["federation"]["telemetry_logs"].append(telemetry)

            # Check Corte Status from telemetry
            if telemetry.get("corte", {}).get("state") != "INACTIVE":
                self.results["federation"]["corte_count"] += 1

    def generate_report(self):
        fed = self.results["federation"]
        latencies = fed["latency_ms"]

        if not latencies:
            logger.error("Sem dados de latência para o report.")
            return

        avg_fed_lat = statistics.mean(latencies)
        p95_lat = statistics.quantiles(latencies, n=20)[18] if len(latencies) > 1 else avg_fed_lat

        report_data = {
            "Benchmark_Config": {"iterations": self.num_iterations},
            "Federation_Avg_Latency_ms": round(avg_fed_lat, 2),
            "Federation_p95_Latency_ms": round(p95_lat, 2),
            "Federation_Corte_Quarantine_Count": fed["corte_count"],
        }

        print("\n" + "="*40)
        print(" SOVEREIGN BENCHMARK INTEGRATED RESULTS")
        print("="*40)
        print(json.dumps(report_data, indent=4))
        print("="*40 + "\n")

if __name__ == "__main__":
    benchmark = SovereignBenchmarkIntegrated(num_iterations=5)
    benchmark.simulate_federation_tasks()
    benchmark.generate_report()
