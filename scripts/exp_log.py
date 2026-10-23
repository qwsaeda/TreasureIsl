#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""EXP 条目一条命令登记（分卷正文 + 索引行 + 分卷条数列，3 处一次改齐）。

**为什么有这个工具**（`docs/process/15_process_ledger.md` 第 2 条流程）：
写一条 EXP 过去要手改 3 处 —— `docs/experience/<volume>.md` 正文 + `docs/experience/index.md`
索引行 + 同文件分卷表的「条数」列。漏一处 ⇒ 经验库失声（EXP-108：索引长期缺 6 行、
code.md 声明 28 实际 33 无人发现；`check_experience_volume_counts` 曾恒不触发）。

用法：
    python scripts/exp_log.py --volume code --id EXP-116 --title "标题" --summary "索引摘要" --body "正文段落"
    python scripts/exp_log.py --volume code --id EXP-116 --title "标题" --summary "摘要" --body-file notes.md
    python scripts/exp_log.py --check --id EXP-116        # 只读：查重 + 校验分卷条数

行为：
- **查重**：`EXP-NNN` 在整个 `docs/experience/` 出现 >1 处 ⇒ 拒绝（ID 唯一，EXP-063）；
- **正文**：`### EXP-NNN <volume标签>：<title>` 段落追加到分卷文件末尾；
- **索引行**：`docs/experience/index.md` 索引表追加 `| EXP-NNN | <volume> | | <summary> 正文见 `<volume>.md` |`；
- **条数列**：分卷表该卷「条数」+1（从分卷文件实际 `### EXP-` 标题数**实测**，不是 +1 猜）；
- **收尾复验**：跑 `doccheck`（ID 唯一 + 跨文档引用有效 + 分卷条数对账）。

退出码：0 = 成功；1 = 查重/缺参/对账失败。幂等：同 id 再跑 = 查重拦截（不重复写）。
"""
from __future__ import annotations

import argparse
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
EXP_DIR = ROOT / "docs" / "experience"
INDEX = EXP_DIR / "index.md"
VOLUMES = ("code", "process", "build", "tools", "formats", "reverse")
VOLUME_TITLE = {
    "code": "代码",
    "process": "流程",
    "build": "构建",
    "tools": "工具",
    "formats": "格式",
    "reverse": "逆向",
}


def write_text(path: pathlib.Path, text: str) -> None:
    path.write_text(text, encoding="utf-8", newline="")


def occurrences(exp_id: str) -> list[str]:
    hits: list[str] = []
    for p in sorted(EXP_DIR.glob("*.md")):
        for i, line in enumerate(p.read_text(encoding="utf-8").splitlines(), start=1):
            if exp_id in line:
                hits.append(f"{p.name}:{i}")
    return hits


def volume_count(volume: str) -> int:
    text = (EXP_DIR / f"{volume}.md").read_text(encoding="utf-8")
    return len(re.findall(r"^### EXP-\d+", text, re.M))


def main() -> int:
    ap = argparse.ArgumentParser(description="EXP 条目一条命令登记（正文 + 索引行 + 条数列）")
    ap.add_argument("--volume", required=True, choices=VOLUMES)
    ap.add_argument("--id", required=True, help="EXP 编号（如 EXP-116）")
    ap.add_argument("--title", help="标题（正文标题行）")
    ap.add_argument("--summary", help="索引行摘要（一句话）")
    ap.add_argument("--body", help="正文段落（可多行）")
    ap.add_argument("--body-file", help="正文段落文件（UTF-8）")
    ap.add_argument("--check", action="store_true", help="只读：查重 + 报分卷条数")
    args = ap.parse_args()

    if not re.fullmatch(r"EXP-\d+", args.id):
        print(f"EXP 编号格式须 EXP-NNN：{args.id}", file=sys.stderr)
        return 1

    hits = occurrences(args.id)
    if args.check:
        print(f"  {args.id} 出现 {len(hits)} 处: {hits or '无'}")
        for v in VOLUMES:
            print(f"    {v}.md: {volume_count(v)} 条")
        return 1 if len(hits) > 1 else 0
    if len(hits) > 1:
        print(f"EXP 编号重复（{len(hits)} 处）：{hits}", file=sys.stderr)
        return 1
    if len(hits) == 1:
        print(f"{args.id} 已存在（{hits[0]}）⇒ 不重复登记", file=sys.stderr)
        return 1
    if not args.title or not args.summary:
        print("需 --title 与 --summary", file=sys.stderr)
        return 1
    body = args.body or ""
    if args.body_file:
        body = (ROOT / args.body_file).read_text(encoding="utf-8")
    if not body.strip():
        body = "- （正文待补）"

    # ① 正文：追加到分卷文件
    vol_path = EXP_DIR / f"{args.volume}.md"
    text = vol_path.read_text(encoding="utf-8")
    if not text.endswith("\n"):
        text += "\n"
    section = f"\n### {args.id} {VOLUME_TITLE[args.volume]}：{args.title}\n{body.rstrip()}\n"
    write_text(vol_path, text + section)
    print(f"  正文已追加: docs/experience/{args.volume}.md")

    # ② 索引行：插到索引表末尾（"## 条目格式" 之前）
    idx = INDEX.read_text(encoding="utf-8")
    row = (
        f"| {args.id} | {args.volume} | | {args.summary} 正文见 `{args.volume}.md` |"
    )
    marker = "\n## 条目格式"
    if marker in idx:
        head, tail = idx.split(marker, 1)
        idx = head.rstrip("\n") + "\n" + row + "\n" + marker + tail
    else:
        idx = idx.rstrip("\n") + "\n" + row + "\n"
    write_text(INDEX, idx)
    print(f"  索引行已追加: docs/experience/index.md")

    # ③ 分卷条数列：从分卷文件实测标题数回填（不是 +1 猜）
    n = volume_count(args.volume)
    pat = re.compile(rf"(\|\s*\[`{re.escape(args.volume)}\.md`\]\([^)]*\)\s*\|[^|]*\|\s*)(\d+)(\s*\|)")
    m = pat.search(idx)
    if m:
        idx = pat.sub(lambda mm: f"{mm.group(1)}{n}{mm.group(3)}", idx, count=1)
        write_text(INDEX, idx)
        print(f"  分卷条数列已回填: {args.volume} = {n}")

    # 收尾复验：doccheck（ID 唯一 + 引用有效 + 分卷条数对账）
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "ti-tools", "--bin", "doccheck"],
        cwd=ROOT,
        capture_output=True,
        encoding="utf-8",
        errors="replace",
    )
    tail = (proc.stdout or "").strip().splitlines()[-1:] or [(proc.stderr or "").strip()]
    print(f"  doccheck: {tail[0] if tail else '(无输出)'}")
    return proc.returncode


if __name__ == "__main__":
    sys.exit(main())
