1. *Criação da estrutura de diretórios*
   - Criar `kernel/rust/src/...`
   - Criar `framework/kotlin/src/cathedral/...`
   - Criar `system/daemons/...`
   - etc
2. *Criar e popular Cargo.toml do kernel*
   - O Cargo.toml do workspace rust dentro de `kernel/rust/` precisa estar correto e compatível com as features pedidas
3. *Implementar os arquivos Kotlin*
   - Implementar `Agent.kt`, `AgentPQC.kt`, `PQC.kt`, `HybridCertificate.kt` etc
4. *Implementar o núcleo Rust*
   - Implementar os crates `cathedral-core`, `cathedral-bridge`, etc
5. *Completar pre commit steps*
   - Complete pre commit steps to make sure proper testing, verifications, reviews and reflections are done.
6. *Submit the change*
