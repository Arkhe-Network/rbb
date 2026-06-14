with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "r") as f:
    lines = f.readlines()

new_lines = []
skip = False
for i, line in enumerate(lines):
    if skip:
        if "})" in line:
            skip = False
        continue
    if "doc = zvec.Doc(id=self.next_id, values={" in line:
        new_lines.append("        doc = zvec.Doc(id=str(self.next_id), vectors={\n")
        new_lines.append("            \"rssm_dense\": rssm_state.tolist(),\n")
        new_lines.append("            \"yolo_sparse\": sparse_vec,\n")
        new_lines.append("            \"action\": action.tolist()\n")
        new_lines.append("        }, fields={\n")
        new_lines.append("            \"reward\": reward\n")
        new_lines.append("        })\n")
        skip = True
        continue
    new_lines.append(line)

with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "w") as f:
    f.writelines(new_lines)
