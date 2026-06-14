with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "r") as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    if i == 109 and "\"reward\": reward" in line:
        continue
    new_lines.append(line)

with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "w") as f:
    f.writelines(new_lines)
