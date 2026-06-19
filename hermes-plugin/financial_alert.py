#!/usr/bin/env python3
"""hermes-plugin/financial_alert.py
Plugin para Hermes: alertas de câmbio e relatórios financeiros.

Selo: CATHEDRAL-ARKHE-HERMES-FINANCIAL-v1.0.0-2026-06-19
"""

import asyncio
import httpx
import structlog

logger = structlog.get_logger(__name__)

class FinancialAlertPlugin:
    """Plugin para monitoramento cambial e notificações."""

    def __init__(self, config: dict):
        self.config = config
        self.cathedral_url = config.get("cathedral_url", "http://localhost:8787")
        self.target_currency = config.get("target_currency", "USD")
        self.threshold = config.get("alert_threshold", 0.05)  # 5% de variação
        self.last_rate = None
        self.hermes = None  # Será injetado pelo Hermes

    async def on_message(self, message):
        """Processa comandos financeiros no chat."""
        text = message.get("text", "")
        if text.startswith("/cambio"):
            # Responde com cotação atual
            rate = await self.get_exchange_rate()
            await self.hermes.send_message(
                chat_id=message["chat_id"],
                text=f"💰 Cotação {self.target_currency}/BRL: {rate:.4f}"
            )
        elif text.startswith("/relatorio"):
            # Gera relatório de compliance
            cnpj = text.split()[1] if len(text.split()) > 1 else ""
            if not cnpj:
                await self.hermes.send_message(
                    chat_id=message["chat_id"],
                    text="⚠️ Use /relatorio <CNPJ> para gerar relatório."
                )
            else:
                report = await self.generate_compliance_report(cnpj)
                await self.hermes.send_message(
                    chat_id=message["chat_id"],
                    text=report[:4000]  # Limita para não estourar limite
                )

    async def get_exchange_rate(self) -> float:
        """Obtém cotação via API do BC ou mercado."""
        # Em produção: chamar API do Banco Central
        return 5.50  # Simula

    async def generate_compliance_report(self, cnpj: str) -> str:
        """Gera relatório via agente financeiro."""
        async with httpx.AsyncClient() as client:
            resp = await client.post(
                f"{self.cathedral_url}/mcp/financial",
                json={
                    "jsonrpc": "2.0",
                    "method": "financial_compliance_report",
                    "params": {"cnpj": cnpj, "days": 30},
                }
            )
            data = resp.json()
            return data.get("result", {}).get("report", "Relatório não disponível.")

    async def monitor_exchange_rates(self):
        """Loop de monitoramento cambial (executado em background)."""
        logger.info("Iniciando monitoramento cambial...")
        while True:
            try:
                rate = await self.get_exchange_rate()
                if self.last_rate is not None:
                    variation = abs(rate - self.last_rate) / self.last_rate
                    if variation > self.threshold:
                        # Alerta via Hermes
                        await self.hermes.send_broadcast(
                            text=f"⚠️ Variação cambial detectada: {variation*100:.2f}% em {self.target_currency}/BRL"
                        )
                self.last_rate = rate
            except Exception as e:
                logger.error("Erro no monitoramento cambial", error=str(e))
            await asyncio.sleep(3600)  # A cada hora
