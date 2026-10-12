# -*- coding: utf-8 -*-
"""S7.1 资产分类脚本：assets_raw/*/manifest.csv → ref/asset_classification.json。

按 docs/impl/14_asset_enhancement.md §2 的 5 类策略给全部 305 条容器条目打标签：
  A_ui        UI 控件/图标/按钮/对话框背景 → 矢量化
  B_portrait  角色立绘/头像/开场 → AI 超分（+可选 SD 重绘）
  C_pixel     兵种/船只/旗帜/地标 → 最近邻 2x
  D_terrain   地形瓦片 → Lanczos 2x + 摄像机 2x
  E_animation 多帧动画（ui.pak num=5 的 4 精灵）→ 时序稳定
  Skip        数据/音频/字体/OLE/未命名桶

E 类判定复用 mapinfo.ini 的 num 字段（与 sprite_atlas 解析同逻辑）：
num>1 的精灵优先归 E，文件名模式不再参与。

用法：python scripts/classify_assets.py
无第三方依赖（标准库 csv/json/re）。
"""
import csv
import json
import re
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
ASSETS_RAW = REPO / "assets_raw"
OUT = REPO / "ref" / "asset_classification.json"

# glmap_m*.pak 地形瓦片 / cards+goods 容器 → Skip（卡片数据/商品表非图像资产）
SKIP_CONTAINERS = {"cards", "goods"}
# 未命名桶（F-209：glpics 3 条名称哈希型，S6.5 起已按内容推断业务名归 C_pixel）
SKIP_NAMES = {
    "Thumbs.db",  # OLE 复合文档（F-209），非游戏图像
}

# A 类：UI 控件/按钮/翻页/对话框背景（ui.pak 的 button_* 由 glpics 侧命名，
# 见 §1.1——ui.pak 内 NN.pcx 编号图按内容归 B/C，button 实际在 glpics.pak）
A_PATTERNS = [
    re.compile(r"^button_"),
    re.compile(r"^switchpage\.pcx$"),
]
# B 类：角色立绘/头像/开场立绘
B_PATTERNS = [
    re.compile(r"^face_"),
    re.compile(r"^Player_"),
    re.compile(r"^start_"),
    re.compile(r"^Target_head\.pcx$"),
    re.compile(r"^Target_knife\.pcx$"),
    re.compile(r"^Logo\.pcx$"),
    re.compile(r"^text_gam"),  # text_gameover / text_gamerwin 结算大图
]
# C 类：兵种/船只/旗帜/地标（glpics 余下像素精灵）
C_PATTERNS = [
    re.compile(r"^troop"),
    re.compile(r"^ship"),
    re.compile(r"^flag_"),
    re.compile(r"^(Landmark|LandMark)"),
    re.compile(r"^helm_"),
    re.compile(r"^role_"),
    re.compile(r"^target_"),
    re.compile(r"^(dialogue|Dialogue)"),  # 对话框位图（F-204 大小写不敏感）
    re.compile(r"^number\.pcx$"),
    re.compile(r"^landinfo\.pcx$"),
    re.compile(r"^demo\.pcx$"),
    re.compile(r"^bg\.pcx$"),
    re.compile(r"^(boat_scene|stripe_bar|icon_stack)\.pcx$"),  # S6.5 glpics 孤儿 key 内容推断名
]


def parse_mapinfo_num(ini_path: Path) -> dict[str, int]:
    """解析 UTF-16LE mapinfo.ini → {filename: num}。"""
    data = ini_path.read_bytes()
    text = data.decode("utf-16")
    out: dict[str, int] = {}
    section: str | None = None
    pending: str | None = None
    pending_num = 1
    for raw in text.replace("\r\n", "\n").split("\n"):
        line = raw.strip().lstrip("\ufeff")
        if not line:
            continue
        if line.startswith("[") and line.endswith("]"):
            if section is not None and pending is not None:
                out[section] = pending_num
            section = line[1:-1]
            pending, pending_num = None, 1
            continue
        if line.startswith("filename="):
            pending = line[9:].strip()
        elif line.startswith("num="):
            pending_num = max(1, int(line[4:].strip()))
    if section is not None and pending is not None:
        out[section] = pending_num
    return out


def classify(container: str, name: str, anim_names: set[str]) -> str:
    if name in SKIP_NAMES:
        return "Skip"
    if name in anim_names:
        return "E_animation"
    if container == "ui":
        # ui.pak 的 NN.pcx：编号图 = 场景背景/控件贴图（内容判定后归 B/C）
        return "C_pixel"
    if container in ("glmap_m01", "glmap_m02"):
        return "D_terrain"
    if container in SKIP_CONTAINERS:
        # cards.lpk 的 card1/2.pcx 是卡牌面图（250×120 PCX）→ C 类批量超分；
        # 其余 .ini/.mdu 数据文件 → Skip
        if name.endswith(".pcx"):
            return "C_pixel"
        return "Skip"
    if container == "glpics":
        for p in A_PATTERNS:
            if p.match(name):
                return "A_ui"
        for p in B_PATTERNS:
            if p.match(name):
                return "B_portrait"
        for p in C_PATTERNS:
            if p.match(name):
                return "C_pixel"
        return "UNKNOWN"
    if container == "music":
        return "Skip"
    return "UNKNOWN"


def main() -> None:
    result: dict = {"entries": {}, "stats": {}}
    unknown: list[str] = []
    anim_names: set[str] = set()

    # 先解析 ui.pak 的 num 字段（E 类）：mapinfo.ini 段名 = 去 .pcx 后缀的文件名
    ui_ini = ASSETS_RAW / "ui" / "mapinfo.ini"
    if ui_ini.is_file():
        for fname, num in parse_mapinfo_num(ui_ini).items():
            if num > 1:
                anim_names.add(f"{fname}.pcx")

    total = 0
    for d in sorted(ASSETS_RAW.iterdir()):
        if not d.is_dir():
            continue
        container = d.name
        manifest = d / "manifest.csv"
        if not manifest.is_file():
            continue
        with manifest.open() as f:
            rows = list(csv.DictReader(f))
        for row in rows:
            name = row["display_name"].strip()
            key = row["key"].strip()
            total += 1
            rel = f"{container}/{key}"
            cls = classify(container, name, anim_names)
            if cls == "UNKNOWN":
                unknown.append(f"{container}/{name}")
            result["entries"][rel] = {
                "class": cls,
                "name": name,
                "container": container,
                "source": row.get("source", ""),
            }

    # glmap_m*.pak 的 mapinfo.ini 条目本身是容器元数据 → Skip
    for k, v in result["entries"].items():
        if v["name"] == "mapinfo.ini":
            v["class"] = "Skip"

    counts: dict[str, int] = {}
    for v in result["entries"].values():
        counts[v["class"]] = counts.get(v["class"], 0) + 1
    result["stats"] = {
        "total": total,
        "by_class": dict(sorted(counts.items())),
        "unknown": unknown,
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True))
    print(f"[classify_assets] {total} 条 → {OUT}")
    print(f"[classify_assets] 分类统计: {result['stats']['by_class']}")
    print(f"[classify_assets] E 类动画: {sorted(anim_names)}")
    if unknown:
        print(f"[classify_assets] UNKNOWN 桶 {len(unknown)} 条: {unknown}")
    else:
        print("[classify_assets] UNKNOWN 桶 = 0（分类覆盖 100%）")


if __name__ == "__main__":
    main()
