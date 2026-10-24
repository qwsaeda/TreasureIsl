# -*- coding: utf-8 -*-
"""CPU\u2194Bevy **\u73b0\u573a**\u53cc\u540e\u7aef\u771f\u5b9e\u573a\u666f\u5bf9\u62cd\uff08\u65e0 golden \u57fa\u7ebf\uff09\u3002

\u5bf9\u6bd4\u4e24\u4e2a**\u5b9e\u65f6\u6e32\u67d3\u5668**\u7684\u8f93\u51fa\uff08\u4e0d\u662f\u4e0e\u5b58\u50a8\u7684 golden PNG \u6bd4\uff09\uff1a
- CPU\uff1a`cargo run -q -p ti-app --features native-shell --bin render-png` \u2192 `evidence/render_png/<scene>.png`\uff081080\u00d71920\uff09
- Bevy\uff1a`cargo run --manifest-path crates/ti-engine/Cargo.toml --bin treasure-bevy-native -- --scene=<id> --shot` \u2192 `evidence/bevy_native_<id>.png`

\u7528\u9014\uff1a\u91cf\u5316\u53cc\u540e\u7aef\u5728 7 \u4e2a\u771f\u5b9e\u573a\u666f\u7684\u50cf\u7d20\u4e00\u81f4\u6027\uff0c\u5e76\u533a\u5206\u5dee\u5f02\u7c7b\u578b\uff1a
- **\u5b57\u4f53\u5149\u6805\u5316\u6b8b\u5dee**\uff08CPU ab_glyph vs Bevy cosmic-text\uff09\u2192 \u9010\u884c\u5c0f\u5e26\u5f62\u72b6\u5dee\u5f02\uff08\u6bcf\u884c\u5dee\u5f02\u5bbd\u5ea6\u5c0f\uff09\uff1b
- **\u7ed3\u6784\u5dee\u5f02**\uff08\u7f3a\u5143\u7d20/\u9519\u4f4d\uff09\u2192 \u5927\u9762\u79ef\u8fde\u901a\u533a\u5dee\u5f02\uff08\u6bcf\u884c\u5dee\u5f02\u5bbd\u5ea6\u5927\uff09\u3002

\u7528\u6cd5\uff1a`python scripts/bevy_cpu_scene_diff.py`\uff08\u81ea\u52a8\u91cd\u751f\u4e24\u8fb9\u8f93\u51fa\uff0c\u5199 `evidence/bevy_cpu_scene_diff.md`\uff09\u3002
"""
import io, os, struct, subprocess, sys, zlib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCENES = ["index", "select_role", "select_map", "main_map", "vs", "message", "saveload"]
CPU_PNG = os.path.join(ROOT, "evidence", "render_png")
BEVY_PNG = os.path.join(ROOT, "evidence")


def decode(path):
    d = io.open(path, "rb").read()
    i, idat, w, h, ct = 8, b"", 0, 0, 0
    while i < len(d):
        ln = struct.unpack(">I", d[i:i + 4])[0]
        typ = d[i + 4:i + 8]
        ch = d[i + 8:i + 8 + ln]
        if typ == b"IHDR":
            w, h, _bd, ct = struct.unpack(">IIBB", ch[:10])
        elif typ == b"IDAT":
            idat += ch
        i += 12 + ln
    raw = zlib.decompress(idat)
    bpp = 4 if ct == 6 else 3
    stride = w * bpp
    out = bytearray()
    prev = bytearray(stride)
    p = 0
    for _y in range(h):
        f = raw[p]; p += 1
        line = bytearray(raw[p:p + stride]); p += stride
        if f == 1:
            for x in range(bpp, stride):
                line[x] = (line[x] + line[x - bpp]) & 255
        elif f == 2:
            for x in range(stride):
                line[x] = (line[x] + prev[x]) & 255
        elif f == 3:
            for x in range(stride):
                a = line[x - bpp] if x >= bpp else 0
                line[x] = (line[x] + ((a + prev[x]) >> 1)) & 255
        elif f == 4:
            for x in range(stride):
                a = line[x - bpp] if x >= bpp else 0
                b = prev[x - bpp] if x >= bpp else 0
                c = prev[x]
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[x] = (line[x] + pr) & 255
        out += line
        prev = line
    return w, h, bpp, bytes(out)


def run(cmd):
    subprocess.run(cmd, cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)


def ensure(sid):
    cpu = os.path.join(CPU_PNG, f"{sid}.png")
    if not os.path.isfile(cpu):
        run(["cargo", "run", "-q", "-p", "ti-app", "--features", "native-shell", "--bin", "render-png"])
    bev = os.path.join(BEVY_PNG, f"bevy_native_{sid}.png")
    if not os.path.isfile(bev):
        run(["cargo", "run", "-q", "--manifest-path", "crates/ti-engine/Cargo.toml",
             "--bin", "treasure-bevy-native", "--", f"--scene={sid}", "--shot"])
    return cpu, bev


