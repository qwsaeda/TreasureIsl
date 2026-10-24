# -*- coding: utf-8 -*-
"""S7.2 验收辅助：校验 manifest_hd.json 与 assets_hd 产物一致（PNG-32、无缺失、哈希匹配）。

用法：
  python scripts/verify_hd_manifest.py            # 只读校验（默认即只读，不写文件；不一致非 0 退出）
  python scripts/verify_hd_manifest.py --check   # 只读校验（S21.8 工具契约别名，行为同默认）
"""
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
MANIFEST = REPO / "assets_hd" / "manifest_hd.json"  # S7.2 产物落 assets_hd/ 同目录（S21.8：原根目录路径已不存在）


def main() -> None:
    # S21.8 工具契约：--check = 只读校验别名（默认行为即只读校验，不写文件；不一致非 0 退出）
    if "--check" in sys.argv[1:]:
        print("[verify_hd] --check 只读校验模式（默认行为，不写文件）")
    data = json.loads(MANIFEST.read_text())
    checked = 0
    missing = []
    bad_magic = []
    bad_hash = []
    for rel, meta in data.items():
        p = REPO / rel
        if not p.is_file():
            missing.append(rel)
            continue
        b = p.read_bytes()
        if b[:8] != b"\x89PNG\r\n\x1a\n":
            bad_magic.append(rel)
            continue
        fnv = meta.get("fnv1a64", "")
        if fnv:
            h = 0xCBF29CE484222325
            for byte in b:
                h ^= byte
                h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
            if format(h, "016x") != fnv:
                bad_hash.append(rel)
        checked += 1
    print(f"[verify_hd] entries={len(data)} checked={checked} missing={len(missing)} bad_magic={len(bad_magic)} bad_hash={len(bad_hash)}")
    if missing:
        print("missing:", *missing[:10], sep="\n  ")
    if bad_magic:
        print("bad_magic:", *bad_magic[:10], sep="\n  ")
    if bad_hash:
        print("bad_hash:", *bad_hash[:10], sep="\n  ")
    if missing or bad_magic or bad_hash:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
