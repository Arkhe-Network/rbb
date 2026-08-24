#!/bin/bash
for f in $(find crates apps native safe-core-sus -name Cargo.toml); do
    if ! grep -q 'license =' "$f" && ! grep -q 'license.workspace' "$f"; then
        if grep -q '^name = ' "$f"; then
             sed -i 's/^name = .*/&\nlicense = "MIT OR Apache-2.0"/' "$f"
        else
             echo 'license = "MIT OR Apache-2.0"' >> "$f"
        fi
    fi
done
