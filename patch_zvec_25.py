with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "r") as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    if "            \"action\": action.tolist() if isinstance(action.tolist(), list) else [float(action)]\n" in line:
        new_lines.append("            \"action\": [float(action)] if np.isscalar(action) or action.ndim == 0 else action.tolist()\n")
        continue
    new_lines.append(line)

with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "w") as f:
    f.writelines(new_lines)
