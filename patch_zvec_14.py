with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "r") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if "query_sparse = {tag.lower(): 1.0 for tag in query_tags}" in line:
        lines[i] = "        # zVEC espera índices inteiros (UINT32) para vetores esparsos\n"
        lines[i] += "        # Mapeando strings para ints usando hash\n"
        lines[i] += "        query_sparse = {abs(hash(tag.lower())) % (2**32): 1.0 for tag in query_tags}\n"
    if "sparse_vec = {tag.lower(): 1.0 for tag in yolo_tags}" in line:
        lines[i] = "        sparse_vec = {abs(hash(tag.lower())) % (2**32): 1.0 for tag in yolo_tags}\n"

with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "w") as f:
    f.writelines(lines)
