#!/usr/bin/env python3
"""Gate item: hd_f1_manifest.json == assets_hd_pack/1x/ui directory (EXP-048/060).

Usage:
  python scripts/check_hd_manifest.py          # fail on drift (non-zero exit)
  python scripts/check_hd_manifest.py --check   # same (gate convention)
  python scripts/check_hd_manifest.py --report  # print details, always exit 0
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "assets_hd_pack" / "hd_f1_manifest.json"
UI_DIR = ROOT / "assets_hd_pack" / "1x" / "ui"

STATIC_RE = re.compile(r"^(\d+)_hd$")
MULTI_RE = re.compile(r"^(\d+)_hd_f(\d+)$")


def main() -> int:
    report = "--report" in sys.argv
    if not MANIFEST.is_file() or not UI_DIR.is_dir():
        print(f"FAIL: missing {MANIFEST.name} or {UI_DIR.name} dir")
        return 0 if report else 1

    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    static_manifest = sorted(int(v) for v in manifest.get("static_f1", []))
    multi_manifest = sorted(int(v) for v in manifest.get("multiframe_f1", []))

    static_actual: set[int] = set()
    multi_actual: set[int] = set()
    for p in UI_DIR.iterdir():
        if not p.is_file() or p.suffix.lower() != ".png":
            continue
        m = STATIC_RE.match(p.stem)
        if m:
            static_actual.add(int(m.group(1)))
            continue
        m = MULTI_RE.match(p.stem)
        if m:
            multi_actual.add(int(m.group(1)))

    problems: list[str] = []
    if static_manifest != sorted(static_actual):
        problems.append(
            f"static_f1 manifest={len(static_manifest)} dir={len(static_actual)} "
            f"manifest-missing={sorted(set(static_actual) - set(static_manifest))[:10]} "
            f"manifest-extra={sorted(set(static_manifest) - set(static_actual))[:10]}"
        )
    if multi_manifest != sorted(multi_actual):
        problems.append(
            f"multiframe_f1 manifest={multi_manifest} dir={sorted(multi_actual)}"
        )

    for line in problems:
        print(f"FAIL: hd_f1_manifest.json drifted from {UI_DIR.as_posix()}: {line}")
    if not problems:
        print(
            f"OK: hd_f1_manifest.json matches {UI_DIR.as_posix()} "
            f"(static={len(static_actual)}, multiframe={len(multi_actual)})"
        )
    if problems and not report:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
