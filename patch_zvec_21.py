with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "r") as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    if "return [{\"id\": r.id, \"score\": r.distance, \"action\": r.entity.get(\"action\"), \"reward\": r.entity.get(\"reward\")} for r in results]" in line:
        new_lines.append("        return [{\"id\": r.id, \"score\": r.score, \"action\": r.vector(\"action\"), \"reward\": r.field(\"reward\")} for r in results]\n")
        continue
    new_lines.append(line)

with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "w") as f:
    f.writelines(new_lines)
