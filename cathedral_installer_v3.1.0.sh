#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# CATHEDRAL ARKHE — INSTALLER v3.1.0-FULL
# Instalação automatizada dos 3 stubs críticos restantes
# ═══════════════════════════════════════════════════════════════════════════════

set -e

echo "╔══════════════════════════════════════════════════════════════════════════════╗"
echo "║  CATHEDRAL ARKHE — INSTALLER v3.1.0-FULL                                   ║"
echo "║  Instalação dos stubs críticos: Lean4, Docker, Blockchain RPC            ║"
echo "╚══════════════════════════════════════════════════════════════════════════════╝"
echo

# ──────────────────────────────────────────────────────────────────────────────
# 1. LEAN 4 (via elan)
# ──────────────────────────────────────────────────────────────────────────────
install_lean4() {
    echo "[1/3] Instalando Lean 4 (elan)..."

    if command -v elan &> /dev/null; then
        echo "  ✓ elan já instalado: $(elan --version)"
        return 0
    fi

    # Instala elan (gerenciador de versões Lean)
    curl https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh -sSf | sh
    source $HOME/.elan/env

    # Instala Lean 4 stable + Mathlib
    elan toolchain install stable
    elan default stable

    # Verifica instalação
    lean --version
    lake --version

    echo "  ✓ Lean 4 instalado com sucesso"
}

# ──────────────────────────────────────────────────────────────────────────────
# 2. DOCKER (via apt)
# ──────────────────────────────────────────────────────────────────────────────
install_docker() {
    echo "[2/3] Instalando Docker..."

    if command -v docker &> /dev/null; then
        echo "  ✓ Docker já instalado: $(docker --version)"
        return 0
    fi

    # Atualiza repositórios e instala dependências
    sudo apt-get update
    sudo apt-get install -y ca-certificates curl gnupg lsb-release

    # Adiciona chave GPG oficial do Docker
    sudo mkdir -p /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg

    # Configura repositório
    echo       "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
      $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

    # Instala Docker Engine
    sudo apt-get update
    sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin

    # Adiciona usuário ao grupo docker
    sudo usermod -aG docker $USER

    # Inicia serviço
    sudo systemctl start docker
    sudo systemctl enable docker

    # Testa instalação
    docker run hello-world

    echo "  ✓ Docker instalado com sucesso"
    echo "  ⚠ Reinicie a sessão para aplicar permissões do grupo docker"
}

# ──────────────────────────────────────────────────────────────────────────────
# 3. PYTHON DEPENDENCIES (docker-py, web3.py)
# ──────────────────────────────────────────────────────────────────────────────
install_python_deps() {
    echo "[3/3] Instalando dependências Python..."

    pip install docker web3 py-solc-x

    echo "  ✓ docker-py instalado"
    echo "  ✓ web3.py instalado"
    echo "  ✓ py-solc-x instalado"
}

# ──────────────────────────────────────────────────────────────────────────────
# EXECUÇÃO
# ──────────────────────────────────────────────────────────────────────────────

install_lean4
install_docker
install_python_deps

echo
echo "╔══════════════════════════════════════════════════════════════════════════════╗"
echo "║  INSTALAÇÃO CONCLUÍDA                                                       ║"
echo "╠══════════════════════════════════════════════════════════════════════════════╣"
echo "║  Próximos passos:                                                           ║"
echo "║  1. Reinicie a sessão: newgrp docker                                        ║"
echo "║  2. Verifique: python -c \"import docker; docker.from_env().ping()\"          ║"
echo "║  3. Verifique: lean --version && lake --version                             ║"
echo "║  4. Configure RPC da RBB Chain em: /etc/cathedral/rbb_rpc.conf             ║"
echo "╚══════════════════════════════════════════════════════════════════════════════╝"
