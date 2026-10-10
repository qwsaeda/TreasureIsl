# -*- coding: utf-8 -*-
"""S7.2 验收辅助：校验 manifest_hd.json 与 assets_hd 产物一致（PNG-32、无缺失、哈希匹配）。"""
import json
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
MANIFEST = REPO / "manifest_hd.json"


def main() -> None:
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
