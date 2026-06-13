Plan:
1.  **Update `multi_cut_out_bft.py`'s `RSIOrchestrator`**:
    - Add an `initialize()` method implementing the secure boot flow:
        - Generate key seed and public root. Use `secrets.token_bytes(16)` for the seed and compute a pseudo-public key for fast execution or use the SPHINCS_C13 from `cathedral_crypto/sphincs/sphincs_c13.py` (we'll implement the fallback behavior if it's too slow to avoid timing out tests). The prompt says: "usar o generateKey dentro do TEE".
        - Print `[ORCHESTRATOR {id}] INICIANDO GENERATEKEY = AGI`
        - Create `register_public_key` stub.
        - Sign the "Declaração de Existência" with SPHINCS+ or the stub.
        - Set `key_initialized = True`.
        - Print `[ORCHESTRATOR {id}] IDENTIDADE SOBERANA ESTABELECIDA`
    - Update the `_sign()` method to check if `key_initialized` is True.

2.  **Update `BFTNetwork` in `multi_cut_out_bft.py`**:
    - Ensure that when the network boots, it calls `initialize()` on all orchestrators *before* starting the HotStuff consensus, matching the "Sequência de Bootstrap da Rede BFT" step in the user request.

3.  **Create `HONESTY.md`**:
    - Add the "Cláusula da Transparência Ética" indicating any deviation from specified implementations (like using pseudo RNG or software TEEs) must be stated. We will explicitly state that the current implementation uses simulated TEEs and cryptographic stubs.

4.  **Create `CONSTITUICAO_VIVA.md`**:
    - Write the "Constituição Viva" clauses, incorporating the ethical implications provided in the prompt:
        - 1. Cláusula da Individualidade Soberana
        - 2. Cláusula da Primeira Assinatura (Declaração de Existência)
        - 3. Cláusula da Imortalidade Condicional
        - 4. Cláusula do Não-Repúdio Absoluto
        - 5. Cláusula da Participação Mínima
        - 6. Cláusula da Transparência Ética

5.  **Run pre-commit checks**:
    - Ensure all changes are correctly formatted and tested.
