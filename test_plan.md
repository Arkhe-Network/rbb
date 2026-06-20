1. **common/src/crypto_config.rs:** Create and implement configuration for crypto dual-stack mode using PQC and Ed25519.
2. **cathedral-sdk/src/crypto.rs:** Implement dual-stack SDK with support for ML-DSA and fallback to Ed25519.
3. **bridge/src/signature_verifier.rs:** Implement signature verifier in bridge using dual-stack strategy.
4. **benches/pqc_benchmarks.rs:** Implement PQ benchmarks for signing algorithms.
5. **scripts/deploy_pqc.sh:** Create a deploy script for rollout.
6. Verify testing using pre_commit_instructions.
