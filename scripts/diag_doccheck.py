# -*- coding: utf-8 -*-
"""诊断 7 个 evidence 文件缺哪些 dod 关键字（doccheck 报错根因）。"""
import os
import re

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def dod_keywords(dod: str) -> list[str]:
    """复制 doccheck.rs 的 dod_keywords 逻辑。"""
    out: set[str] = set()
    rest = dod
    while True:
        start = rest.find("`")
        if start < 0:
            break
        after = rest[start + 1:]
        end = after.find("`")
        if end < 0:
            break
        frag = after[:end].strip()
        if len(frag) >= 6:
            out.add(frag)
        rest = after[end + 1:]
    for word in re.split(r"[^a-zA-Z0-9_]", dod):
        if len(word) >= 5 and word not in ("cargo", "powershell", "python"):
            out.add(word)
    return sorted(out)


def main() -> None:
    with open(os.path.join(REPO, "plan/ledger.yaml"), encoding="utf-8") as f:
        ledger = f.read()

    ids = ["S3.3", "S3.4", "S3.7", "S3.8", "S3.10", "S3.11", "S7.6"]
    for tid in ids:
        m = re.search(rf"id: {re.escape(tid)},.*?dod: (\"[^\"]+\")", ledger)
        if not m:
            print(f"{tid}: dod NOT FOUND")
            continue
        dod = m.group(1).strip().strip('"')
        keywords = dod_keywords(dod)

        ev_path = os.path.join(REPO, f"evidence/{tid}.log")
        with open(ev_path, encoding="utf-8") as f:
            text = f.read()

        found = [k for k in keywords if k and k in text]
        missing = [k for k in keywords if k and k not in text]

        print(f"=== {tid} ===")
        print(f"  dod: {dod[:100]}")
        print(f"  keywords ({len(keywords)}): {keywords[:10]}")
        print(f"  found: {found[:5]}")
        print(f"  MISSING: {missing[:5]}")
        print()


if __name__ == "__main__":
    main()
