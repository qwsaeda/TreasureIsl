# -*- coding: utf-8 -*-
"""S1.4b 辅助：实测裁决 .Map 的记录级布局。

背景（冲突待裁决）：
- docs/reverse-facts.md F-306（S1.11 实测）：`words.len() == 5 + 6*count`，
  `count = words[4]`，`records_off = 32` 字节。
- ref/pseudocode/00044004_sub_44004.c（exe 侧 S0.2 导出）：读法是
  2×ReadFile(4B) → 1×ReadFile(0x18=24B) → 1×ReadFile(4B)=count → count×ReadFile(0x18)。

本脚本对全部 .Map 逐个试候选布局，只输出「能整除且 count 落在合理区间」的结果。
"""
import glob
import os
import struct
import sys

DATA = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "data")


def words_of(path):
    with open(path, "rb") as fh:
        raw = fh.read()
    assert len(raw) >= 16, path
    w = [raw[i : i + 4].hex() for i in range(0, 4, 4)]
    head = struct.unpack_from("<4I", raw, 0)
    body = struct.unpack_from("<%dI" % ((len(raw) - 16) // 4), raw, 16)
    return head, list(body)


def try_layout(body, count_idx, base):
    """count = body[count_idx]，记录起点 = base，返回是否整除吻合。"""
    if count_idx >= len(body):
        return None
    count = body[count_idx]
    if count > 100000:
        return None
    need = base + 6 * count
    return count, need, need == len(body)


def main():
    files = sorted(glob.glob(os.path.join(DATA, "*.Map")))
    cands = [("F-306: count=words[4], rec@5", 4, 5), ("A: count=words[6], rec@7", 6, 7),
             ("B: count=words[8], rec@9", 8, 9), ("C: count=words[4], rec@6", 4, 6)]
    tally = {name: 0 for name, _, _ in cands}
    detail = {name: [] for name, _, _ in cands}
    for path in files:
        head, body = words_of(path)
        rel = os.path.basename(path)
        for name, ci, base in cands:
            r = try_layout(body, ci, base)
            if r and r[2]:
                tally[name] += 1
                detail[name].append("%s(count=%d)" % (rel, r[0]))
    print("文件总数: %d" % len(files))
    print("\n=== 候选布局整除吻合统计 ===")
    for name, _, _ in cands:
        print("  %-28s %2d/%d" % (name, tally[name], len(files)))
    best = max(tally, key=lambda k: tally[k])
    print("\n=== 最佳候选: %s ===" % best)
    for name, ci, base in cands:
        if name == best:
            for d in detail[name]:
                print("   ", d)
    print("\n=== 逐文件前 12 个 word（words = 16B 头之后的 u32 流）===")
    for path in files:
        head, body = words_of(path)
        print("%-20s len=%3d head=%s" % (os.path.basename(path), len(body), list(head)))
        print("    words[0:12] = %s" % body[:12])


main()
