#!/bin/bash
# Cathedral ARKHE v28.3 — LLM Server Startup Script
# Selo: CATHEDRAL-ARKHE-v28.3-START-LLM-2026-06-16

set -euo pipefail

echo "═══════════════════════════════════════════════════════════════"
echo " 🧠 Cathedral ARKHE v28.3 — Starting vLLM Server"
echo "═══════════════════════════════════════════════════════════════"

MODEL_DIR=${CATHEDRAL_MODEL_PATH:-/models}
PORT=${VLLM_PORT:-8000}
HOST="0.0.0.0"

# Opcional: extrair parâmetros do manifest.json se existir
MODEL_ID="cathedral-llm-v28.3"
if [ -f "/app/config/manifest.json" ]; then
    MODEL_ID=$(grep -o '"model_id": *"[^"]*"' /app/config/manifest.json | cut -d'"' -f4 || echo "cathedral-llm-v28.3")
fi

echo "🚀 Iniciando modelo: $MODEL_ID a partir de $MODEL_DIR"

# Executando vLLM com otimizações para Cathedral
# Adicione --quantization awq ou gptq conforme necessário
exec python -m vllm.entrypoints.openai.api_server \
    --host "$HOST" \
    --port "$PORT" \
    --model "$MODEL_DIR" \
    --served-model-name "$MODEL_ID" \
    --trust-remote-code \
    --max-model-len 4096 \
    --tensor-parallel-size 1
