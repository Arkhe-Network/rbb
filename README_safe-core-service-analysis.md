# Análise safe-core-service v1.1.1

## Resumo
O serviço apresenta problemas críticos relacionados à arquitetura Rust (teste de integração falhando por ser binário ao invés de biblioteca), integridade de hash de decisão e vulnerabilidades a replay attacks.

## Bugs Críticos

*   **T1: Crate binária ao invés de lib.** Testes de integração em `tests/` não conseguem importar funções porque o crate é marcado como `[[bin]]` sem um root lib export. **Correção:** Adicionar `src/lib.rs` e separar a inicialização.
*   **T2: Near-duplicate detection quebrada.** O hash calculado localmente para comparação (`title|body`) não corresponde ao hash guardado no DB (`prev:domain:title:body:handle`).
*   **T3: Replay Attack.** O nonce de autenticação não é validado e armazenado; a requisição é válida pela janela inteira de 5 minutos, permitindo flood ou replays maliciosos.
*   **T4: Integridade do hash.** O hash de decisão falha em cobrir todos os campos sensíveis (is_boundary, negation_body) e a assinatura usa uma preimage que omite o previous_hash, rompendo o link criptográfico pleno da corrente.
*   **T5: Migrações não processadas.** O `init_db` apenas conecta, sem rodar `sqlx::migrate!()`, de modo que o serviço falha num container limpo sem BD construído manualmente.
*   **T6: CORS Permissivo.** A configuração default `CorsLayer::new()` é excessivamente permissiva, ignorando restrições de domínios conhecidos caso não exista `ALLOWED_ORIGINS` populado.

## Outras Observações

*   Eventos da tabela `events` (como 'treasury', 'attest', 'moderation') não são populados nas operações atuais.
*   Modelos `NewDecision` e `TreasuryEntry` marcados em `models.rs` são mortos e não consumidos pelos Handlers.
*   Não há limites / ratelimit aplicados para `register`.
*   A query no Attest de decisões (`generate_attestation`) só faz o check do snapshot (tail id e count total) e não valida a chain via hash backtracking iterativo.
