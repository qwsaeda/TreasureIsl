# -*- coding: utf-8 -*-
"""S1.4b 辅助：统计 20 个 UI `.Map` 里每条控件记录的 f3（控件事件 id）分布。

布局依据（实测 + exe 双向确认）：
  [16B header: width,height,flags,reserved]
  words[0..4] = 表头 4 u32
  words[4]    = count
  words[5+6*i : 5+6*i+6] = 记录 i 的 f0..f5，**f3 = words[5+6*i+3]**
exe 侧：sub_44004 循环 `ReadFile(0x18)` 读 6×u32；sub_46294 `STR R2,[R0,#0x10]` 把 f3 落到 ctrl+0x10。
地形图 glmapm01/02/03（head 20×20/15×15）不适用本布局，单独跳过。
"""
import collections
import glob
import json
import os
import struct

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(ROOT, "data")
TERRAIN = {"glmapm01.Map", "glmapm02.Map", "glmapm03.Map"}


def parse(path):
    with open(path, "rb") as fh:
        raw = fh.read()
    head = struct.unpack_from("<4I", raw, 0)
    words = list(struct.unpack_from("<%dI" % ((len(raw) - 16) // 4), raw, 16))
    if len(words) < 5 or len(words) != 5 + 6 * words[4]:
        return None
    recs = []
    for i in range(words[4]):
        f = words[5 + 6 * i : 5 + 6 * i + 6]
        recs.append({"index": i, "f0": f[0], "f1": f[1], "f2": f[2],
                     "event_id": f[3], "f4": f[4], "f5": f[5]})
    return {"header": {"width": head[0], "height": head[1], "flags": head[2],
                       "reserved": head[3]},
            "table_head": words[0:4], "count": words[4], "records": recs}


def main():
    dist = collections.Counter()
    per_file = {}
    total = 0
    for path in sorted(glob.glob(os.path.join(DATA, "*.Map"))):
        name = os.path.basename(path)
        if name in TERRAIN:
            per_file[name] = {"skipped": "terrain layout (head 20x20/15x15)"}
            continue
        sc = parse(path)
        if sc is None:
            per_file[name] = {"skipped": "layout mismatch"}
            continue
        per_file[name] = {"count": sc["count"],
                          "table_head": sc["table_head"],
                          "header": sc["header"],
                          "records": sc["records"]}
        for r in sc["records"]:
            dist[r["event_id"]] += 1
            total += 1
    print("UI .Map 文件数 = %d  控件记录总数 = %d" % (len(per_file), total))
    print("不同 f3 取值数 = %d" % len(dist))
    print("\n=== f3 分布（值: 出现次数）===")
    for v, c in sorted(dist.items(), key=lambda kv: (kv[0] if kv[0] != 0xFFFFFFFF else -1)):
        tag = "  <- 哨兵 -1（无事件）" if v == 0xFFFFFFFF else ""
        print("  %12d : %4d%s" % (v if v != 0xFFFFFFFF else -1, c, tag))
    print("\n=== 每个文件的 f3 去重集合 ===")
    for name, info in per_file.items():
        if "records" not in info:
            print("  %-20s SKIP %s" % (name, info["skipped"]))
            continue
        ids = sorted({r["event_id"] for r in info["records"]})
        pretty = ["-1" if i == 0xFFFFFFFF else str(i) for i in ids]
        print("  %-20s n=%-4d %s" % (name, info["count"], ",".join(pretty)))


main()
