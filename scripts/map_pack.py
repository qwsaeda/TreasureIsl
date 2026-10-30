#!/usr/bin/env python3
"""map_pack.py — 地图包工具（S31 / `docs/process/21_map_package_spec.md`）。

**一张图 = 一个可分发的包**：`assets/maps/<id>/` = 已装态（`manifest.json`/`map.json`/
`layout.json`/`strings.json`/`assets/`）；`dist/maps/<id>.map-pack/` = 可分发件。
加图 / 换图 = 出一包 + 装上（运行期可切包），**不改 Rust、不手改多份 JSON**。

子命令：
  pack <id>            已装包 → 可分发件 `dist/maps/<id>.map-pack/`（+ `.zip`），写 sha256 清单
  verify <path>        校验包：schema/兼容/资产齐+哈希/不变量 I1~I9 -> 违规非 0
  install <path>       装包 → `assets/maps/<id>/` + 接线（默认包数据 / 资产登记 / 文案合并）
  uninstall <id>       卸包（保留构建期默认包数据引用）
  list                 列出已装包（id / version / data_version / 校验结果）
  add <id>             从 `_template` 包引导新包骨架（map.json 待填，verify 不过直至补齐）
  check                已装态 vs 构建期默认数据漂移对账（只读，漂移非 0；布局只比默认包）
  sync                 从已装包回注构建期默认数据（全部包 map.json → board.json；默认包 layout → scenes/manifest）

工具契约（`14_tooling_catalog.md` §1.2）：幂等 + `--check` 只读 + 违规非 0 退出。
默认包指针 = `assets/maps/.default`（文本 = 包 id）；构建期默认数据只镜像默认包。
"""

import argparse
import hashlib
import json
import os
import shutil
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAPS_DIR = ROOT / "assets" / "maps"
DIST_DIR = ROOT / "dist" / "maps"
POOL_DIR = ROOT / "assets" / "ui"
BOARD_JSON = ROOT / "assets" / "data" / "board.json"
SCENES_JSON = ROOT / "assets" / "data" / "scenes.json"
STRINGS_JSON = ROOT / "assets" / "data" / "strings.json"
MANIFEST_JSON = ROOT / "assets" / "data" / "ui_schema" / "scene_manifest_v3.json"
UI_MANIFEST = ROOT / "assets" / "ui_manifest.json"

SCHEMA = "map-pack/1"
DATA_VERSION = "board-v5"
SPACE_COUNT = 31
ISLE_KINDS = 10
PACK_ASSET_DIR = "assets"
POOL_PREFIX = "assets/ui/"
MAP_PREFIX = "assets/maps/"
DEFAULT_PTR = MAPS_DIR / ".default"
TEMPLATE_DIR = MAPS_DIR / "_template"

# 环形几何 8pt 校验豁免：center_x = 540（屏心）
RING_8PT_EXEMPT = {"center_x"}


# ----------------------------------------------------------------- 基础工具


def dump_canonical(path: Path, obj) -> None:
    """indent=1 逐字节规范形（与 `check_scenes_canonical` 同一序列化口径，防整文件 churn）。"""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, ensure_ascii=False, indent=1) + "\n", encoding="utf-8", newline="\n")


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_of(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(131072), b""):
            digest.update(chunk)
    return digest.hexdigest()


def installed_packs() -> list[Path]:
    if not MAPS_DIR.is_dir():
        return []
    return sorted(
        p for p in MAPS_DIR.iterdir() if p.is_dir() and p.name != "_template" and (p / "manifest.json").is_file()
    )


def default_pack() -> Path:
    """构建期默认包（指针 `assets/maps/.default`）；无指针 = 首个已装包。"""
    if DEFAULT_PTR.is_file():
        want = DEFAULT_PTR.read_text(encoding="utf-8").strip()
        hit = MAPS_DIR / want
        if want and hit.is_dir() and (hit / "manifest.json").is_file():
            return hit
    packs = installed_packs()
    if not packs:
        raise SystemExit("map_pack: 无已装包，默认包不可定位（先 install / bootstrap）")
    return packs[0]


def default_pack_id() -> str:
    return pack_id_of(default_pack())


def pack_id_of(pack_dir: Path) -> str:
    manifest = load_json(pack_dir / "manifest.json")
    return str(manifest.get("id", pack_dir.name))


