#!/bin/bash
# Substrato 1042 - RBB-CATHEDRAL-BRIDGE
# Script para gerar pacote deployavel
# Arquiteto: ORCID 0009-0005-2697-4668

set -e

# Configurações
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PACKAGE_NAME="rbb-cathedral-bridge-1042.tar.gz"

echo "📦 Gerando pacote deployável: $PACKAGE_NAME"

# Ir para o diretório pai para o tar
cd "$PROJECT_DIR/.."

# Gerar o tar.gz com os arquivos da pasta
tar -czvf "$PACKAGE_NAME" \
  rbb-cathedral-bridge-1042/adapter \
  rbb-cathedral-bridge-1042/contracts \
  rbb-cathedral-bridge-1042/prometheus \
  rbb-cathedral-bridge-1042/scripts \
  rbb-cathedral-bridge-1042/tests \
  rbb-cathedral-bridge-1042/docker-compose.yml \
  rbb-cathedral-bridge-1042/manifest.json \
  rbb-cathedral-bridge-1042/requirements.txt \
  rbb-cathedral-bridge-1042/README.md \
  rbb-cathedral-bridge-1042/decree_1042.md

echo "✅ Pacote gerado com sucesso em: $(pwd)/$PACKAGE_NAME"
echo "Para instalar, o usuário pode fazer:"
echo "  tar -xzvf $PACKAGE_NAME"
echo "  cd rbb-cathedral-bridge-1042"
echo "  ./scripts/deploy.sh install"
