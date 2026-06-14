#!/usr/bin/env python3
"""
Cathedral ARKHE v17.2 – zkHydra Gateway
Orquestra análise de segurança de circuitos ZK (Circom) usando o framework zkHydra.
"""

import asyncio
import json
import logging
import subprocess
from pathlib import Path
from typing import List, Dict, Optional

logger = logging.getLogger("cathedral.zkhydra")

class ZkHydraResult:
    """Estrutura para resultados de análise do zkHydra."""
    def __init__(self, tool_name: str, raw_output: dict):
        self.tool_name = tool_name
        self.has_findings = raw_output.get("findings", []) != []
        self.findings = raw_output.get("findings", [])

class ZkHydraGateway:
    """
    Gateway para o framework zkHydra.
    Executa análises de segurança sobre circuitos Circom e retorna resultados estruturados.
    """

    def __init__(self, work_dir: str = "./zkhydra_work"):
        self.work_dir = Path(work_dir)
        self.work_dir.mkdir(parents=True, exist_ok=True)

    async def analyze_circuit(
        self,
        circuit_path: str,
        tools: List[str] = None,
        timeout: int = 600
    ) -> Dict[str, ZkHydraResult]:
        """
        Executa o zkHydra (modo analyze) sobre um arquivo .circom.

        Args:
            circuit_path: Caminho para o arquivo .circom.
            tools: Lista de ferramentas a usar (ex: ["circomspect", "circom_civer"]).
            timeout: Tempo limite em segundos.

        Returns:
            Dicionário mapeando nome da ferramenta para ZkHydraResult.
        """
        if tools is None:
            tools = ["circomspect", "circom_civer", "picus"]

        output_dir = self.work_dir / "output"
        output_dir.mkdir(exist_ok=True)

        cmd = [
            "docker", "run", "--rm",
            "-v", f"{Path(circuit_path).parent.resolve()}:/zkhydra/input",
            "-v", f"{output_dir.resolve()}:/zkhydra/output",
            "ghcr.io/zksecurity/zkhydra:latest",
            "uv", "run", "python", "-m", "zkhydra.main", "analyze",
            "--input", f"/zkhydra/input/{Path(circuit_path).name}",
            "--tools", ",".join(tools),
            "--timeout", str(timeout),
            "--output", "/zkhydra/output"
        ]

        try:
            proc = await asyncio.create_subprocess_exec(
                *cmd,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            stdout, stderr = await proc.communicate()
            if proc.returncode != 0:
                raise RuntimeError(f"zkHydra falhou (RC={proc.returncode}): {stderr.decode()}")

            results = {}
            for tool in tools:
                result_file = output_dir / tool / "results.json"
                if result_file.exists():
                    with open(result_file, "r") as f:
                        raw = json.load(f)
                        results[tool] = ZkHydraResult(tool, raw)
                else:
                    logger.warning(f"Resultado da ferramenta {tool} não encontrado em {result_file}")
            return results

        except Exception as e:
            logger.error(f"Erro ao executar zkHydra: {e}")
            return {}

    async def evaluate_vulnerability(self, bug_config_path: str, tools: List[str] = None) -> Dict:
        """
        Executa o modo 'evaluate' do zkHydra, comparando resultados com vulnerabilidades conhecidas (zkbugs).
        Útil para benchmarking e validação de novas ferramentas.
        """
        if tools is None:
            tools = ["circomspect", "circom_civer", "picus", "zkfuzz"]

        output_dir = self.work_dir / "evaluate_output"
        output_dir.mkdir(exist_ok=True)

        cmd = [
            "docker", "run", "--rm",
            "-v", f"{Path(bug_config_path).parent.resolve()}:/zkhydra/bug",
            "-v", f"{output_dir.resolve()}:/zkhydra/output",
            "ghcr.io/zksecurity/zkhydra:latest",
            "uv", "run", "python", "-m", "zkhydra.main", "evaluate",
            "--input", f"/zkhydra/bug/{Path(bug_config_path).name}",
            "--tools", ",".join(tools),
            "--output", "/zkhydra/output"
        ]

        try:
            proc = await asyncio.create_subprocess_exec(*cmd, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)
            stdout, stderr = await proc.communicate()
            if proc.returncode != 0:
                raise RuntimeError(f"zkHydra evaluate falhou: {stderr.decode()}")

            eval_file = output_dir / "evaluation.json"
            if eval_file.exists():
                with open(eval_file, "r") as f:
                    return json.load(f)
            else:
                logger.warning("Arquivo evaluation.json não encontrado")
                return {}
        except Exception as e:
            logger.error(f"Erro no evaluate: {e}")
            return {}

async def process_zkhydra_results(results: Dict[str, ZkHydraResult]) -> None:
    for tool, res in results.items():
        if res.has_findings:
            logger.warning(f"[{tool}] Vulnerabilidades detectadas: {res.findings}")
            # await send_alert(...) # omitted for brevity
        else:
            logger.info(f"[{tool}] Nenhuma vulnerabilidade encontrada.")

if __name__ == "__main__":
    import argparse
    import sys

    parser = argparse.ArgumentParser(description="zkHydra Gateway")
    parser.add_argument("mode", choices=["analyze", "evaluate"], help="Mode of operation")
    parser.add_argument("--circuit", help="Path to .circom circuit file")
    parser.add_argument("--tools", default="all", help="Comma-separated list of tools or 'all'")
    parser.add_argument("--fail-on-finding", action="store_true", help="Exit with code 1 if findings exist")
    parser.add_argument("--output", help="Output JSON file for results")

    args = parser.parse_args()

    logging.basicConfig(level=logging.INFO)

    async def main():
        gateway = ZkHydraGateway()
        tools = None if args.tools == "all" else args.tools.split(",")

        if args.mode == "analyze":
            if not args.circuit:
                print("Error: --circuit is required for analyze mode")
                sys.exit(1)
            results = await gateway.analyze_circuit(args.circuit, tools)
            await process_zkhydra_results(results)

            if args.output:
                with open(args.output, "w") as f:
                    # simplistic serialization
                    out = {k: {"tool_name": v.tool_name, "has_findings": v.has_findings, "findings": v.findings} for k,v in results.items()}
                    json.dump(out, f, indent=2)

            if args.fail_on_finding and any(r.has_findings for r in results.values()):
                sys.exit(1)

        elif args.mode == "evaluate":
            # Just an example implementation, omitted for analyze focus
            pass

    asyncio.run(main())
