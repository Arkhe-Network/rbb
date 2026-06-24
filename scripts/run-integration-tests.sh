#!/bin/bash
set -e

echo "Running integration tests..."
cargo test -p cathedral-taproot-bridge --lib --tests
