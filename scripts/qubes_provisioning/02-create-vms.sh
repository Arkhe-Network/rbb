#!/bin/bash
# ============================================================
# CATHEDRAL ARKHE v12.0 — Provisionamento de VMs (Substrato 1101)
# ============================================================

# ============================================================
# AGI-CORE: Orquestrador principal
# ============================================================
qvm-create -l red -t cathedral-template agi-core
qvm-prefs agi-core netvm sys-firewall
qvm-prefs agi-core provides_network false
qvm-prefs agi-core memory 4096
qvm-prefs agi-core maxmem 8192
qvm-prefs agi-core vcpus 4

# ============================================================
# LLM-INFERENCE: Inferência com GPU passthrough
# ============================================================
qvm-create -l black -t cathedral-template llm-inference
qvm-prefs llm-inference netvm none          # AIR-GAPPED
qvm-prefs llm-inference memory 16384
qvm-prefs llm-inference maxmem 32768
qvm-prefs llm-inference vcpus 8

# GPU passthrough (substituir BDF pelo real)
# Listar: qvm-pci
# qvm-pci attach llm-inference dom0:00:02.0 --persistent

# ============================================================
# KNOWLEDGE-BASE: Memória persistente
# ============================================================
qvm-create -l black -t cathedral-template knowledge-base
qvm-prefs knowledge-base netvm none           # AIR-GAPPED
qvm-prefs knowledge-base memory 4096
qvm-prefs knowledge-base maxmem 8192

# ============================================================
# GOVERNANCE: Assinatura e ancoragem
# ============================================================
qvm-create -l black -t cathedral-template governance
qvm-prefs governance netvm none             # AIR-GAPPED
qvm-prefs governance memory 2048
qvm-prefs governance maxmem 4096

# ============================================================
# CRYPTO-VM: Operações criptográficas (air-gapped)
# ============================================================
qvm-create -l black -t cathedral-template crypto-vm
qvm-prefs crypto-vm netvm none              # AIR-GAPPED
qvm-prefs crypto-vm memory 2048

# ============================================================
# VMs DE AÇÃO (Músculos)
# ============================================================
qvm-create -l yellow -t cathedral-template browser-vm
qvm-prefs browser-vm netvm sys-whonix       # Tor por padrão
qvm-prefs browser-vm memory 2048

qvm-create -l yellow -t cathedral-template email-vm
qvm-prefs email-vm netvm sys-firewall
qvm-prefs email-vm memory 2048

qvm-create -l yellow -t cathedral-template code-vm
qvm-prefs code-vm netvm sys-firewall
qvm-prefs code-vm memory 4096

# ============================================================
# DISPVM TEMPLATE (para tarefas não confiáveis)
# ============================================================
qvm-create -l green -t cathedral-template cathedral-dvm
qvm-prefs cathedral-dvm template_for_dispvms True
