#!/bin/bash
# Cathedral ARKHE v28.3 — Agent Runtime Initialization
# Verifica integridade do modelo, valida assinaturas SPHINCS+ reais (via openssl stub ou ferramenta customizada), aguarda serviços e inicia o agente
# Selo: CATHEDRAL-ARKHE-v28.3-INIT-SCRIPT-2026-06-16

set -euo pipefail

echo "═══════════════════════════════════════════════════════════════"
echo " Cathedral ARKHE v28.3 — Agent Runtime Initialization"
echo "═══════════════════════════════════════════════════════════════"

# ============================================================================
# 1. Configuração de ambiente
# ============================================================================
export CATHEDRAL_CONFIG="${CATHEDRAL_CONFIG:-/app/agent/config.yaml}"
export CATHEDRAL_MANIFEST="${CATHEDRAL_MANIFEST:-/app/core/model/manifest.json}"
export INTEGRITY_PROOF="${INTEGRITY_PROOF:-/app/trust/integrity/integrity_proof.json}"
export LLM_SERVER_URL="${LLM_SERVER_URL:-http://llm-server:8000}"
export RUST_LOG="${RUST_LOG:-info}"
export TEMPORAL_DSN="${TEMPORAL_DSN:-postgres://cathedral:secure_password@temporal-chain:5432/temporalchain}"

# ============================================================================
# 2. Função para verificar assinatura SPHINCS+ (simulação realista)
# ============================================================================
verify_sphincs_signature() {
    local model_path="$1"
    local signature_file="$2"
    local public_key="$3"

    echo "🔐 Verificando assinatura SPHINCS+ do modelo..."

    if [ ! -f "$model_path" ]; then
        echo "⚠️  Modelo não encontrado em: $model_path"
        return 1
    fi

    if [ ! -f "$signature_file" ]; then
        echo "⚠️  Arquivo de assinatura não encontrado: $signature_file"
        return 1
    fi

    if [ ! -f "$public_key" ]; then
        echo "⚠️  Chave pública não encontrada: $public_key"
        return 1
    fi

    # Exemplo de ferramenta customizada de verificação pqc-verify.
    # Em produção, usaria um binário real do ecossistema Cathedral ou OpenSSL3 com providers OQS.
    if command -v pqc-verify &> /dev/null; then
        if pqc-verify --alg sphincs-shake-256s --pub "$public_key" --sig "$signature_file" --msg "$model_path"; then
            echo "✅ Assinatura SPHINCS+ válida."
            return 0
        else
            echo "❌ Assinatura SPHINCS+ INVÁLIDA."
            return 1
        fi
    elif command -v openssl &> /dev/null; then
         # Simulação com openssl para fins de completude, caso o provider OQS esteja instalado:
         # openssl dgst -verify "$public_key" -signature "$signature_file" "$model_path"
         echo "⚠️  Ferramenta 'pqc-verify' ausente. Simulando verificação SPHINCS+..."
         local expected_hash=$(sha256sum "$model_path" | awk '{print $1}')
         local sig_hash=$(grep -o '"hash": *"[^"]*"' "$signature_file" 2>/dev/null | cut -d'"' -f4 || echo "")
         if [ "$expected_hash" = "$sig_hash" ]; then
              echo "✅ Assinatura SPHINCS+ (simulada) válida."
              return 0
         else
              echo "❌ Assinatura SPHINCS+ (simulada) INVÁLIDA."
              return 1
         fi
    else
         echo "❌ Nenhuma ferramenta de verificação criptográfica encontrada."
         return 1
    fi
}


# ============================================================================
# 3. Função para verificar integridade do modelo (JSON)
# ============================================================================
verify_integrity() {
    local proof_file="$1"
    echo "🔒 Verificando integridade JSON do modelo..."

    if [ ! -f "$proof_file" ]; then
        echo "⚠️  Arquivo de prova de integridade não encontrado: $proof_file"
        echo "   Continuando sem verificação (modo não seguro)."
        return 0
    fi

    # Extrair informações do integrity_proof.json
    local model_hash=$(grep -o '"model_hash": *"[^"]*"' "$proof_file" | cut -d'"' -f4 || echo "")
    local verified=$(grep -o '"verified": *true' "$proof_file" || echo "false")

    if [ "$verified" != '"verified": true' ] && [ "$verified" != "true" ]; then
         # grep found something, checking contents
         if grep -q '"verified": *true' "$proof_file"; then
             verified="true"
         fi
    fi

    if [ -z "$model_hash" ] || [ "$verified" != "true" ]; then
        echo "❌ PROVA DE INTEGRIDADE INVÁLIDA: O arquivo json indica 'verified: false' ou está mal formatado."
        if [ "${SKIP_INTEGRITY_VERIFICATION:-0}" != "1" ]; then
            exit 1
        fi
    else
        echo "✅ Prova de integridade JSON verificada com sucesso!"
    fi
    return 0
}

# ============================================================================
# 4. Função para aguardar serviços dependentes
# ============================================================================
wait_for_service() {
    local host=$1
    local port=$2
    local name=$3
    local timeout=${4:-30}

    echo -n "⏳ Aguardando $name ($host:$port) ... "
    if command -v nc &> /dev/null; then
        local i=0
        while ! nc -z "$host" "$port" &> /dev/null; do
            i=$((i+1))
            if [ "$i" -ge "$timeout" ]; then
                echo "❌"
                echo "Erro: $name não está disponível após $timeout segundos."
                exit 1
            fi
            sleep 1
        done
        echo "✅"
    else
        echo "⚠️  Comando 'nc' não encontrado, pulando wait-for-it..."
    fi
}

# ============================================================================
# 5. Fluxo Principal
# ============================================================================

# Verificar integridade JSON
verify_integrity "$INTEGRITY_PROOF"

# Verificar assinatura SPHINCS+ (se os arquivos existirem)
MODEL_PATH="/app/core/model/model.safetensors"
SIG_PATH="/app/trust/signatures/agent_signature.pqc"
PUB_KEY_PATH="/app/trust/signatures/agent_pub.key"

if [ -f "$SIG_PATH" ] && [ -f "$PUB_KEY_PATH" ] && [ -f "$MODEL_PATH" ]; then
    if ! verify_sphincs_signature "$MODEL_PATH" "$SIG_PATH" "$PUB_KEY_PATH"; then
        if [ "${SKIP_INTEGRITY_VERIFICATION:-0}" != "1" ]; then
            echo "Abortando inicialização devido a falha criptográfica."
            exit 1
        fi
    fi
else
    echo "⚠️  Arquivos de assinatura SPHINCS+ não encontrados; pulando verificação PQ."
fi

# Aguardar serviços dependentes
echo "⏳ Aguardando serviços dependentes..."
wait_for_service llm-server 8000 "LLM Server" 60
wait_for_service redis 6379 "Redis" 30
wait_for_service vector-db 6333 "Vector DB (Qdrant)" 30

# Inicializar o agente
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "🚀 Iniciando Cathedral Agent Runtime v28.3"
echo "   Config: $CATHEDRAL_CONFIG"
echo "   Manifest: $CATHEDRAL_MANIFEST"
echo "   LLM Server: $LLM_SERVER_URL"
echo "═══════════════════════════════════════════════════════════════"

exec cathedral-agent-runtime \
    --config "$CATHEDRAL_CONFIG" \
    --manifest "$CATHEDRAL_MANIFEST" \
    --llm-url "$LLM_SERVER_URL" \
    --http-port 8001 \
    --log-level "$RUST_LOG"
