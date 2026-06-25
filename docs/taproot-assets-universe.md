# Taproot Assets Universe

O modelo Universe é uma federação de nós que armazenam e disponibilizam provas de ativos para os clientes.

- **Genesis Proofs**: provam a emissão do ativo validado contra um tx de Bitcoin.
- **Transfer Proofs**: provam que a quantia não foi inflacionada ou gasta duas vezes através de provas Merkle que acompanham as transferências.

Clientes `tapd` podem adicionar servidores Universe remotos para sincronizar provas de ativos e garantir validade antes de aceitar transferências de rede e Lightning.
