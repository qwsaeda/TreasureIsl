#!/usr/bin/env python3
"""wasm_browser_e2e.py — Bevy wasm 浏览器端 e2e（Playwright 无头 Chromium）。

**行业口径**（wasm 交付物 4 层验证，与 `gate.ps1` native 层 `e2e_playthrough`
互补）：
  L1 实例化 = 无 missing-import（`env-stub.js` 覆盖 wasm 全部 env 导入）
     + start section 跑通（Bevy 主循环启动）；
  L2 渲染 = Bevy 资产 fetch 全部 2xx（字体/贴图/地图包，`.meta` 404 降级不算
     失败——`serve.rs` 缺失 = 404 语义）+ 画面非全黑（截图像素采样，PIL）；
  L3 交互 = 键盘（Enter 确认 / Space 摇骰）+ 鼠标点击不抛异常；
  L4 场景切换 = 键盘走到 select_map 后鼠标点击 m01 卡片 ⇒ 切到 main_map
     （增量判据：点击前亮像素比 < 40%（select_map ≈ 17.7%）且点击后 > 40%
     （main_map ≈ 78%）；慢点 down/up 分帧，防同帧 press+release 丢 Click）。

用法：
  python3 scripts/wasm_browser_e2e.py            # 全 4 层（断言 L1+L2+L3+L4）
  python3 scripts/wasm_browser_e2e.py --check    # 只读：不截图、打印判据（0 新产物）
  python3 scripts/wasm_browser_e2e.py --url=http://host:port/

前提：
  ① `serve` 静态服务在跑（`cargo run -p ti-tools --bin serve -- --dir web --port 8080`）；
  ② 本机有 Playwright Chromium（浏览器缺失时自动探测 `~/.cache/ms-playwright/chromium-*/`
     已装版，`TI_CHROMIUM_PATH` 可显式覆盖）；截图采样用 `Pillow`。
退出码 0 = 全绿；非 0 = 失败（判据打印到 stdout）。
工具契约（`14_tooling_catalog.md` §1.2）：幂等 + 只读模式 + 非 0 退出。
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SHOT_OUT = REPO / "evidence" / "wasm_browser_e2e.png"


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--url", default="http://127.0.0.1:8080/", help="已部署 web/ 的静态服务地址")
    p.add_argument("--check", action="store_true", help="只读模式：打印判据，不写截图")
    p.add_argument("--timeout", type=int, default=90, help="等待 start/渲染的秒数上限")
    p.add_argument("--out", default=str(SHOT_OUT), help="渲染截图输出路径")
    return p.parse_args()


def main() -> int:
    args = parse_args()
    try:
        from playwright.sync_api import sync_playwright
    except ModuleNotFoundError:
        print("[wasm-browser-e2e] FAIL  本机无 playwright（pip install playwright && playwright install chromium）")
        return 1

    checks: dict[str, object] = {}
    console_errors: list[str] = []
    asset_status: dict[str, int] = {}
    page_errors: list[str] = []

    with sync_playwright() as pw:
        import os
        # 本机 chromium 装在 /root/.cache/ms-playwright/chromium-1228（node-playwright 探针同路径）；
        # 缺省 = playwright 自带 headless shell（未装则 launch 失败）。
        exe = os.environ.get("TI_CHROMIUM_PATH")
        launch_kwargs = dict(headless=True, args=["--use-gl=swiftshader", "--enable-unsafe-swiftshader"])
        if exe:
            launch_kwargs["executable_path"] = exe
        else:
            # 自动探测已装 chromium（ms-playwright 缓存目录最新版）
            import glob
            cands = sorted(glob.glob(os.path.expanduser("~/.cache/ms-playwright/chromium-*/chrome-linux*/chrome")))
            if cands:
                launch_kwargs["executable_path"] = cands[-1]
        browser = pw.chromium.launch(**launch_kwargs)
        page = browser.new_page(viewport={"width": 540, "height": 960})

        def on_console(msg) -> None:
            text = msg.text
            if "ERROR" in text and ".meta" not in text and "favicon" not in text:
                console_errors.append(text[:300])
            if "missing" in text.lower() and "import" in text.lower():
                page_errors.append(text[:300])

        def on_pageerror(exc) -> None:
            page_errors.append(str(exc.message))

        def on_response(resp) -> None:
            u = resp.url
            if "/assets/" in u or "treasure-bevy-wasm" in u:
                asset_status[u.rsplit("8080/", 1)[-1] if "8080" in u else u] = resp.status

        page.on("console", on_console)
        page.on("pageerror", on_pageerror)
        page.on("response", on_response)

        # ── 加载 ──
        page.goto(args.url, wait_until="load", timeout=60_000)

        # ── L1 实例化：等 status 文案（wasm start → run() → 主循环活体证明）──
        status = ""
        for _ in range(args.timeout // 2):
            status = page.evaluate("document.getElementById('status')?.textContent ?? ''")
            if status.startswith(("运行中", "加载失败")):
                break
            page.wait_for_timeout(2_000)
        checks["L1_start"] = status

        # ── L2 渲染：等 Bevy 首帧资产落地，截图采像素（WebGL canvas 不能 2D drawImage 读，用 PIL）──
        page.wait_for_timeout(min(20_000, args.timeout * 1000))

        def bright_ratio() -> float:
            """截图全幅亮像素比（%）：select_map ≈ 17.7 / main_map ≈ 78。"""
            import io
            from PIL import Image
            im = Image.open(io.BytesIO(page.screenshot())).convert("RGB")
            w, h = im.size
            return 100.0 * sum(1 for px in im.getdata() if max(px) > 100) / (w * h)

        checks["L2_render"] = {"ok": True, "bright_ratio": round(bright_ratio(), 1)}

        asset_fail = {u: s for u, s in asset_status.items() if s >= 500 or (s >= 400 and ".meta" not in u)}
        checks["L2_assets"] = {"ok": sum(1 for s in asset_status.values() if s < 400),
                               "meta404": sum(1 for u, s in asset_status.items() if s == 404 and u.endswith(".meta")),
                               "fail": asset_fail}

        # ── L3 交互：Enter / Space / 鼠标点击，0 页面异常 = 通过 ──
        page.keyboard.press("Enter")
        page.wait_for_timeout(3_000)
        page.keyboard.press("Space")
        page.wait_for_timeout(3_000)
        page.mouse.click(270, 480)
        page.wait_for_timeout(3_000)
        after = page.evaluate("document.getElementById('status')?.textContent ?? ''")
        checks["L3_input"] = {"after": after, "page_errors": page_errors}

        # ── L4 场景切换：select_map 鼠标点 m01 卡片 ⇒ main_map（增量判据）──
        # 先确保在 select_map（L3 的 Space 可能已推进到 main_map，用 Back 退回）
        for _ in range(3):
            if bright_ratio() < 40.0:  # 已在 select_map（≈17.7%）
                break
            page.keyboard.press("Back")
            page.wait_for_timeout(2_000)
        before_bright = bright_ratio()
        # m01 卡片 = select_map 右侧面板顶行（540×960 视口：x≈430, y≈100）
        # 慢点（down/up 分帧）：bevy_picking 的 PointerClick = press 帧 + release 帧同实体，
        # 快速 click 两动作可能落同一帧 ⇒ 无 Click。
        page.mouse.move(430, 100)
        page.wait_for_timeout(1_000)
        page.mouse.down()
        page.wait_for_timeout(500)
        page.mouse.up()
        page.wait_for_timeout(5_000)
        after_bright = bright_ratio()
        checks["L4_scene"] = {
            "before": round(before_bright, 1),
            "after": round(after_bright, 1),
            "transitioned": before_bright < 40.0 and after_bright > 40.0,
        }

        if not args.check:
            Path(args.out).parent.mkdir(parents=True, exist_ok=True)
            page.screenshot(path=args.out)
            print(f"[wasm-browser-e2e] 截图 -> {args.out}")

        browser.close()

    # ── 判据汇总 ──
    l1 = isinstance(checks["L1_start"], str) and checks["L1_start"].startswith("运行中")
    l2r = checks["L2_render"]["ok"] and checks["L2_render"]["bright_ratio"] > 5.0
    l2a = not checks["L2_assets"]["fail"]
    l3 = not checks["L3_input"]["page_errors"]
    l4 = checks.get("L4_scene", {}).get("transitioned", False)
    print(json.dumps({"L1_start": l1, "L2_render": l2r, "L2_assets": l2a, "L3_input": l3,
                      "L4_scene": l4,
                      "detail": {k: checks[k] for k in ("L1_start", "L2_render", "L3_input", "L4_scene")}},
                     ensure_ascii=False, indent=2, default=str))
    ok = l1 and l2r and l2a and l3 and l4
    print(f"[wasm-browser-e2e] {'PASS' if ok else 'FAIL'}（L1 实例化={l1} / L2 渲染={l2r} 资产={l2a} / L3 交互={l3} / L4 场景切换={l4}）")
    if console_errors:
        print(f"[wasm-browser-e2e] console 硬错误 {len(console_errors)} 条（前 5）:")
        for e in console_errors[:5]:
            print("  ", e)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
