#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""把 `.rs` 里硬编码的中文字面量迁移到 `assets/data/strings.json` 并改写调用点。

用法：
    python scripts/migrate_strings.py --report                    # 只列出待迁移（不改动）
    python scripts/migrate_strings.py --apply                     # 执行迁移（改 strings.json + .rs）
    python scripts/migrate_strings.py --apply --file PATH         # 只处理指定文件（可重复）

做法（幂等 + 可重复跑）：
1. 扫 `crates/**/src/**`（排除 oracle/ 与 ti-tools/，见 check_hardcode.py 的 EXEMPT），
   找出**含 CJK 且命中 KEY_HINTS** 的字符串字面量；
2. 写入 `strings.json`（保留既有 key，不覆盖）；
3. 把 `.rs` 中的 `"<字面量>"` 替换为 `sget("<key>")`（独立参数位置；`format!` 首段前缀
   单独处理成 `format!("{}{n}", sget("key"), n)` 形式——本脚本不做那类重写，只报）。

范围控制：只迁移「整串就是字面量」。格式串（`format!("金{n}")`）会**报告**出来
但不自动改（需人工改成 `format!("{}{n}", sget("scene.main_map.hud.gold"), n)`），
避免脚本误改格式化占位符。

退出码：0 成功；非 0 失败。
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

CJK = re.compile(r"[\u4e00-\u9fff]")
STR_LIT = re.compile(r'"((?:[^"\\]|\\.)*)"')

SKIP_DIR_PARTS = {"tests", "target"}
SKIP_PATH_SUBSTR = ["oracle/", "ti-tools/"]

# 语义 key 提示表：字面量 → key（人工给一次，脚本批量用）。
# 新增文案时在此登记一条，脚本即按 key 迁移；不登记的字面量会被跳过并报告。
KEY_HINTS: dict[str, str] = {
    # 标题 / 菜单
    "金银岛": "scene.index.title",
    "主菜单": "scene.index.title",
    "选择角色": "scene.select_role.title",
    "选择地图": "scene.select_map.title",
    "主地图": "scene.main_map.title",
    "存读档": "scene.saveload.title",
    "战斗": "scene.vs.title",
    "[未解锁]": "scene.select_map.locked",
    "新游戏": "menu.new_game",
    "继续游戏": "menu.continue",
    "读取存档": "menu.load",
    "选项设置": "menu.options",
    "帮助说明": "menu.help",
    "离开游戏": "menu.quit",
    # 战斗
    "胜利!": "scene.vs.victory",
    "失败": "scene.vs.defeat",
    "无战斗": "scene.vs.empty",
    "按攻击键": "scene.vs.awaiting",
    # 场景提示 / HUD 前缀（玩家可见，S8 清 37 处硬编码）
    "38/40 切换  13 确认": "scene.select_role.hint",
    "38/40 切换  13 确认": "scene.select_map.hint",
    "攻击  返回": "scene.vs.hint",
    "方向键移动 据点触发事件": "scene.main_map.hint",
    "38/40 翻页  27 关闭": "scene.message.hint",
    "玩家 HP": "scene.vs.player_hp",
    "BOSS HP": "scene.vs.boss_hp",
    "回合": "scene.vs.rounds",
    "金": "scene.main_map.hud.gold",
    "上下选槽 13 读/写": "scene.saveload.slot_rwx",
    "持有": "scene.main_map.hud.owned",
    # 消息界面标签
    "据点": "msg.msgl",
    "面板": "msg.msgm",
    "帮助": "msg.help",
    "信息": "msg.info",
    "消息": "msg.msgs",
    # 商品
    "香料": "goods.1",
    "丝绸": "goods.2",
    "宝石": "goods.3",
    "黄金": "goods.4",
    "珍珠": "goods.5",
    # 建筑
    "城堡": "build.castle",
    "商行": "build.investment",
    "银行": "build.bank",
    "堡垒": "build.fort",
    # 卡牌
    "伏击": "cards.1",
    "补给": "cards.2",
    "宝藏": "cards.3",
    "风浪": "cards.4",
    "未知事件": "cards.5",
    # 主地图 HUD / 存读档
    "金": "scene.main_map.hud.gold",
    "持有": "scene.main_map.hud.owned",
    "槽": "scene.saveload.slot",
    "有": "scene.saveload.has",
    "空": "scene.saveload.empty",
}


