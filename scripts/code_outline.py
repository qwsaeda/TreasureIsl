#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""源结构清单：文件 / 顶层项 / 方法（含行范围）—— 拆分与重构前的「模块边界 + 函数清单」工具。

规范依据（`docs/process/02_conventions.md` §9.1）：动手前先产出**模块边界与函数清单**，
据此机械搬移，避免「先切文件再想边界」造成的返工（EXP-112）。

用法：
  python scripts/code_outline.py crates/ti-app/src/game_app.rs        # 单文件清单（顶层项 + 方法）
  python scripts/code_outline.py --crate ti-app                        # 全 crate 概览（文件 × 顶层项 × 方法数 × 最大方法）
  python scripts/code_outline.py --crate ti-app --json                 # 机器可读（供其它工具消费）
  python scripts/code_outline.py --check                               # 校验：清单可解析 + 无「孤立顶层 fn」

约定：幂等 + 只读 + `--check` 非 0 退出（`docs/process/04_tooling.md` §1 新增工具要求）。
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

ITEM_RE = re.compile(
    r'^(pub(?:\([^)]*\))?\s+)?(struct|impl|fn|enum|const|static|type|mod|trait)\b'
)
METHOD_RE = re.compile(r'^    (pub(?:\([^)]*\))?\s+)?(fn|const)\s+(\w+)')
MOD_RE = re.compile(r'^\s*(pub(?:\([^)]*\))?\s+)?mod\s+(\w+)')


def strip_noise(line: str) -> str:
    """去掉字符串/字符字面量/行注释里的花括号（配平用）。"""
    out = []
    i = 0
    n = len(line)
    while i < n:
        c = line[i]
        if c == '/' and i + 1 < n and line[i + 1] == '/':
            break
        if c == '"':
            i += 1
            while i < n:
                if line[i] == '\\':
                    i += 2
                    continue
                if line[i] == '"':
                    i += 1
                    break
                i += 1
            out.append('""')
            continue
        out.append(c)
        i += 1
    return ''.join(out)


def include_docs(lines: list[str], start: int) -> int:
    """向上吞掉连续的 `///` / `#[...]` 行（方法的文档与属性属于方法本身）。"""
    i = start - 1
    while i >= 0:
        s = lines[i].strip()
        if s.startswith('///') or s.startswith('#[') or s.startswith('#!['):
            i -= 1
        else:
            break
    return i + 1


def extent(lines: list[str], start: int) -> int:
    """从 start 起按花括号配平找该项结束行（0-based 闭区间）。"""
    d = 0
    seen = False
    j = start
    while j < len(lines):
        clean = strip_noise(lines[j])
        d += clean.count('{') - clean.count('}')
        if '{' in clean:
            seen = True
        if seen and d <= 0:
            return j
        j += 1
    return len(lines) - 1


def outline_file(path: str) -> dict:
    """单文件清单：顶层项（含 impl 内方法）。"""
    rel = os.path.relpath(path, ROOT).replace('\\', '/')
    lines = open(path, encoding='utf-8', errors='replace').read().splitlines()
    items = []
    methods = []
    orphan_fns = []
    i = 0
    while i < len(lines):
        m = ITEM_RE.match(lines[i])
        if not m:
            i += 1
            continue
        kind = m.group(2)
        start = include_docs(lines, i)
        if kind in ('struct', 'impl', 'enum', 'mod', 'trait'):
            end = extent(lines, i)
        else:
            end = i
            while end < len(lines) and not lines[end].rstrip().endswith((';', ',')):
                end += 1
        head = lines[i].strip()[:80]
        items.append({
            'kind': kind,
            'line': start + 1,
            'end': end + 1,
            'lines': end - start + 1,
            'head': head,
        })
        if kind == 'impl':
            j = i + 1
            while j < end:
                mm = METHOD_RE.match(lines[j])
                if mm:
                    mstart = include_docs(lines, j)
                    if mm.group(2) == 'fn':
                        mend = extent(lines, j)
                    else:
                        mend = j
                        while mend < end and not lines[mend].rstrip().endswith(';'):
                            mend += 1
                    methods.append({
                        'name': mm.group(3),
                        'line': mstart + 1,
                        'lines': mend - mstart + 1,
                    })
                    j = mend + 1
                    continue
                j += 1
        elif kind == 'fn' and not lines[i].lstrip().startswith('pub fn'):
            orphan_fns.append({'name': m.group(0)[:40], 'line': start + 1})
        i = end + 1
    return {
        'file': rel,
        'lines': len(lines),
        'items': items,
        'methods': methods,
        'orphan_fns': orphan_fns,
    }


def crate_files(crate: str) -> list[str]:
    base = os.path.join(ROOT, 'crates', crate)
    out = []
    for dp, dn, fn in os.walk(base):
        if 'target' in dp:
            continue
        for f in fn:
            if f.endswith('.rs'):
                out.append(os.path.join(dp, f))
    return sorted(out)


def main() -> int:
    ap = argparse.ArgumentParser(description='源结构清单（模块边界 + 函数清单）')
    ap.add_argument('path', nargs='?', help='单个 .rs 文件')
    ap.add_argument('--crate', help='整 crate 概览')
    ap.add_argument('--json', action='store_true', help='机器可读输出')
    ap.add_argument('--check', action='store_true', help='校验：清单可解析 + 无孤立顶层 fn')
    args = ap.parse_args()

    if not args.path and not args.crate:
        ap.print_help()
        return 2

    results = []
    if args.path:
        p = args.path if os.path.isabs(args.path) else os.path.join(ROOT, args.path)
        if not os.path.exists(p):
            print('文件不存在：%s' % args.path, file=sys.stderr)
            return 2
        results.append(outline_file(p))
    else:
        for p in crate_files(args.crate):
            results.append(outline_file(p))

    if args.json:
        print(json.dumps(results, ensure_ascii=False, indent=1))
    else:
        for r in results:
            print('=== %s（%d 行）===' % (r['file'], r['lines']))
            for it in r['items']:
                print('  %-7s %5d..%-5d (%4d)  %s'
                      % (it['kind'], it['line'], it['end'], it['lines'], it['head']))
            for m in r['methods']:
                print('      fn %-34s 行 %5d（%d 行）' % (m['name'], m['line'], m['lines']))

    if args.check:
        errors = []
        for r in results:
            if r['lines'] == 0:
                errors.append('%s 为空' % r['file'])
            for o in r['orphan_fns']:
                errors.append('%s:%d 孤立顶层 fn（不在任何 impl 内）：%s'
                              % (r['file'], o['line'], o['name']))
        if errors:
            for e in errors:
                print('[code_outline] FAIL：%s' % e, file=sys.stderr)
            return 1
        print('[code_outline] PASS：%d 个文件清单可解析，无孤立顶层 fn' % len(results))
    return 0


if __name__ == '__main__':
    sys.exit(main())
