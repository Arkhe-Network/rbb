# Pilot Omnipresence PoC

Protótipo para validar a camada de armazenamento e consenso. Tem como missão verificar se os protocolos escolhidos (IPFS + Hotmint) garantem a presença distribuída da rede.

## Arquitetura

- **Armazenamento**: IPFS (interplanetary file system) para armazenamento imutável e descentralizado.
- **Consenso**: Hotmint BFT / HotStuff-2 para coordenar ações e eventos do sistema.

## Execução

Use o Docker Compose para subir os nós locais:

```bash
docker-compose up -d
```

## Cenários de Teste

1. **Testes de Resiliência**: Desconectar `hotmint-node2` e verificar se o consenso continua rodando com outros nós se existirem.
2. **Armazenamento de Dados**: Adicionar um arquivo ao IPFS e verificar se pode ser recuperado a partir do seu hash e se os nós concordam sobre o hash.
3. **Descoberta de Nós**: Certificar-se de que os nós de consenso conseguem se descobrir e assinar blocos na rede local.