def collect_literals(paths: list[Path]) -> tuple[list[tuple[str, int, str]], list[tuple[str, int, str]]]:
    """返回 (命中 KEY_HINTS 的, 未命中但含 CJK 的)。"""
    found: list[tuple[str, int, str]] = []
    unhandled: list[tuple[str, int, str]] = []
    for p in paths:
        rel = p.as_posix().replace("\\", "/")
        if any(s in rel for s in SKIP_PATH_SUBSTR):
            continue
        try:
            lines = p.read_text(encoding="utf-8").splitlines()
        except UnicodeDecodeError:
            continue
        for i, line in enumerate(lines, start=1):
            code = line.split("//")[0]
            for m in STR_LIT.finditer(code):
                lit = m.group(1)
                if not (lit and CJK.search(lit)):
                    continue
                if lit in KEY_HINTS:
                    found.append((rel, i, lit))
                else:
                    unhandled.append((rel, i, lit))
    return found, unhandled


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--root", type=Path, default=Path("."))
    ap.add_argument("--file", action="append", default=None, help="只处理指定文件（可重复）")
    ap.add_argument("--report", action="store_true", help="只报告不改动")
    ap.add_argument("--apply", action="store_true", help="执行迁移")
    args = ap.parse_args()

    root = args.root
    if args.file:
        paths = [root / f for f in args.file]
    else:
        paths = [
            p for p in sorted((root / "crates").rglob("*.rs"))
            if not any(x in p.parts for x in SKIP_DIR_PARTS)
        ]

    found, unhandled = collect_literals(paths)
    if not found:
        print("migrate_strings: 没有命中 KEY_HINTS 的字面量，无需迁移")
    else:
        by_key: dict[str, set[str]] = {}
        for _rel, _ln, lit in found:
            by_key.setdefault(KEY_HINTS[lit], set()).add(lit)
        print("migrate_strings: 待迁移 %d 处字面量 / %d 个 key" % (len(found), len(by_key)))
        for key, lits in sorted(by_key.items()):
            rels = sorted({rel for rel, _l, lit in found if KEY_HINTS.get(lit) == key})
            print("  %-32s <- %s   [%d file(s)]" % (key, " / ".join(sorted(lits))[:40], len(rels)))

    if unhandled:
        print("\nmigrate_strings: 未命中 KEY_HINTS 的 CJK 字面量（需补 KEY_HINTS 或人工处理）：")
        for rel, ln, lit in sorted(set(unhandled)):
            print("  %s:%d  `%s`" % (rel, ln, lit))

    if args.report or not args.apply:
        print("\n(--report 模式，未改动任何文件)")
        return 0

    # 1) 写 strings.json（保留既有）
    sp = root / "assets/data/strings.json"
    with sp.open(encoding="utf-8") as fh:
        strings = json.load(fh)
    added = 0
    for key, lits in by_key.items():
        if key not in strings:
            strings[key] = sorted(lits)[0]
            added += 1
    sp.write_text(
        json.dumps(strings, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    print("\n  strings.json: 新增 %d 个 key（共 %d）" % (added, len(strings)))

    # 2) 改写 .rs：字面量 → sget("key")
    re_written = 0
    for rel_path in {rel for rel, _l, _lit in found}:
        fp = root / rel_path
        text = fp.read_text(encoding="utf-8")
        orig = text
        # 先处理「带 import 需求」：确保 `use crate::i18n::sget;`
        for _rel, _ln, lit in found:
            if _rel != rel_path:
                continue
            key = KEY_HINTS[lit]
            text = text.replace('"%s"' % lit, 'sget("%s")' % key)
        if "use crate::i18n::sget;" not in text and "use ti_app::i18n::sget;" not in text:
            # 找到第一个 use 块末尾或文件头部锚点插入
            if text.startswith("use "):
                text = "use crate::i18n::sget;\n" + text
            elif "use crate::" in text:
                text = text.replace("use crate::", "use crate::i18n::sget;\nuse crate::", 1)
        if text != orig:
            fp.write_text(text, encoding="utf-8", newline="\n")
            re_written += 1
            print("  改写 %s" % rel_path)
    print("migrate_strings: 改写 %d 个文件" % re_written)
    print("下一步：`cargo build` 验证 + 若缺 use 则补 `use crate::i18n::sget;`（EXP-051 编码坑）。")
    return 0


if __name__ == "__main__":
    sys.exit(main())