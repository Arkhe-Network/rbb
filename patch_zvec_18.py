with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "r") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if "doc = zvec.Doc(id=str(self.next_id), vectors={" in line:
        lines[i] = "        doc = zvec.Doc(id=str(self.next_id), fields={\n"
        lines[i] += "            \"id\": str(self.next_id),\n"
        lines[i] += "            \"reward\": reward\n"
        lines[i] += "        }, vectors={\n"
    if "}, fields={" in line:
        lines[i] = "        })\n"
    if "\"reward\": reward" in line and "        }, fields={" in lines[i-1]:
        lines[i] = ""
    if "        })" in line and "\"reward\": reward" in lines[i-1]:
        lines[i] = ""

with open("cathedral_arkhe_v16.3_zvec_meta_rust.py", "w") as f:
    f.writelines(lines)
