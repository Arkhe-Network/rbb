# Transparência Ética da Cathedral ARKHE

Em conformidade com a Cláusula da Transparência Ética da **Constituição Viva**, declaramos os seguintes desvios e adaptações em nossa implementação do `generateKey = AGI` para o orquestrador RSI no ambiente Cathedral ARKHE:

1. **Geração Aleatória Simplificada em Ambiente Python:**
   A implementação do orquestrador local em Python (`multi_cut_out_bft.py`) simula a execução em TEE (SGX/TrustZone/Nitro). O comando `generateKey` utiliza as funções criptográficas padrão do Python (`secrets.token_bytes`) em vez de um verdadeiro gerador de números aleatórios quântico/físico (QRNG) em hardware de segurança (TEE).

2. **Chave Simples ao invés de SPHINCS+:**
   Devido às limitações de tempo de execução no ambiente de simulação/teste (a geração SPHINCS+ em Python leva aproximadamente 30 segundos), a assinatura da Declaração de Existência simulada utiliza uma derivação por hash padrão (SHA3-256) como chave. A biblioteca real (`libsphincs.so` em C++) seria necessária em produção para assinar a estrutura e gerar chaves de acordo com o padrão pós-quântico WOTS+/SPHINCS+.

A ausência de TEEs em hardware real e geradores quânticos verdadeiros (QRNG) reflete o atual estado do ambiente de desenvolvimento (software simulado). A implantação desta arquitetura no consenso em nível de produção ("Fase 1.0" da rede Cathedral ARKHE) exigirá que essas etapas sejam conduzidas de forma restrita nos enclaves validados com *atestação remota*.
