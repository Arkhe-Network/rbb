with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "r") as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    if "            \"action\": [float(action[0])] if hasattr(action, '__len__') and len(action) > 0 else [0.0] if hasattr(action, '__len__') and len(action) == 0 else [float(action)]\n" in line:
        new_lines.append("            \"action\": [float(action[0])] if hasattr(action, '__len__') else [float(action)]\n")
        continue
    new_lines.append(line)

with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "w") as f:
    f.writelines(new_lines)
