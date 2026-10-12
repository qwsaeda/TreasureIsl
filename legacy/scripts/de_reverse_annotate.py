# -*- coding: utf-8 -*-
"""去逆向化标注清理 v3：把注释/文档中的 sub_xxxx 溯源标注替换为语义短名。

规则：
- 只动注释（//! / /// / 行内注释）和 .md 文档
- 不动 oracle 对拍逻辑代码
- 语义名从 ref/functions_*.csv 的 reason 列提取首个中文短语
- 缺失的 sub 保留原样（不猜测）

用法：python scripts/de_reverse_annotate.py [--dry-run]
"""
import glob
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def build_sub_semantic_map() -> dict[str, str]:
    """从 ref/functions_*.csv 构建 sub_xxxx → 语义短名映射。"""
    sub_map: dict[str, str] = {}
    ref_dir = os.path.join(REPO, "ref")
    for fname in sorted(os.listdir(ref_dir)):
        if not fname.startswith("functions"):
            continue
        with open(os.path.join(ref_dir, fname), encoding="utf-8") as fh:
            lines = fh.read().strip().split("\n")
        for line in lines[1:]:  # skip header
            parts = line.split(",")
            if len(parts) < 5:
                continue
            name = parts[2].strip()
            module = parts[3].strip()
            reason = parts[4].strip() if len(parts) > 4 else ""
            if not name.startswith("sub_"):
                continue
            # 从 reason 提取首个中文短语（跳过地址/英文）
            m = re.search(r"([一-鿿]{2,16})|([a-zA-Z]{4,16})", reason)
            if m:
                kw = m.group(1) or m.group(2)
                sub_map[name] = kw
            else:
                sub_map[name] = module

    # 手动补全：ref/functions_*.csv 未覆盖的 7 个 RNG 播种点
    manual = {
        "sub_15A20": "卡牌效果",
        "sub_18B44": "事件",
        "sub_1C238": "伤害",
        "sub_27A3C": "据点",
        "sub_29B80": "价格",
        "sub_34E20": "事件",
        "sub_36F14": "卡牌",
    }
    sub_map.update(manual)
    return sub_map


SUB_PATTERN = re.compile(r"sub_([0-9A-Fa-f]{4,6})\b")


def main() -> None:
    dry_run = "--dry-run" in sys.argv
    sub_map = build_sub_semantic_map()
    print(f"Mapped {len(sub_map)} sub entries")

    target_files: list[str] = []
    for pattern in [
        "crates/ti-core/src/rules/*.rs",
        "crates/ti-core/src/oracle/*.rs",
        "crates/ti-app/src/*.rs",
        "docs/impl/*.md",
        "docs/design/*.md",
    ]:
        for fp in glob.glob(os.path.join(REPO, pattern)):
            if os.path.isfile(fp):
                target_files.append(fp)

    total = 0
    all_missing: set[str] = set()
    for fp in target_files:
        with open(fp, encoding="utf-8") as fh:
            text = fh.read()

        count = 0
        missing: set[str] = set()

        def repl(m: re.Match) -> str:
            nonlocal count
            full = m.group(0)
            if full in sub_map:
                count += 1
                return sub_map[full]
            missing.add(full)
            return full

        new_text = SUB_PATTERN.sub(repl, text)

        if count > 0 or missing:
            if count > 0 and not dry_run:
                with open(fp, "w", encoding="utf-8") as fh:
                    fh.write(new_text)
            rel = os.path.relpath(fp, REPO)
            tag = "[dry-run] " if dry_run else ""
            miss_msg = f" (missing: {', '.join(sorted(missing))})" if missing else ""
            if count > 0:
                print(f"  {tag}{rel}: {count} refs{miss_msg}")
                total += count
            if missing:
                all_missing.update(missing)

    if all_missing:
        print(f"\nMissing ({len(all_missing)}): {', '.join(sorted(all_missing))}")
    print(f"\nTotal: {total} sub_xxxx refs replaced across files")
    if dry_run:
        print("(dry-run: no files written)")


if __name__ == "__main__":
    main()
