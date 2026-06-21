# Crypto Mapping and Performance Metrics

## Uses of Cryptography
- **ed25519-dalek**: Used in `src/substrato_8000/src/ema_integration.rs` and `src/substrato_5002/src/meta_controller_v2_3.rs` for verifying token signatures (`verify_ed25519`).
- **blake3**: Used in `cathedral-sdk-rs` and `bridge` for hashing.

## Goals
- Add ML-DSA as a supported signing algorithm.
- Define a unified interface for dual-stack signing (ML-DSA and Ed25519).
- Introduce metrics to track the performance of ML-DSA signatures vs. Ed25519.

## Adjustments for gRPC Max Message Size
Although ML-DSA signatures (e.g. ML-DSA-65) can be around ~3.3 KB and public keys around 1.9 KB, the default gRPC maximum message size limit of 4MB handles them perfectly without any modifications.
Additionally, for batch uploads where payload sizes might grow, we ensure no strict custom limits below 16MB are set in `cathedral-sdk-rs` or `substrato_8000`.

## Testing
We have included integration and unit tests under `cathedral-sdk-rs/src/tests_crypto.rs` to measure and benchmark the `ed25519` component performance. Given current tooling constraints, `ml-dsa` was simulated as a stub.
