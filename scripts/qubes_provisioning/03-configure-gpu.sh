#!/bin/bash
# ============================================================
# CATHEDRAL ARKHE v12.0 — Configuração GPU Passthrough (Substrato 1101)
# ============================================================

# Editar /etc/default/grub no dom0
# GRUB_CMDLINE_LINUX="... iommu=pt iommu=1 rd.qubes.hide_all_usb swiotlb=8192"

# Aplicar
# sudo grub2-mkconfig -o /boot/grub2/grub.cfg

# No dom0: listar dispositivos PCI
# qvm-pci

# No llm-inference: verificar GPU visível
# qvm-run -a llm-inference "lspci | grep VGA"

# Testar inferência
# qvm-run -a llm-inference "python3 -c 'import torch; print(torch.cuda.is_available())'"
