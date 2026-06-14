with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "r") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if "self.collection = zvec.open_collection(collection_name)" in line:
        lines[i] = "                self.collection = zvec.open(collection_name)\n"

with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "w") as f:
    f.writelines(lines)
