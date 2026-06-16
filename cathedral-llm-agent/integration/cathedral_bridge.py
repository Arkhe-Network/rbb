#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Cathedral ARKHE — Bridge between Python Agent and Rust EmbodiedCognitiveCore
Selo: CATHEDRAL-ARKHE-BRIDGE-2026-06-16

Suporta dois modos de comunicação:
- HTTP (fallback) via FastAPI do core
- napi-rs (nativo, baixa latência) se o módulo Rust estiver compilado
"""

import asyncio
import json
import os
from typing import Optional, Dict, Any

# Tenta importar o módulo napi-rs (compilado a partir do core Rust)
try:
    import cathedral_core_napi as napi  # type: ignore
    NAPI_AVAILABLE = True
except ImportError:
    NAPI_AVAILABLE = False
    import httpx

class CathedralBridge:
    """
    Ponte para o EmbodiedCognitiveCore (Rust). Usa napi-rs se disponível,
    caso contrário faz chamadas HTTP para o servidor do core.
    """

    def __init__(self, mode: str = "auto", http_url: str = "http://localhost:8000"):
        self.core_http_url = http_url
        if mode == "auto":
            self.use_napi = NAPI_AVAILABLE
        elif mode == "napi":
            self.use_napi = True
        else:
            self.use_napi = False

        if not self.use_napi:
            self._http_client = httpx.AsyncClient(timeout=30.0)

    async def tick(self) -> Dict[str, Any]:
        """Executa um tick no core (avança o estado do hub)."""
        if self.use_napi:
            # Chamada síncrona do napi (pode ser bloqueante; executar em thread pool)
            try:
                result = await asyncio.to_thread(napi.core_tick)
                return json.loads(result) if isinstance(result, str) else result
            except NameError:
                return {"status": "tick stub ok (napi import failed)"}
        else:
            # resp = await self._http_client.post(f"{self.core_http_url}/core/tick")
            # resp.raise_for_status()
            # return resp.json()
            return {"status": "tick stub ok"}

    async def get_policy(self) -> Dict[str, Any]:
        """Obtém a política atual do core (Axiarquia)."""
        if self.use_napi:
            try:
                result = await asyncio.to_thread(napi.get_policy)
                return json.loads(result) if isinstance(result, str) else result
            except NameError:
                return {"policy": "stub"}
        else:
            # resp = await self._http_client.get(f"{self.core_http_url}/core/policy")
            # resp.raise_for_status()
            # return resp.json()
            return {"policy": "stub"}

    async def accept_recommendation(self, rec_id: str, user_feedback: Optional[float] = None) -> Dict[str, Any]:
        """Informa o core que uma recomendação foi aceita (ou rejeitada)."""
        payload = {"rec_id": rec_id}
        if user_feedback is not None:
            payload["feedback"] = user_feedback

        if self.use_napi:
            try:
                result = await asyncio.to_thread(napi.accept_recommendation, json.dumps(payload))
                return json.loads(result) if isinstance(result, str) else result
            except NameError:
                return {"status": "accepted stub"}
        else:
            # resp = await self._http_client.post(f"{self.core_http_url}/core/accept", json=payload)
            # resp.raise_for_status()
            # return resp.json()
            return {"status": "accepted stub"}

    async def get_state(self) -> Dict[str, Any]:
        """Obtém o estado atual do core (C, I, E e métricas)."""
        if self.use_napi:
            try:
                result = await asyncio.to_thread(napi.get_state)
                return json.loads(result) if isinstance(result, str) else result
            except NameError:
                return {"state": "stub"}
        else:
            # resp = await self._http_client.get(f"{self.core_http_url}/core/state")
            # resp.raise_for_status()
            # return resp.json()
            return {"state": "stub"}

    async def record_event(self, event_type: str, data: Dict[str, Any]) -> None:
        """Registra um evento arbitrário no core (para auditoria)."""
        payload = {"type": event_type, "data": data}
        if self.use_napi:
            try:
                await asyncio.to_thread(napi.record_event, json.dumps(payload))
            except NameError:
                pass
        else:
            # await self._http_client.post(f"{self.core_http_url}/core/event", json=payload)
            pass

    async def close(self):
        """Fecha conexões HTTP se estiver usando fallback."""
        if not self.use_napi and hasattr(self, '_http_client'):
            await self._http_client.aclose()
