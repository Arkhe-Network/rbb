# Taproot Assets gRPC API

Mapeamento completo da API gRPC do Taproot Assets Daemon (`tapd`).

## 1. Serviço `TaprootAssets` (Core)
- **GetInfo**: Informações do nó.
- **ListAssets**: Lista ativos da carteira.
- **ListBalances**: Balanços por ativo ou grupo.
- **NewAddr**: Cria novo endereço para recebimento de ativos.
- **SendAsset**: Envia ativos para um endereço on-chain ou Lightning.
- **BurnAsset**: Queima (invalida) unidades de um ativo.
- **VerifyProof**: Verifica uma prova de ativo localmente.

## 2. Serviço `AssetWallet` (Gestão)
- **CreateAsset**: Inicia a criação de um novo ativo pendente.
- **IssueAsset**: Emite lotes do ativo pendente on-chain.

## 3. Serviço `Universe` (Federação)
- **QueryUniverse**: Consulta provas num servidor Universe local/remoto.

A arquitetura usa gRPC com porta 10029 por padrão, necessitando de conexões via mTLS e macaroons de autorização.
