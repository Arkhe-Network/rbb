with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "r") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if "queries=[zvec.Query(field_name=\"rssm_dense\", vector=query_dense.tolist())]," in line:
        lines[i] = "            queries=[\n"
        lines[i] += "                zvec.Query(field_name=\"rssm_dense\", vector=query_dense.tolist()),\n"
        lines[i] += "                zvec.Query(field_name=\"yolo_sparse\", vector=query_sparse)\n"
        lines[i] += "            ],\n"
    if "topk=top_k" in line:
        lines[i] = "            topk=top_k,\n"
        lines[i] += "            reranker=zvec.RrfReRanker(60)\n"

with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "w") as f:
    f.writelines(lines)
