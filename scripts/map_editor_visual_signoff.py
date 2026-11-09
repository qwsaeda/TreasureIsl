#!/usr/bin/env python3
"""map_editor_visual_signoff.py — 地图编辑器前端视觉签收（Playwright 无头 Chromium 出图）。

用途：对 `docs/map_editor_guide.md` 设计稿的实现（`crates/ti-console/web/index.html` 地图页
编辑器 + 10 步向导面板）做最终视觉签收——无头浏览器打开平台「地图」页，触发编辑器 + 内容
面板 + 向导面板出图，落 `evidence/ui_concept/` 供用户过目（A 纪律「确认后才落码」的收口面）。

工具契约（14 号 §1.2）：
- `--check` 只读模式（产物在位 + 非空，缺则非 0 退出；不重新截图）；
- 幂等（同输入同产物：每次重截覆盖同路径，文件名稳定 = map_editor_signoff_*）。

前置：平台在 `127.0.0.1:8787` 在跑（`cargo run -p ti-console`）。缺则非 0 退出。

用法：
  python3 scripts/map_editor_visual_signoff.py            # 截图 4 张 → evidence/ui_concept/
  python3 scripts/map_editor_visual_signoff.py --check    # 只读校验产物在位
"""

import argparse
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT_DIR = ROOT / "evidence" / "ui_concept"
BASE_URL = "http://127.0.0.1:8787"

# 4 张签收截图（稳定文件名，幂等覆盖）
SHOTS = [
    "map_editor_signoff_full.png",        # 整页地图（编辑器 + 向导 + 指引）
    "map_editor_signoff_canvas.png",      # 环形画布（节点两态 + 岛色带）
    "map_editor_signoff_content.png",     # I9/S34.5 内容面板（events/merchants/vs_theme）
    "map_editor_signoff_wizard.png",      # 10 步向导面板
]


def check() -> int:
    problems = []
    for name in SHOTS:
        p = OUT_DIR / name
        if not p.is_file() or p.stat().st_size < 2048:
            problems.append(f"{name}（{'缺' if not p.is_file() else '过小 ' + str(p.stat().st_size) + 'B'}）")
    if problems:
        print(f"map_editor_visual_signoff check: {len(problems)} 张缺/过小")
        for item in problems:
            print(f"  [FAIL] {item}")
        return 1
    print(f"map_editor_visual_signoff check: 0 缺（{len(SHOTS)} 张视觉签收图在位非空）")
    return 0


def server_up() -> bool:
    try:
        with urllib.request.urlopen(BASE_URL + "/", timeout=5) as resp:
            return resp.status == 200
    except (urllib.error.URLError, ConnectionError, TimeoutError):
        return False


def capture() -> int:
    if not server_up():
        print(f"map_editor_visual_signoff: 平台 {BASE_URL} 未起（先 cargo run -p ti-console），拒绝截图", file=sys.stderr)
        return 1
    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        print("map_editor_visual_signoff: 缺 playwright（pip install playwright + install chromium），拒绝截图", file=sys.stderr)
        return 1

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    with sync_playwright() as pw:
        browser = pw.chromium.launch(headless=True)
        page = browser.new_page(viewport={"width": 1280, "height": 900})
        page.goto(BASE_URL + "/", wait_until="networkidle", timeout=30000)

        # 切到「地图」页
        page.click('button[data-page="map"]')
        page.wait_for_timeout(600)

        # 展开编辑器 + 内容面板 + 向导面板
        page.click("#map-edit-btn")
        page.wait_for_timeout(400)
        # 展开内容配置 <details>
        content_detail = page.query_selector("#map-editor details")
        if content_detail:
            content_detail.evaluate("el => el.open = true")
        # 渲染 10 步向导
        page.wait_for_timeout(400)

        # ① 整页地图（编辑器 + 向导 + 指引）
        page.screenshot(path=str(OUT_DIR / SHOTS[0]), full_page=False)

        # ② 环形画布
        canvas = page.query_selector("#ed-canvas")
        if canvas:
            canvas.screenshot(path=str(OUT_DIR / SHOTS[1]))

        # ③ 内容面板（I9/S34.5）
        content_box = page.query_selector("#ed-content-panels")
        if content_box:
            content_box.screenshot(path=str(OUT_DIR / SHOTS[2]))

        # ④ 向导面板
        wizard = page.query_selector("#ed-wizard")
        if wizard:
            wizard.screenshot(path=str(OUT_DIR / SHOTS[3]))

        browser.close()

    # 校验产物
    problems = []
    for name in SHOTS:
        p = OUT_DIR / name
        if not p.is_file() or p.stat().st_size < 2048:
            problems.append(name)
    if problems:
        print(f"map_editor_visual_signoff: {len(problems)} 张截图失败/过小：{problems}", file=sys.stderr)
        return 1
    print(f"map_editor_visual_signoff: 0 缺（{len(SHOTS)} 张 → {OUT_DIR.relative_to(ROOT)}）")
    for name in SHOTS:
        print(f"  {name}（{ (OUT_DIR/name).stat().st_size } B）")
    return 0


def main(argv) -> int:
    ap = argparse.ArgumentParser(description="地图编辑器前端视觉签收（Playwright 无头出图）")
    ap.add_argument("--check", action="store_true", help="只读：产物在位 + 非空，缺则非 0（不重新截图）")
    args = ap.parse_args(argv)
    if args.check:
        return check()
    return capture()


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
