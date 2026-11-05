#!/usr/bin/env python3
"""APK 体积构成审计（S35.0 规范 = docs/process/24_apk_size_std.md）。

用途
----
① 构建侧（`build_apk.sh` / `build_wasm.ps1`）调 `--refset` 拿「引擎实际引用的资产清单」，
   打包只带引用件（未引用 UI 图不进产物）；
② 体检侧 `--apk dist/...apk` 出体积构成表（按目录聚合 压缩后/原始 大小 + 每类压缩率），
   作为平台「APK 体积」页数据源 + 发布前置判据；
③ 退出码：`--refset`/`--apk` 成功 = 0；APK 缺 lib 段或 .so 为 Stored（未压缩）= 非 0（判据 = S35.0 标准①）。

口径（单一事实源 = docs/process/24_apk_size_std.md）
----------------------------------------------------
- 引用集 = `crates/ti-{engine,app}` 字符串字面量 + `format!` 动态前缀（icon_/isle_/island_home_/
  role_ 全带：运行时按名 blit、缺图回退 ⇒ 砍任一 = 视觉回归）+ `assets/data` 岛 kind +
  地图包 POOL_INDEX + `registry.json` audio `path`（音频 4 项恒带）；
- .so 判据 = APK 内 `lib/<abi>/libandroid_activity.so` 压缩率 < 30% ⇒ FAIL（Stored = 0% 压缩）；
- UI 判据 = 引用集外的 `assets/ui/*.png` 不得进 APK（进即漂移，FAIL）。

只读：本脚本 0 写盘（`--refset` 打印到 stdout）。
"""
from __future__ import annotations

import json
import os
import re
import sys
import zipfile

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LIT_RE = re.compile(r'"(assets/[A-Za-z0-9_./-]+?\.(?:png|jpg|json|wav|fnt|dat|otf|ttf))"')
# 动态 format! 前缀展开口径（24 号 §3.2）：运行时按名拼接，缺图走代码矢量/通用回退 ⇒
# 同前缀全带，不逐枚甄别。
DYN_PREFIXES = ("assets/ui/icon_", "assets/ui/isle_", "assets/ui/island_home_", "assets/ui/role_")
MAP_JSON_NAME_RE = re.compile(r'"(isle_[a-z_]+|island_[a-z0-9_]+)(?:\.png)?"')


def collect_refset() -> dict[str, list[str]]:
    """引擎引用集：代码字面量 + format! 动态前缀全带 + 地图包岛 kind + registry audio。"""
    refs: set[str] = set()
    for crate in ("crates/ti-engine", "crates/ti-app"):
        for dirpath, _dirs, files in os.walk(os.path.join(REPO, crate)):
            for fn in files:
                if not fn.endswith(".rs"):
                    continue
                try:
                    text = open(os.path.join(dirpath, fn), encoding="utf-8", errors="ignore").read()
                except OSError:
                    continue
                for m in LIT_RE.finditer(text):
                    refs.add(m.group(1))
    # 动态前缀全带（磁盘有即带；见 24 号 §3.2）
    for dirpath, _dirs, files in os.walk(os.path.join(REPO, "assets/ui")):
        for fn in files:
            full = os.path.relpath(os.path.join(dirpath, fn), REPO).replace(os.sep, "/")
            if full.startswith(DYN_PREFIXES):
                refs.add(full)
    # 地图包 JSON 岛 kind（board 10 类 + 专属 island_home）
    import pathlib
    # 地图包/数据 JSON 岛 kind 只收「磁盘有文件」的 kind（token 名 isle_black 等 = 色板键，非资产）
    for fi in pathlib.Path(REPO, "assets/maps").rglob("*.json"):
        for m in MAP_JSON_NAME_RE.finditer(fi.read_text(errors="ignore")):
            p = "assets/ui/" + m.group(1) + ".png"
            if os.path.isfile(os.path.join(REPO, p)):
                refs.add(p)
    for fi in pathlib.Path(REPO, "assets/data").rglob("*.json"):
        for m in MAP_JSON_NAME_RE.finditer(fi.read_text(errors="ignore")):
            p = "assets/ui/" + m.group(1) + ".png"
            if os.path.isfile(os.path.join(REPO, p)):
                refs.add(p)
        # S39 数据化：`scenes.json`（数据）引用的 UI 资产是 `"assets/ui/<name>.png"` 全路径
        # 字面量（如 `index_plank_0.png`）——不在代码字面量、非动态前缀、非岛 kind，
        # 引用集若不扫数据 JSON ⇒ 数据驱动资产被引用集裁剪掉（wasm/APK 部署 404）。
        # 同 LIT_RE 口径扫数据 JSON 全路径字面量（24 号标准③：引用集变更先改工具）。
        for m in LIT_RE.finditer(fi.read_text(errors="ignore")):
            refs.add(m.group(1))
    # registry audio（`path` 字段；缺登记 = 漏音频）
    audio: list[str] = []
    registry = os.path.join(REPO, "assets/registry.json")
    if os.path.isfile(registry):
        reg = json.load(open(registry, encoding="utf-8"))
        for item in reg.get("audio", []):
            p = item.get("path") or item.get("file", "")
            if p:
                audio.append(p)
    ui = sorted(r for r in refs if r.startswith("assets/ui/"))
    fonts = sorted(r for r in refs if r.startswith("assets/fonts/"))
    data = sorted(r for r in refs if r.startswith("assets/registry.json"))
    legacy = sorted(r for r in refs if r.startswith("assets/") and not (r.startswith("assets/ui/") or r.startswith("assets/fonts/")))
    # 缺口登记（24 号 §3.1）：m04 manifest `central_asset=island_home_m04` 数据已声明但
    # 资产未生成；运行时按名读 → 缺图回退链（⇒ island_home_m01），包内不带此文件（is_file 跳过）。
    refs.add("assets/ui/island_home_m04.png")
    refs.add("assets/ui/icon.png")  # 引擎代码直引（非 icon_ 动态前缀），缺图回退代码矢量
    return {"ui": ui, "fonts": fonts, "audio": audio, "data": data, "legacy": legacy}


