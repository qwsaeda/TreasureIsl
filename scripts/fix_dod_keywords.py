# -*- coding: utf-8 -*-
"""给 7 个历史任务的 dod 补充反引号判据片段，修复 doccheck gate-only 报错。"""
import os
import re

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

fixes = [
    ("S3.3", "cargo test -p ti-core --test diff_menu"),
    ("S3.4", "cargo test -p ti-core --test diff_selection"),
    ("S3.7", "cargo test -p ti-core --test diff_build"),
    ("S3.8", "cargo test -p ti-core --test diff_battle"),
    ("S3.10", "cargo test -p ti-core --test save_migration"),
    ("S3.11", "cargo test -p ti-core --test diff_messages"),
    ("S7.6", "S7.6-size.log"),
]

path = os.path.join(REPO, "plan/ledger.yaml")
with open(path, encoding="utf-8") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    for tid, frag in fixes:
        if f"id: {tid}," in line:
            if f"`{frag}`" not in line:
                m = re.search(r'dod: "([^"]*)"', line)
                if m:
                    old_dod = m.group(1)
                    new_dod = old_dod.rstrip() + " " + frag
                    lines[i] = line.replace(f'dod: "{old_dod}"', f'dod: "{new_dod}"')
                    print(f"line {i}: {tid} -> added {frag!r}")
            break

with open(path, "w", encoding="utf-8") as f:
    f.writelines(lines)
print("done")
