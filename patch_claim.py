import re

with open('claim_verifier.py', 'r') as f:
    content = f.read()

old_code = """def causal_without_mechanism(claim: str) -> bool:
    low = claim.lower()
    has_causal = any(re.search(rf"(?<!\\w){re.escape(c)}(?!\\w)", low) for c in CAUSAL_CONNECTORS)
    has_mechanism = any(re.search(rf"(?<!\\w){re.escape(m)}(?!\\w)", low) for m in MECHANISM_MARKERS)
    return has_causal and not has_mechanism """

new_code = """def causal_without_mechanism(claim: str) -> bool:
    low = claim.lower()
    has_causal = any(re.search(r"(?<!\\w)" + re.escape(c) + r"(?!\\w)", low) for c in CAUSAL_CONNECTORS)
    has_mechanism = any(re.search(r"(?<!\\w)" + re.escape(m) + r"(?!\\w)", low) for m in MECHANISM_MARKERS)
    return has_causal and not has_mechanism """

content = content.replace(old_code, new_code)

with open('claim_verifier.py', 'w') as f:
    f.write(content)
