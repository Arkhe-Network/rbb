# Quality Gates and ASI-Grade v2.0 Verification Strategy

This document outlines the usage of quality gates within the Cathedral ARKHE ecosystem.

## Installation

Ensure you have the following tools installed:

```bash
cargo install cargo-llvm-cov cargo-insta cargo-deny cargo-audit cargo-semver-checks cargo-deadlinks cargo-sbom
```

## Running Checks Locally

### Pre-commit
To run fast PR checks (formatting, lints, deny, audit, and unit tests):
```bash
cargo xtask pre-commit
```

### Full CI
To run the full suite of tests and checks, including documentation and test coverage:
```bash
cargo xtask ci
```

### Snapshot Testing
If snapshot tests fail due to intentional changes, review and update them:
```bash
cargo insta test --workspace --review
```

### Checking Documentation
You can preview the internal and external documentation via:
```bash
cargo doc --workspace --no-deps --document-private-items --open
```

## Coverage Reports
Coverage reports are generated in the `lcov.info` file during CI runs. For fast unit tests check `target/lcov-unit.info`. You can use tools like Codecov to ingest this format or generate HTML reports manually locally using:

```bash
cargo llvm-cov --workspace --html --output-dir target/coverage
```