# ----------------------------------------------------------------- 校验（I1~I9）


def verify_pack(pack_dir: Path) -> list[str]:
    """返回问题清单（空 = 通过）。判据 = 规范 §5 不变量 + §6 版本兼容。"""
    problems: list[str] = []
    manifest_path = pack_dir / "manifest.json"
    if not manifest_path.is_file():
        return [f"{pack_dir}: 缺 manifest.json（不是合法包）"]
    try:
        manifest = load_json(manifest_path)
    except json.JSONDecodeError as exc:
        return [f"{pack_dir}: manifest.json 解析失败 {exc}"]

    # §6 版本兼容
    if manifest.get("schema") != SCHEMA:
        problems.append(f"schema = {manifest.get('schema')!r}（期望 {SCHEMA!r}）-> 拒绝安装")
    if manifest.get("data_version") != DATA_VERSION:
        problems.append(
            f"data_version = {manifest.get('data_version')!r}（期望 {DATA_VERSION!r}）-> 拒绝加载"
        )
    pack_id = str(manifest.get("id", ""))
    if not pack_id:
        problems.append("manifest 缺 id")

    # 包内数据文件在位
    for name in ("map.json", "layout.json", "strings.json"):
        if not (pack_dir / name).is_file():
            problems.append(f"缺 {name}")

    # I1~I5：赛道结构
    map_data = manifest.get("map") or {}
    spaces = map_data.get("spaces") or []
    if len(spaces) != SPACE_COUNT:
        problems.append(f"I1 赛道格数 = {len(spaces)}（期望 {SPACE_COUNT}）")
    kinds = [s.get("kind") for s in spaces]
    start_n = kinds.count("start")
    terr_n = kinds.count("territory")
    boss_n = kinds.count("boss")
    feat_n = len(spaces) - start_n - terr_n - boss_n
    if start_n != 1:
        problems.append(f"I2 起点数 = {start_n}（期望 1）")
    if terr_n != 20:
        problems.append(f"I2 领地格数 = {terr_n}（期望 20）")
    if feat_n != 9:
        problems.append(f"I2 功能格数 = {feat_n}（期望 9）")
    if boss_n != 1:
        problems.append(f"I2 BOSS 格数 = {boss_n}（期望 1）")
    isle_count: dict[str, int] = {}
    isle_group: dict[str, set] = {}
    for space in spaces:
        isle = space.get("isle")
        if not isle:
            continue
        isle_count[isle] = isle_count.get(isle, 0) + 1
        isle_group.setdefault(isle, set()).add(space.get("group"))
    if len(isle_count) != ISLE_KINDS:
        problems.append(f"I3 岛型数 = {len(isle_count)}（期望 {ISLE_KINDS}）")
    for isle, count in sorted(isle_count.items()):
        if count != 2:
            problems.append(f"I3 岛型 `{isle}` = {count} 格（期望 2）")
        if len(isle_group.get(isle, set())) != 1:
            problems.append(f"I3 岛型 `{isle}` 两格不同区（group 不一致）")
    if spaces and kinds[0] != "start":
        problems.append("I5 首格不是 start（格序 = 前进方向）")
    if spaces and kinds[len(spaces) // 2] != "boss":
        problems.append(f"I5 环半周（第 {len(spaces) // 2} 格）不是 boss")

    # I9：每图 events[]/merchants[]（territory.md §6/§7；S33.2）
    events = map_data.get("events") or []
    if not events:
        problems.append("I9 events[] 为空（每图必须配特有随机事件）")
    merchants = map_data.get("merchants") or []
    merchant_ids = {str(m.get("id", "")) for m in merchants}
    if "weapon" not in merchant_ids:
        problems.append("I9 merchants 缺武器商人（id=weapon）")
    if "slave" not in merchant_ids:
        problems.append("I9 merchants 缺奴隶商人（id=slave）")
    # 领地格逐格：领地格必须有 building（初始等级 ≥1）；功能格无 isle
    for i, s in enumerate(spaces):
        if s.get("isle") and (not isinstance(s.get("building"), int) or s.get("building", 0) < 1):
            problems.append(f"I9 spaces[{i}] 领地格缺 building（初始等级 ≥1）")
        if s.get("isle") and not s.get("resource"):
            problems.append(f"I9 spaces[{i}] 领地格缺 resource（产出地形）")

    # I6：环几何 8pt
    ring = (manifest.get("layout") or {}).get("ring") or {}
    for key, value in ring.items():
        if key in RING_8PT_EXEMPT or not isinstance(value, int):
            continue
        if value % 8 != 0:
            problems.append(f"I6 ring.{key} = {value}（非 8 整除）")

    # I8：焦点 N8 往返（world_canvas ≡ bbox_pct × 1080×1920 ±1px）
    layout = manifest.get("layout") or {}
    canvas = layout.get("world_canvas")
    pct = (layout.get("focus_bbox_pct") or manifest.get("focus_bbox_pct") or {})
    if canvas and pct:
        for key, dim in (("x", 1080), ("y", 1920), ("w", 1080), ("h", 1920)):
            want = round(pct.get(key, 0) / 100 * dim)
            got = canvas.get(key)
            if got is None or abs(got - want) > 1:
                problems.append(f"I8 focus.{key} = {got}（≠ bbox_pct×{dim} = {want}，容差 1px）")
    elif canvas and not pct:
        problems.append("I8 缺 focus_bbox_pct（无法做 N8 往返）")

    # I7：资产齐全 + 哈希一致（pack 与 pool 两种来源都算；pool 资产还须在全局池登记）
    ui_registered = _ui_registered_files()
    for entry in manifest.get("assets") or []:
        rel = str(entry.get("file", ""))
        source = str(entry.get("source", "pack"))
        if source == "pack":
            target = pack_dir / rel
        elif source == "pool":
            target = ROOT / rel
        else:
            problems.append(f"assets[{rel}].source = {source!r}（期望 'pack' / 'pool'）")
            continue
        if not target.is_file():
            problems.append(f"I7 资产缺失 {rel}（source={source}）")
            continue
        want_bytes = entry.get("bytes")
        if isinstance(want_bytes, int) and target.stat().st_size != want_bytes:
            problems.append(f"I7 字节数不符 {rel}（{target.stat().st_size} ≠ {want_bytes}）")
        want_hash = str(entry.get("sha256", ""))
        if want_hash and sha256_of(target) != want_hash:
            problems.append(f"I7 哈希不符 {rel}（sha256 不一致）")
        if source == "pool" and rel.replace(os.sep, "/") not in ui_registered:
            problems.append(f"I7 池资产 {rel} 未在 ui_manifest.json 登记")

    # invariants 自述 vs 实测
    inv = manifest.get("invariants") or {}
    for key, actual in (("spaces", len(spaces)), ("start", start_n), ("territory", terr_n), ("boss", boss_n)):
        if key in inv and inv[key] != actual:
            problems.append(f"invariants.{key} = {inv[key]}（实测 {actual}）")
    return problems


# ----------------------------------------------------------------- 打包 / 校验 / 安装


def cmd_pack(map_id: str) -> int:
    """已装包 → 可分发件（含 sha256 清单；同输入同产物 = 幂等）。"""
    pack_dir = MAPS_DIR / map_id
    if not (pack_dir / "manifest.json").is_file():
        print(f"map_pack pack: 未找到已装包 {pack_dir}（先 install 或手工建包）", file=sys.stderr)
        return 1
    problems = verify_pack(pack_dir)
    if problems:
        print(f"map_pack pack: 包校验失败（{len(problems)} 项）-> 不出包", file=sys.stderr)
        for item in problems:
            print(f"  [FAIL] {item}", file=sys.stderr)
        return 1
    out_dir = DIST_DIR / f"{map_id}.map-pack"
    if out_dir.exists():
        shutil.rmtree(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    for name in ("manifest.json", "map.json", "layout.json", "strings.json"):
        shutil.copy2(pack_dir / name, out_dir / name)
    src_assets = pack_dir / PACK_ASSET_DIR
    if src_assets.is_dir():
        shutil.copytree(src_assets, out_dir / PACK_ASSET_DIR)
    # 池资产副本进分发件（可独立分发；manifest I7 已验哈希）
    manifest = load_json(pack_dir / "manifest.json")
    pool_copied = False
    for entry in manifest.get("assets") or []:
        if str(entry.get("source", "pack")) != "pool":
            continue
        src = ROOT / str(entry["file"])
        dst = out_dir / "pool" / Path(str(entry["file"])).name
        dst.parent.mkdir(parents=True, exist_ok=True)
        if src.is_file():
            shutil.copy2(src, dst)
            pool_copied = True
    if pool_copied:
        (out_dir / "pool" / "POOL_INDEX.json").write_text(
            json.dumps(
                [
                    {"file": e["file"], "sha256": e.get("sha256"), "bytes": e.get("bytes")}
                    for e in manifest.get("assets") or []
                    if str(e.get("source", "pack")) == "pool"
                ],
                ensure_ascii=False,
                indent=1,
            )
            + "\n",
            encoding="utf-8",
            newline="\n",
        )
    zip_path = DIST_DIR / f"{map_id}.map-pack.zip"
    if zip_path.exists():
        zip_path.unlink()
    with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED) as zf:
        for item in sorted(out_dir.rglob("*")):
            if item.is_file():
                zf.write(item, item.relative_to(out_dir))
    print(f"map_pack pack: {map_id} → {out_dir.relative_to(ROOT)} + {zip_path.relative_to(ROOT)}")
    return 0


def cmd_verify(path: Path) -> int:
    """只读校验包（违规非 0）。

    参数面（S32.3 平台制作指引使用面）：裸 id（如 `m01`）= 解析为已装包目录
    `assets/maps/<id>`；目录路径 / `.map-pack` / `.zip` = 原有行为不变。"""
    if not path.exists() and (MAPS_DIR / path.name / "manifest.json").is_file():
        path = MAPS_DIR / path.name
    if path.is_file() and path.suffix == ".zip":
        import tempfile

        with tempfile.TemporaryDirectory() as tmp:
            with zipfile.ZipFile(path) as zf:
                zf.extractall(tmp)
            problems = verify_pack(Path(tmp))
    else:
        problems = verify_pack(path)
    if problems:
        print(f"map_pack verify: {path} -> {len(problems)} 项不合规")
        for item in problems:
            print(f"  [FAIL] {item}")
        return 1
    print(f"map_pack verify: {path} -> 0 违规（schema/兼容/I1~I9 全过）")
    return 0


def cmd_install(path: Path) -> int:
    """装包 → assets/maps/<id>/ + 接线（默认包数据 / 资产登记 / 文案合并）。幂等 upsert。"""
    if path.is_file() and path.suffix == ".zip":
        import tempfile

        tmp = Path(tempfile.mkdtemp())
        try:
            with zipfile.ZipFile(path) as zf:
                zf.extractall(tmp)
            return _install_dir(tmp)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
    return _install_dir(path)


def _install_dir(src: Path) -> int:
    problems = verify_pack(src)
    if problems:
        print(f"map_pack install: 拒绝安装（{len(problems)} 项不合规）", file=sys.stderr)
        for item in problems:
            print(f"  [FAIL] {item}", file=sys.stderr)
        return 1
    manifest = load_json(src / "manifest.json")
    map_id = str(manifest["id"])
    dest = MAPS_DIR / map_id
    if dest.exists():
        shutil.rmtree(dest)
    dest.mkdir(parents=True, exist_ok=True)
    for name in ("manifest.json", "map.json", "layout.json", "strings.json"):
        shutil.copy2(src / name, dest / name)
    src_assets = src / PACK_ASSET_DIR
    if src_assets.is_dir():
        shutil.copytree(src_assets, dest / PACK_ASSET_DIR)
    # 接线：构建期默认数据（board.json）；池资产已在 verify_pack I7 校验（含 ui_manifest 登记）
    _wire_board(manifest)
    if not DEFAULT_PTR.is_file():
        DEFAULT_PTR.write_text(map_id + "\n", encoding="utf-8", newline="\n")
    write_pack_index()  # S31.2b：浏览器侧包清单（index.json）与已装态对齐（幂等）
    print(f"map_pack install: {map_id} → {dest.relative_to(ROOT)}（board.json 已接线 + index.json 更新）")
    return 0


def _wire_board(manifest: dict) -> None:
    """把包的 map 段写进构建期默认包 `board.json`（maps.<id>）。"""
    board = load_json(BOARD_JSON)
    board.setdefault("maps", {})[str(manifest["id"])] = json.loads(json.dumps(manifest["map"]))
    dump_canonical(BOARD_JSON, board)


def _ui_registered_files() -> set:
    """ui_manifest.json 已登记的资产 file 集合（池资产登记口径；缺失 = 空集）。"""
    if not UI_MANIFEST.is_file():
        return set()
    try:
        manifest = load_json(UI_MANIFEST)
    except json.JSONDecodeError:
        return set()
    return {str(a.get("file")) for a in manifest.get("assets", []) if a.get("file")}


def cmd_uninstall(map_id: str) -> int:
    dest = MAPS_DIR / map_id
    if not dest.is_dir():
        print(f"map_pack uninstall: {map_id} 未安装（幂等，0 退出）")
        return 0
    was_default = DEFAULT_PTR.is_file() and DEFAULT_PTR.read_text(encoding="utf-8").strip() == map_id
    shutil.rmtree(dest)
    if was_default:
        remaining = installed_packs()
        if remaining:
            DEFAULT_PTR.write_text(remaining[0].name + "\n", encoding="utf-8", newline="\n")
            print(f"map_pack uninstall: 默认包指针 → {remaining[0].name}")
    write_pack_index()  # S31.2b：index.json 与卸包后已装态对齐
    print(f"map_pack uninstall: 已卸 {map_id}（构建期默认数据引用保留，运行期该包缺失 → 回退默认包数据）")
    return 0


def cmd_list() -> int:
    if not installed_packs():
        print("map_pack list: 无已装包")
        return 0
    for pack_dir in installed_packs():
        manifest = load_json(pack_dir / "manifest.json")
        problems = verify_pack(pack_dir)
        state = "OK" if not problems else f"违规 {len(problems)} 项"
        print(
            f"  {manifest.get('id'):<8} v{manifest.get('version', '?')} "
            f"data={manifest.get('data_version', '?')}  {state}  {pack_dir.relative_to(ROOT)}"
        )
    return 0


def cmd_check() -> int:
    """只读对账：已装包 vs 构建期默认数据。

    - 全部已装包：`board.json maps.<id>` vs 包 `map.json`（赛道数据是构建期全量内嵌）
    - 仅默认包（`assets/maps/.default`）：`layout.json` vs `scenes.json main_map` / `scene_manifest_v3`
      （布局只镜像默认包；非默认包布局只在运行期切包时生效）
    漂移非 0。
    """
    problems: list[str] = []
    board = load_json(BOARD_JSON)
    packs = installed_packs()
    for pack_dir in packs:
        manifest = load_json(pack_dir / "manifest.json")
        map_id = str(manifest["id"])
        # 无 `map` 键的包（草稿态）= 不参与回注（cmd_sync 跳过）⇒ 对账亦跳过
        # （board.json 不应有 maps.<id> 段；若有 = 脏残留，报漂移）。
        if "map" not in manifest:
            if map_id in board.get("maps", {}):
                problems.append(f"board.json maps.{map_id} 残留（已装包 {pack_dir.name} 无 map 键，不应有回注段）")
            continue
        if board.get("maps", {}).get(map_id) != manifest.get("map"):
            problems.append(f"board.json maps.{map_id} 与已装包 {pack_dir.name}/map.json 漂移")
    try:
        default_id = default_pack_id()
        default_manifest = load_json(default_pack() / "manifest.json")
    except SystemExit:
        return 1
    if str(default_manifest.get("id")) != default_id:
        problems.append(f"manifest id ≠ 目录名（{default_manifest.get('id')} vs {default_id}）")
    _layout_drift_problems(default_id, default_manifest, problems)
    if problems:
        print(f"map_pack check: {len(problems)} 处漂移")
        for item in problems:
            print(f"  [FAIL] {item}")
        return 1
    print(f"map_pack check: 0 漂移（{len(packs)} 个已装包 vs board.json；默认包 {default_id} vs 布局数据）")
    return 0


def _layout_drift_problems(map_id: str, manifest: dict, problems: list[str]) -> None:
    """默认包 layout.json vs scenes.json main_map / scene_manifest_v3 focus。"""
    layout = manifest.get("layout") or {}
    scenes = load_json(SCENES_JSON)
    manifest_v3 = load_json(MANIFEST_JSON)
    main_map = scenes.get("main_map", {})
    ring = layout.get("ring") or {}
    for key, value in ring.items():
        if main_map.get("ring", {}).get(key) != value:
            problems.append(f"scenes.json main_map.ring.{key} 与默认包 {map_id} 漂移")
    canvas = layout.get("world_canvas")
    if canvas and main_map.get("world_canvas") != canvas:
        problems.append(f"scenes.json main_map.world_canvas 与默认包 {map_id} 漂移")
    pct = layout.get("focus_bbox_pct")
    if pct:
        cur = manifest_v3.get("scenes", {}).get("main_map", {}).get("narrative", {}).get("focus", {})
        if cur.get("bbox_pct") != pct:
            problems.append(f"manifest main_map.focus.bbox_pct 与默认包 {map_id} 漂移")


def cmd_sync() -> int:
    """回注构建期默认数据：全部已装包 `map.json` → `board.json maps.<id>`；
    默认包（`.default`）`layout.json` → `scenes.json main_map` / `scene_manifest_v3 focus` / `strings.json`。幂等。"""
    if not installed_packs():
        print("map_pack sync: 无已装包 -> 无可回注内容", file=sys.stderr)
        return 1
    board = load_json(BOARD_JSON)
    # 回注 = 全量重算：清 `maps` 段再逐包回注（旧回注残留 / 已卸包脏段 随 sync 清零；
    # 幂等 = 已装包不变时 sync 前后 board.json 逐字节一致）。
    board["maps"] = {}
    for pack_dir in installed_packs():
        manifest = load_json(pack_dir / "manifest.json")
        # 无 `map` 键的包（如 m04 在途草稿态）= 跳过本包回注（口径 = `verify_pack`/
        # `cmd_check` 的 `.get` 容错；旧 `manifest["map"]` 下标对草稿包直接 KeyError，
        # 15 号第 24 行待固化项 = S32.12 A 侧 P2-1）。
        if "map" not in manifest:
            continue
        board["maps"][str(manifest.get("id", pack_dir.name))] = json.loads(json.dumps(manifest["map"]))
    default_manifest = load_json(default_pack() / "manifest.json")
    default_id = str(default_manifest["id"])
    layout = default_manifest.get("layout") or {}
    scenes = load_json(SCENES_JSON)
    manifest_v3 = load_json(MANIFEST_JSON)
    strings = load_json(STRINGS_JSON)
    main_map = scenes.setdefault("main_map", {})
    if layout.get("ring"):
        main_map["ring"] = json.loads(json.dumps(layout["ring"]))
    if layout.get("world_canvas"):
        main_map["world_canvas"] = json.loads(json.dumps(layout["world_canvas"]))
        main_map.setdefault("focus_anchor", {})["bbox"] = json.loads(json.dumps(layout["world_canvas"]))
    if layout.get("island_home"):
        main_map["island_home"] = json.loads(json.dumps(layout["island_home"]))
    if layout.get("deck"):
        for key, rect in layout["deck"].items():
            main_map[f"deck_{key}"] = json.loads(json.dumps(rect))
    if layout.get("hint_y"):
        main_map.setdefault("hint", {})["y"] = layout["hint_y"]
    if layout.get("focus_bbox_pct"):
        manifest_v3.setdefault("scenes", {}).setdefault("main_map", {}).setdefault("narrative", {}).setdefault(
            "focus", {}
        )["bbox_pct"] = json.loads(json.dumps(layout["focus_bbox_pct"]))
    for key, value in (default_manifest.get("strings") or {}).items():
        strings[key] = value
    dump_canonical(BOARD_JSON, board)
    dump_canonical(SCENES_JSON, scenes)
    dump_canonical(MANIFEST_JSON, manifest_v3)
    dump_canonical(STRINGS_JSON, strings)
    print(
        f"map_pack sync: {len(installed_packs())} 个已装包 map.json → board.json；"
        f"默认包 {default_id} layout → scenes.json / manifest / strings"
    )
    return 0


def cmd_add(map_id: str) -> int:
    """从 `_template` 包引导新包骨架（共享布局 + 赛道骨架；map.json 待填 -> verify 不过直至补齐）。"""
    if not TEMPLATE_DIR.is_dir():
        print(f"map_pack add: 模板包 {TEMPLATE_DIR.relative_to(ROOT)} 不存在", file=sys.stderr)
        return 1
    dest = MAPS_DIR / map_id
    if dest.exists():
        print(f"map_pack add: {map_id} 已安装（先 uninstall 再 add）", file=sys.stderr)
        return 1
    shutil.copytree(TEMPLATE_DIR, dest)
    manifest_path = dest / "manifest.json"
    manifest = load_json(manifest_path)
    manifest["id"] = map_id
    manifest["version"] = "0.1.0-draft"
    dump_canonical(manifest_path, manifest)
    strings = load_json(dest / "strings.json")
    strings[f"glmap{map_id}.Map"] = strings.get("glmap_template.Map", f"{map_id}")
    dump_canonical(dest / "strings.json", strings)
    print(f"map_pack add: {map_id} 骨架已建（填 map.json 31 格 + 出专属资产后 verify / install / pack）")
    return 0


def write_pack_index() -> Path:
    """生成 `assets/maps/index.json`（`map-pack-index/1`，`build_wasm.ps1` 4e 段部署进
    `web/assets/maps/`，浏览器侧运行期枚举已装包；缺 = 内嵌默认包 §6 回退）。

    形状 = `{ "schema", "default", "ids" }`：
    - `default` = `.default` 指针内容（缺 = `null`；指针目标不合规时壳层回退 `ids[0]`）；
    - `ids` = 全部已装包 id（字典序，`manifest.json` 在位的目录）。
    幂等（同输入同产物）；无已装包 = 写空清单（壳层 §6 回退，不报错）。
    """
    ids = sorted(pack_dir.name for pack_dir in installed_packs())
    default_id = None
    if DEFAULT_PTR.is_file():
        candidate = DEFAULT_PTR.read_text(encoding="utf-8").strip()
        if candidate in ids:
            default_id = candidate
    index = MAPS_DIR / "index.json"
    dump_canonical(
        index,
        {"schema": "map-pack-index/1", "default": default_id, "ids": ids},
    )
    return index


def cmd_index() -> int:
    """生成 `assets/maps/index.json`（浏览器侧包清单；幂等，与 install/uninstall 后状态对齐）。"""
    index = write_pack_index()
    ids = installed_packs()
    print(
        f"map_pack index: {index.relative_to(ROOT)}（{len(ids)} 个已装包；"
        f"default = {DEFAULT_PTR.read_text(encoding='utf-8').strip() if DEFAULT_PTR.is_file() else '无'}）"
    )
    return 0


# ----------------------------------------------------------------- CLI


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="地图包工具（S31 / 21_map_package_spec.md）")
    sub = parser.add_subparsers(dest="cmd", required=True)
    p_pack = sub.add_parser("pack", help="已装包 → 可分发件")
    p_pack.add_argument("id")
    p_verify = sub.add_parser("verify", help="校验包（只读）")
    p_verify.add_argument("path")
    p_install = sub.add_parser("install", help="装包 + 接线")
    p_install.add_argument("path")
    p_uninstall = sub.add_parser("uninstall", help="卸包")
    p_uninstall.add_argument("id")
    sub.add_parser("list", help="列出已装包")
    p_add = sub.add_parser("add", help="从 _template 包引导新包骨架")
    p_add.add_argument("id")
    sub.add_parser("check", help="已装态 vs 默认数据漂移对账（只读）")
    sub.add_parser("sync", help="从已装包回注构建期默认数据")
    sub.add_parser("index", help="生成 assets/maps/index.json（浏览器侧包清单，幂等）")
    args = parser.parse_args(argv[1:])

    if args.cmd == "pack":
        return cmd_pack(args.id)
    if args.cmd == "verify":
        return cmd_verify(Path(args.path))
    if args.cmd == "install":
        return cmd_install(Path(args.path))
    if args.cmd == "uninstall":
        return cmd_uninstall(args.id)
    if args.cmd == "list":
        return cmd_list()
    if args.cmd == "add":
        return cmd_add(args.id)
    if args.cmd == "check":
        return cmd_check()
    if args.cmd == "sync":
        return cmd_sync()
    if args.cmd == "index":
        return cmd_index()
    parser.print_help()
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
