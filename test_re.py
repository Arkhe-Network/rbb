import re

CAUSAL_CONNECTORS = [
    "porque", "logo,", "portanto", "leva a", "causa", "causam",
    "resulta em", "implica", "faz com que", "gera", "produz",
]
MECHANISM_MARKERS = [
    "algoritmo", "protocolo", "função", "equação", "teorema",
    "hash", "assinatura", "compilador", "teste", "prova",
    "mecanismo", "invariante", "circuito", "processo formal",
]
low = "logo,"
has_causal = any(re.search(r"(?<!\w)" + re.escape(c) + r"(?!\w)", low) for c in CAUSAL_CONNECTORS)
print("logo,: ", has_causal)

low2 = "produzir"
has_causal2 = any(re.search(r"(?<!\w)" + re.escape(c) + r"(?!\w)", low2) for c in CAUSAL_CONNECTORS)
print("produzir: ", has_causal2)

low3 = "ele produz alguma coisa"
has_causal3 = any(re.search(r"(?<!\w)" + re.escape(c) + r"(?!\w)", low3) for c in CAUSAL_CONNECTORS)
print("ele produz: ", has_causal3)