def unused_ui(refset: dict[str, list[str]]) -> list[tuple[str, int]]:
    """磁盘有、引用集无的 `assets/ui/` 文件（(路径, 字节) 降序；构建侧据此裁剪）。"""
    refd = set(refset["ui"])
    out: list[tuple[str, int]] = []
    ui_dir = os.path.join(REPO, "assets/ui")
    for dirpath, _dirs, files in os.walk(ui_dir):
        for fn in files:
            rel = os.path.relpath(os.path.join(dirpath, fn), REPO).replace(os.sep, "/")
            if rel not in refd:
                out.append((rel, os.path.getsize(os.path.join(REPO, rel))))
    out.sort(key=lambda t: (-t[1], t[0]))
    return out


def apk_breakdown(apk_path: str) -> dict:
    z = zipfile.ZipFile(apk_path)
    agg: dict[str, dict] = {}
    so_info: dict = {}
    total_c = total_u = 0
    for i in z.infolist():
        name, u, c = i.filename, i.file_size, i.compress_size
        if name.endswith("/"):
            continue
        total_c += c
        total_u += u
        if name.startswith("lib/"):
            key = "lib/" + name.split("/")[1] + "/.so"
            so_info["entry"] = name
            so_info["file_size"] = u
            so_info["compress_size"] = c
            so_info["method"] = "Stored" if i.compress_type == zipfile.ZIP_STORED else "Deflate"
        elif name.startswith("assets/ui/v6/"):
            key = "assets/ui/v6"
        elif name.startswith("assets/ui/vs_v2/"):
            key = "assets/ui/vs_v2"
        elif name.startswith("assets/ui/"):
            key = "assets/ui(其余)"
        elif name.startswith("assets/fonts"):
            key = "assets/fonts"
        elif name.startswith("assets/"):
            key = "assets(其余)"
        else:
            key = name.split("/")[0]
        a = agg.setdefault(key, {"compressed": 0, "uncompressed": 0, "files": 0})
        a["compressed"] += c
        a["uncompressed"] += u
        a["files"] += 1
    return {"apk": apk_path, "total_compressed": total_c, "total_uncompressed": total_u,
            "by_dir": agg, "lib": so_info}


def main() -> int:
    if "--refset" in sys.argv:
        print(json.dumps(collect_refset(), ensure_ascii=False))
        return 0
    if "--refset-unused" in sys.argv:
        print(json.dumps(unused_ui(collect_refset()), ensure_ascii=False))
        return 0
    if "--apk" in sys.argv:
        idx = sys.argv.index("--apk")
        path = sys.argv[idx + 1]
        bd = apk_breakdown(path)
        print(json.dumps(bd, ensure_ascii=False, indent=1))
        lib = bd["lib"]
        # 判据①：.so 必须 ZIP_DEFLATED（压缩率口径 = csize/filesize，Stored = 100% 即 FAIL）
        if not lib:
            print("FAIL: APK 无 lib/ 段（缺 .so）", file=sys.stderr)
            return 2
        ratio = lib["compress_size"] / lib["file_size"] if lib["file_size"] else 0.0
        # Deflate-9 对 .so 实测 ≈38%（原始 100%）；Stored = 100%（ratio=1.0）。
        # 阈值取 0.90：压缩后 ratio 必 < 0.5，Stored 必 = 1.0；0.90 是安全分界。
        if ratio >= 0.90 or lib.get("method", "Deflate") == "Stored":
            print(f"FAIL: .so 未压缩（ratio={ratio:.0%}；须 ZIP_DEFLATED）", file=sys.stderr)
            return 3
        # 判据②：APK 内 `assets/ui/` 全在引用集内（未引用图 0 进包）
        refset = collect_refset()
        refd = set(refset["ui"])
        z2 = zipfile.ZipFile(path)
        drift = [i.filename for i in z2.infolist()
                 if i.filename.startswith("assets/ui/") and i.filename not in refd]
        if drift:
            for d in sorted(drift)[:10]:
                print(f"FAIL: 未引用资产进包 {d}", file=sys.stderr)
            if len(drift) > 10:
                print(f"FAIL: … 另 {len(drift) - 10} 项", file=sys.stderr)
            return 4
        if "--json" not in sys.argv:
            print(f"OK: .so 压缩率 {ratio:.0%}（{lib['method']}）；APK 压缩后 "
                  f"{bd['total_compressed']/1048576:.1f} MB；未引用 UI 漂移 0")
        return 0
    print(__doc__.strip().splitlines()[0])
    return 1


if __name__ == "__main__":
    sys.exit(main())
