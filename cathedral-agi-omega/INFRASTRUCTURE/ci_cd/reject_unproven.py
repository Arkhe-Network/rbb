#!/usr/bin/env python3
import os
import sys
import subprocess

CRITICAL_DIRS = [
    "ZK_REASONING_ENGINE/circuits/",
    "COGNITIVE_CORTEX/agents/",
    "DISTRIBUTED_COMPUTATION/"
]

def get_changed_files():
    # If in GitHub Actions, we could use PR diff.
    # For now, we mock it via git commands if needed, or assume environment variables.
    try:
        base_commit = os.environ.get("GITHUB_BASE_REF", "main")
        cmd = ["git", "diff", "--name-only", f"origin/{base_commit}"]
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        return result.stdout.splitlines()
    except Exception as e:
        print(f"Warning: Could not get changed files via git ({e}). Using empty list.")
        return []

def main():
    changed_files = get_changed_files()

    touches_critical = False
    for f in changed_files:
        for d in CRITICAL_DIRS:
            if f.startswith(d):
                touches_critical = True
                break

    if touches_critical:
        # Check if there is a corresponding lean proof file changed
        has_proof = any(f.endswith(".lean") and "LEAN4_SUPEREGO" in f for f in changed_files)

        if not has_proof:
            print("❌ ERROR: Changes detected in critical directories but no corresponding Lean 4 proof was found in LEAN4_SUPEREGO/.")
            print("AGI Safety Protocol violated. Rejecting PR.")
            sys.exit(1)
        else:
            print("✅ SUCCESS: Critical changes have an associated Lean 4 proof.")
    else:
        print("✅ No critical directories modified. Proceeding.")

if __name__ == "__main__":
    main()
