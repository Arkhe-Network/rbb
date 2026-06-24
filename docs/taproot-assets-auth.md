# Taproot Assets Authentication

O `tapd` utiliza credenciais granulares (Macaroons) em conjunto com mTLS.

## Macaroons
Um macaroon é um token bearer com caveats criptográficos. O `tapd` delega a validação ao `lnd` interno e usa os macaroons `admin`, `readonly`, entre outros, para fornecer controle fino.

As operações exigem enviar os macaroons nos headers da requisição gRPC no formato Hex-encoded no metadado `macaroon`.
