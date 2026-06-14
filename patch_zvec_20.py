with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "r") as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    if "zvec.FieldSchema(\"id\", zvec.DataType.STRING)" in line:
        new_lines.append("                zvec.FieldSchema(\"id\", zvec.DataType.UINT64),\n")
        continue
    if "doc = zvec.Doc(id=str(self.next_id), fields={" in line:
        new_lines.append("        doc = zvec.Doc(id=str(self.next_id), fields={\n")
        continue
    if "\"id\": str(self.next_id)," in line:
        new_lines.append("            \"id\": self.next_id,\n")
        continue
    new_lines.append(line)

with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "w") as f:
    f.writelines(new_lines)
