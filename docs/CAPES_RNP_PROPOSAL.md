# 📋 Proposta de Cooperação — Safe-Core Academic para CAPES/RNP

## 1. Contexto

A fragmentação dos sistemas acadêmicos brasileiros (SIGAA, SIPAC, Sucupira, CAFe) gera retrabalho significativo para as IES e dificulta a coleta de dados pela CAPES. O Safe-Core Academic propõe uma **camada de interoperabilidade semântica** que automatiza a integração entre esses sistemas.

## 2. Solução Proposta

### 2.1 Arquitetura

```
IES (SIGAA/SIPAC) → Academic Adapter → Safe-Core Core → Sucupira
                                    ↓
                              Merkle Tree (Audit)
                                    ↓
                           Dashboard de Transparência
```

### 2.2 Benefícios

| Benefício | Descrição |
|-----------|-----------|
| **Redução de retrabalho** | Automatização da coleta de dados |
| **Auditoria imutável** | Merkle Tree com assinatura criptográfica |
| **LGPD compliance** | Pseudonimização imediata de dados sensíveis |
| **Transparência** | Dashboard público com integridade verificável |

## 3. Cronograma

| Fase | Atividade | Prazo |
|------|-----------|-------|
| 1 | Piloto UnB | 3 meses |
| 2 | Expansão para 5 IES | 6 meses |
| 3 | Integração com Sucupira | 9 meses |
| 4 | Escala nacional | 18 meses |

## 4. Recursos Necessários

- **Equipe técnica:** 3 engenheiros Rust
- **Infraestrutura:** Servidores para Qdrant + Merkle Tree
- **Parcerias:** RNP (infraestrutura), CAPES (integração)

## 5. Orçamento Estimado

| Item | Custo |
|------|-------|
| Desenvolvimento | R$ 500.000 |
| Infraestrutura | R$ 200.000 |
| Treinamento | R$ 100.000 |
| Total | R$ 800.000 |

## 6. Próximos Passos

1. Reunião com CAPES para apresentação
2. Reunião com RNP para infraestrutura
3. Definição de métricas de sucesso
4. Assinatura de termo de cooperação
5. Início do piloto UnB
