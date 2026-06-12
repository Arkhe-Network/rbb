#!/bin/bash
# ============================================================
# CATHEDRAL ARKHE v12.0 — Provisionamento de Template (Substrato 1101)
# ============================================================

# Clonar template minimal Fedora
qvm-clone fedora-39-minimal cathedral-template

# Instalar dependências comuns no template
qvm-run -u root cathedral-template "dnf install -y python3 python3-pip rust cargo golang postgresql-server postgresql-contrib"

# Instalar blst no template
qvm-run -u root cathedral-template "cargo install blst"

# Atualizar template
qvm-run -u root cathedral-template "dnf upgrade -y"
