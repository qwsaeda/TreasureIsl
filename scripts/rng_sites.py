# -*- coding: utf-8 -*-
"""S3.12 子步1：定位 RNG 调用点并产出 ref/functions_rng.csv。

读 ref/functions.csv + ref/pseudocode/，统计全 exe 的 rand()/srand() 调用点，
按模块（cards/trade/battle/mainmap/saveload）归类，写 ref/functions_rng.csv。
不改动 ref/functions.csv 本体。

模块归属规则（best-effort，推测项在 notes 里标注）：
  cards    卡牌/事件：sub_3D4C8（卡牌 ctor）、sub_45E78（入口 wrapper）、
           sub_3C4CC/sub_3C9B8/sub_3B9D4/sub_3BB84/sub_3BC34/sub_3BFD0/
           sub_3C664/sub_3C784/sub_3C8E8/sub_32960/sub_49B94/sub_4BB34/
           sub_4D848/sub_51940
  trade    贸易：sub_49120、sub_47820、sub_47D40
  battle   战斗：sub_13CE4/sub_1409C/sub_14F90/sub_15368/sub_15538/
           sub_31518/sub_3169C/sub_327D0/sub_3289C/sub_32E00/sub_32E3C
  mainmap  主地图：sub_2101C
  saveload 存读档：sub_4E314/sub_4E568（srand 播种 + rand()%5）
"""
import csv
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CSV_PATH = os.path.join(ROOT, "ref", "functions.csv")
PC_DIR = os.path.join(ROOT, "ref", "pseudocode")
OUT = os.path.join(ROOT, "ref", "functions_rng.csv")

MOD = {
    "0003D4C8": "cards", "00045E78": "cards",
    "0003C4CC": "cards", "0003C9B8": "cards",
    "0003B9D4": "cards", "0003BB84": "cards", "0003BC34": "cards",
    "0003BFD0": "cards", "0003C664": "cards", "0003C784": "cards",
    "0003C8E8": "cards", "00032960": "cards", "00049B94": "cards",
    "0004BB34": "cards", "0004D848": "cards", "00051940": "cards",
    "00049120": "trade", "00047820": "trade", "00047D40": "trade",
    "00013CE4": "battle", "0001409C": "battle", "00014F90": "battle",
    "00015368": "battle", "00015538": "battle", "00031518": "battle",
    "0003169C": "battle", "000327D0": "battle", "0003289C": "battle",
    "00032E00": "battle", "00032E3C": "battle",
    "0002101C": "mainmap",
    "0004E314": "saveload", "0004E568": "saveload",
    # CRT thunks（EXP-037：游戏不自带 RNG，__imp_* 是 coredll 导入）
    "000585E0": "crt", "00058634": "crt",
}


def main():
    rows = list(csv.DictReader(open(CSV_PATH, encoding="utf-8")))
    by_addr = {r["addr"].upper(): r for r in rows}

    lines = ["kind,site,func,module,size,notes"]
    seen = set()
    stats = {"rand": 0, "srand": 0, "thunk": 0}

    for f in sorted(os.listdir(PC_DIR)):
        m = re.match(r"^([0-9A-Fa-f]{8})_(\w+)\.c$", f)
        if not m:
            continue
        addr = m.group(1).upper()
        r = by_addr.get(addr)
        if r is None:
            continue
        text = open(os.path.join(PC_DIR, f), encoding="utf-8", errors="replace").read()
        nrand = len(re.findall(r"\brand\s*\(", text))
        nsrand = len(re.findall(r"\bsrand\s*\(", text))
        nmod = len(re.findall(r"__imp_rand", text))
        mod = MOD.get(addr, "unresolved")
        if nrand:
            stats["rand"] += nrand
            notes = "rand() x%d" % nrand
            if mod == "unresolved":
                notes += " (推测)"
            lines.append(
                "rand,%s,%s,%s,%s,%s" % (addr.lower(), r["name"], mod, r["size"], notes)
            )
            seen.add(addr)
        if nsrand:
            stats["srand"] += nsrand
            lines.append(
                "srand,%s,%s,%s,%s,seed=wMinute+wSecond+wMilliseconds (ms级时间播种)"
                % (addr.lower(), r["name"], mod, r["size"])
            )
            seen.add(addr)
        if nmod:
            stats["thunk"] += 1
            lines.append(
                "thunk,%s,%s,crt,%s,__imp_rand core导入 (EXP-037: 游戏不自带RNG)"
                % (addr.lower(), r["name"], r["size"])
            )
            seen.add(addr)

    with open(OUT, "w", encoding="utf-8", newline="") as fh:
        fh.write("\n".join(lines) + "\n")

    print("=== S3.12 substep1: RNG call sites ===")
    print("rows=%d  rand_calls=%d  srand_sites=%d  thunks=%d" % (
        len(lines) - 1, stats["rand"], stats["srand"], stats["thunk"]))
    mods = {}
    for line in lines[1:]:
        p = line.split(",")
        if p[0] in ("rand", "srand"):
            mods[p[3]] = mods.get(p[3], 0) + 1
    for k in sorted(mods):
        print("  %-12s %d" % (k, mods[k]))
    print("output=%s" % os.path.relpath(OUT, ROOT))


if __name__ == "__main__":
    main()
