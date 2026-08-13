#!/bin/bash
# Remove duplicate kem_sk
sed -i 's/kem_sk: alloc::vec::Vec<u8>,//' crates/arkhe-quantum-auth/tests/integration_tests.rs
# Actually, wait, let's see where they are declared.
