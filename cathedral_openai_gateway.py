from fastapi import FastAPI, Request, HTTPException
from fastapi.responses import JSONResponse
import json
import time
import logging
from typing import Optional, List, Dict, Any
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import ConsoleSpanExporter, SimpleSpanProcessor
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor

# Initialize logging
logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
logger = logging.getLogger("cathedral.openai_gateway")

# OpenTelemetry configuration
trace.set_tracer_provider(TracerProvider())
tracer = trace.get_tracer(__name__)
# Export to console for debugging purposes as per requirements
trace.get_tracer_provider().add_span_processor(SimpleSpanProcessor(ConsoleSpanExporter()))

app = FastAPI(title="Cathedral OpenAI Gateway", version="1.0.0")
FastAPIInstrumentor.instrument_app(app)

# Local imports for the Cathedral ecosystem
import sys
import os
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
try:
    from cathedral_arkhe_v12_1_2_agi import CathedralOrchestratorV12_1_2
    orch = CathedralOrchestratorV12_1_2(config=None)
    logger.info("Integrated CathedralOrchestratorV12_1_2 successfully.")
except Exception as e:
    logger.error(f"Failed to load CathedralOrchestratorV12_1_2: {e}")
    orch = None

@app.post("/v1/chat/completions")
async def chat_completions(request: Request):
    with tracer.start_as_current_span("openai_chat_completions") as span:
        try:
            body = await request.json()
        except Exception:
            raise HTTPException(status_code=400, detail="Invalid JSON body")

        model = body.get("model", "Rio-3.5-Open-397B")
        messages = body.get("messages", [])

        span.set_attribute("model", model)
        span.set_attribute("num_messages", len(messages))

        if not messages:
            raise HTTPException(status_code=400, detail="No messages provided")

        prompt = messages[-1].get("content", "")

        # 1. Roteamento Federado (Simulated as we are connecting to OrchestratorV5 logic)
        with tracer.start_as_current_span("federated_routing") as route_span:
            route_span.set_attribute("target_model", model)
            route_span.set_attribute("routing.allowed_jurisdictions", "BRA,ORB")
            # In a full Rust integration, this would call arkhe_core::inference::federated_router

        # 2. Protocolo de Corte 294 / TEE Verification (Simulated by Orchestrator's internal methods)
        with tracer.start_as_current_span("corte_protocol") as corte_span:
            corte_span.set_attribute("corte.status", "INACTIVE")

        # 3. Inferência utilizando o Orquestrador Integrado
        with tracer.start_as_current_span("orchestrator_infer") as infer_span:
            if orch:
                try:
                    orch_result = orch.infer(prompt, max_tokens=body.get("max_tokens", 50))
                    output_text = orch_result.get("output", f"Resposta gerada pela Federação usando o modelo {model}.")
                    infer_span.set_attribute("orchestrator.cycle", orch.cycle_count)
                    infer_span.set_attribute("orchestrator.telemetry", json.dumps(orch.get_telemetry()))
                except Exception as e:
                    output_text = f"Erro no Orchestrator: {e}"
                    infer_span.set_attribute("orchestrator.status", "failed")
            else:
                output_text = f"Resposta gerada pela Federação usando o modelo {model}. (Mock pois Orchestrator falhou)"
                infer_span.set_attribute("orchestrator.status", "failed")

        response_data = {
            "id": f"chatcmpl-arkhe-{int(time.time()*1000)}",
            "object": "chat.completion",
            "created": int(time.time()),
            "model": model,
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": output_text
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": len(prompt.split()),
                "completion_tokens": len(output_text.split()),
                "total_tokens": len(prompt.split()) + len(output_text.split())
            }
        }

        span.set_attribute("response.tokens", response_data["usage"]["total_tokens"])
        return JSONResponse(content=response_data)

@app.get("/health")
def health_check():
    return {"status": "ok", "orchestrator": "online" if orch else "offline"}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run("cathedral_openai_gateway:app", host="0.0.0.0", port=8080, log_level="info")
