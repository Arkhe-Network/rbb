#!/usr/bin/env python3
import json
import hashlib
from datetime import datetime
import os
import yaml

# Simulated OBSIDIAN_API for testing
OBSIDIAN_API = "http://localhost:27123"

def post_analysis(title, content, domain, version, selo):
    """Envia uma análise para o Obsidian como nota."""
    hash_val = hashlib.sha256(content.encode()).hexdigest()
    frontmatter = {
        "title": title,
        "type": "analysis",
        "domain": domain,
        "version": version,
        "date": datetime.now().isoformat(),
        "status": "rascunho",
        "hash": hash_val,
        "selo": selo,
        "tags": ["analysis", domain]
    }
    note = f"---\n{yaml.dump(frontmatter)}---\n\n{content}"
    path = f"01 - Analyses/{domain}/{title}.md"
    # To run test, we just print instead of making real request
    print(f"Would PUT to {OBSIDIAN_API}/vault/{path}")
    print("Content:")
    print(note)
    return True

if __name__ == "__main__":
    post_analysis("Test Analysis", "Hello world from BLOCK 11", "BLOCK-11", "v1.0", "HANKEL-SEAL-123")
