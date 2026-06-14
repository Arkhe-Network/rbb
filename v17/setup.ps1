<#
.SYNOPSIS
Instalador Automático do Cathedral AGI v17.0 (Fases 1 a 4)

.DESCRIPTION
Este script automatiza a configuração do ambiente para o Cathedral AGI v17.0 no Windows 11.
Ele configura o Fast Brain no Windows Nativo e o Slow Brain no WSL2 com SGLang.

.NOTES
Execute este script como Administrador.
#>

$ErrorActionPreference = "Stop"

Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "  Instalador Automático Cathedral AGI v17.0 (Fases 1-4) " -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host ""

# Verificações Iniciais
Write-Host "[1/5] Verificando pré-requisitos..." -ForegroundColor Yellow

# Verificar Python no Windows
if (-not (Get-Command "python" -ErrorAction SilentlyContinue)) {
    Write-Error "Python não encontrado no Windows. Por favor, instale o Python 3.10+ e adicione ao PATH."
    exit 1
}

# Verificar WSL
if (-not (Get-Command "wsl" -ErrorAction SilentlyContinue)) {
    Write-Error "WSL não encontrado. Por favor, instale o WSL2 com 'wsl --install'."
    exit 1
}

Write-Host "Pré-requisitos confirmados." -ForegroundColor Green
Write-Host ""

# Configuração do Fast Brain (Windows Nativo)
Write-Host "[2/5] Configurando o Fast Brain (Windows Nativo)..." -ForegroundColor Yellow

$VENV_DIR = "C:\Cathedral\v17\venv_fast"
$SRC_DIR = "C:\Cathedral\v17"

if (-not (Test-Path $SRC_DIR)) {
    New-Item -ItemType Directory -Force -Path $SRC_DIR | Out-Null
}

if (-not (Test-Path $VENV_DIR)) {
    Write-Host "Criando ambiente virtual Python em $VENV_DIR..."
    python -m venv $VENV_DIR
}

Write-Host "Instalando dependências do Fast Brain..."
& "$VENV_DIR\Scripts\python.exe" -m pip install --upgrade pip
& "$VENV_DIR\Scripts\python.exe" -m pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121
& "$VENV_DIR\Scripts\python.exe" -m pip install psutil pynvml transformers trl axolotl fastapi uvicorn requests pyadl opencv-python pydantic
Write-Host "Fast Brain configurado." -ForegroundColor Green
Write-Host ""

# Configuração do Slow Brain (WSL2 + SGLang)
Write-Host "[3/5] Configurando o Slow Brain (WSL2 + SGLang)..." -ForegroundColor Yellow

# Script bash a ser executado dentro do WSL, com line endings do linux.
$WSL_SCRIPT = @"
#!/bin/bash
set -e
echo "Atualizando pacotes no WSL..."
sudo apt-get update && sudo apt-get install -y python3-pip python3-venv curl
if [ ! -d ~/cathedral_v17_wsl ]; then
    mkdir -p ~/cathedral_v17_wsl
fi
cd ~/cathedral_v17_wsl
if [ ! -d venv_slow ]; then
    python3 -m venv venv_slow
fi
source venv_slow/bin/activate
pip install --upgrade pip
echo "Instalando SGLang e dependencias no WSL..."
pip install "sglang[all]" vllm flashinfer
echo "Configuração do Slow Brain concluída."
"@ -replace "`r`n", "`n"

# Passamos diretamente via std_in
$WSL_SCRIPT | wsl bash

Write-Host "Slow Brain configurado." -ForegroundColor Green
Write-Host ""

# Teste Inicial
Write-Host "[4/5] Preparando Teste Inicial..." -ForegroundColor Yellow

$TEST_SCRIPT = @"
import psutil
try:
    import pynvml
    pynvml.nvmlInit()
    print('NVIDIA NVML carregado com sucesso.')
except ImportError:
    print('pynvml nao instalado ou GPU NVIDIA ausente.')
except Exception as e:
    print('Erro ao carregar NVML:', e)

print('Verificando Fast Brain... OK')
print('CPU:', psutil.cpu_percent(), '%')
"@

$TEST_SCRIPT_PATH = "$SRC_DIR\test_setup.py"
Set-Content -Path $TEST_SCRIPT_PATH -Value $TEST_SCRIPT

Write-Host "[5/5] Executando Teste Inicial..." -ForegroundColor Yellow
& "$VENV_DIR\Scripts\python.exe" $TEST_SCRIPT_PATH

Write-Host ""
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host " Instalação Concluída (Fases 1-4)!" -ForegroundColor Green
Write-Host " Arquivos de integração (como orchestrator_v17_full.py) "
Write-Host " podem ser executados com: "
Write-Host " $VENV_DIR\Scripts\python.exe C:\Cathedral\v17\orchestrator_v17_full.py"
Write-Host "========================================================" -ForegroundColor Cyan