def compare(cpu, bev):
    w1, h1, b1, p1 = decode(bev)
    w2, h2, b2, p2 = decode(cpu)
    assert (w1, h1) == (w2, h2), f"size {w1}x{h1} vs {w2}x{h2}"
    total = w1 * h1
    diff = 0
    maxd = 0
    rows = {}
    for y in range(h1):
        c = 0
        base = y * w1
        for x in range(w1):
            o1 = (base + x) * b1
            o2 = (base + x) * b2
            dr = abs(p1[o1] - p2[o2])
            dg = abs(p1[o1 + 1] - p2[o2 + 1])
            db = abs(p1[o1 + 2] - p2[o2 + 2])
            m = max(dr, dg, db)
            if m > 2:
                c += 1
                if m > maxd:
                    maxd = m
        if c:
            diff += c
            rows[y] = c
    wide = sum(1 for v in rows.values() if v > w1 * 0.6)
    return {"diff": diff, "total": total, "ratio": diff / total, "maxd": maxd,
            "rows": len(rows),
            "top": min(rows) if rows else -1, "bot": max(rows) if rows else -1,
            "wide_rows": wide}


def check_mode():
    """S21.8 tooling-contract readonly mode: compare existing PNG pairs only (no regen, no report write; missing output => non-zero exit)."""
    print("# bevy_cpu_scene_diff --check (readonly: existing PNG pairs only, nothing written)")
    print("| scene | diff px | diff % | max ch diff | diff rows | row span | wide rows (>=60% width) |")
    print("|---|---|---|---|---|---|---|")
    missing = 0
    for sid in SCENES:
        cpu, bev = os.path.join(CPU_PNG, f"{sid}.png"), os.path.join(BEVY_PNG, f"bevy_native_{sid}.png")
        if not (os.path.isfile(cpu) and os.path.isfile(bev)):
            missing += 1
            print(f"| {sid} | output missing | - | - | - | - | - |")
            continue
        m = compare(cpu, bev)
        print(f"| {sid} | {m['diff']} / {m['total']} | {m['ratio']*100:.3f}% | {m['maxd']} | {m['rows']} | "
              f"y {m['top']}..{m['bot']} | {m['wide_rows']} |")
    if missing:
        print(f"bevy_cpu_scene_diff --check: FAIL ({missing} scene outputs missing; run default mode first)")
        return 1
    print("bevy_cpu_scene_diff --check: OK (7 scene pairs compared, readonly, nothing written)")
    return 0


def main():
    if "--check" in sys.argv:
        return check_mode()
    lines = ["# S18.19\uff1aCPU\u2194Bevy \u73b0\u573a\u53cc\u540e\u7aef\u771f\u5b9e\u573a\u666f\u5bf9\u62cd\uff08\u65e0 golden \u57fa\u7ebf\uff09", "",
             "\u5bf9\u6bd4\u4e24\u4e2a**\u5b9e\u65f6\u6e32\u67d3\u5668**\uff1aCPU `render-png` bin \u2192 `evidence/render_png/<scene>.png`\uff1b",
             "Bevy `treasure-bevy-native --scene=<id> --shot` \u2192 `evidence/bevy_native_<id>.png`\u3002\u5de5\u5177\uff1a`scripts/bevy_cpu_scene_diff.py`\u3002",
             "", "| \u573a\u666f | \u5dee\u5f02\u50cf\u7d20 | \u5dee\u5f02\u7387 | \u6700\u5927\u901a\u9053\u5dee | \u5dee\u5f02\u884c\u6570 | \u5dee\u5f02\u884c\u533a\u95f4 | \u5bbd\u5e26\u884c\uff08\u2265 60% \u5bbd\uff09 |",
             "|---|---|---|---|---|---|---|"]
    for sid in SCENES:
        cpu, bev = ensure(sid)
        if not (os.path.isfile(cpu) and os.path.isfile(bev)):
            lines.append(f"| {sid} | \u8f93\u51fa\u7f3a\u5931 | - | - | - | - | - |")
            continue
        m = compare(cpu, bev)
        lines.append(f"| {sid} | {m['diff']} / {m['total']} | {m['ratio']*100:.3f}% | {m['maxd']} | {m['rows']} | "
                     f"y {m['top']}..{m['bot']} | {m['wide_rows']} |")
    lines += ["", "\u8bfb\u6cd5\uff1a\u300c\u5bbd\u5e26\u884c\u300d= \u5dee\u5f02\u50cf\u7d20\u5360\u884c\u5bbd \u2265 60% \u7684\u884c\u6570\uff08\u5927\u9762\u79ef\u8fde\u901a\u533a \u2192 \u7ed3\u6784\u5dee\u5f02\uff09\uff1b",
              "\u5c11\u91cf\u5c0f\u5e26\u5f62\u72b6\u5dee\u5f02\uff08\u5bbd\u5e26\u884c \u2248 0\uff09\u2192 **\u5b57\u4f53\u5149\u6805\u5316\u6b8b\u5dee**\uff08CPU ab_glyph vs Bevy cosmic-text\uff09\u3002"]
    io.open(os.path.join(ROOT, "evidence", "bevy_cpu_scene_diff.md"), "w",
            encoding="utf-8", newline="\n").write("\n".join(lines) + "\n")
    print("\n".join(lines[6:]))
    return 0


if __name__ == "__main__":
    sys.exit(main())