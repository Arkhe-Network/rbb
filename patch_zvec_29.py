with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "r") as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    if "            \"action\": [float(action[0])] if hasattr(action, '__len__') else [float(action)]\n" in line:
        new_lines.append("            \"action\": action.tolist() if isinstance(action.tolist(), list) else [float(action)]\n")
        continue
    if "doc = zvec.Doc(id=str(self.next_id), fields={" in line:
        new_lines.append("        doc = zvec.Doc(id=str(self.next_id), fields={\n")
        new_lines.append("            \"reward\": reward\n")
        new_lines.append("        }, vectors={\n")
        new_lines.append("            \"rssm_dense\": rssm_state.tolist(),\n")
        new_lines.append("            \"yolo_sparse\": sparse_vec,\n")
        new_lines.append("            \"action\": action.tolist() if isinstance(action.tolist(), list) else [float(action)]\n")
        new_lines.append("        })\n")
        continue
    if any(x in line for x in [
        "            \"id\": self.next_id,\n",
        "            \"reward\": reward\n",
        "        }, vectors={\n",
        "            \"rssm_dense\": rssm_state.tolist(),\n",
        "            \"yolo_sparse\": sparse_vec,\n",
        "        })\n"
    ]) and i > 100 and i < 115:
        continue
    new_lines.append(line)

with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "w") as f:
    f.writelines(new_lines)
