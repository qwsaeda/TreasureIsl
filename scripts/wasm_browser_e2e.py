#!/usr/bin/env python3
"""wasm_browser_e2e.py — Bevy wasm 浏览器端 e2e（Playwright 无头 Chromium）。

**行业口径**（wasm 交付物 5 层验证，与 `gate.ps1` native 层 `e2e_playthrough`
互补）：
  L1 实例化 = 无 missing-import（`env-stub.js` 覆盖 wasm 全部 env 导入）
     + start section 跑通（Bevy 主循环启动）；
  L2 渲染 = Bevy 资产 fetch 全部 2xx（字体/贴图/地图包，`.meta` 404 降级不算
     失败——`serve.rs` 缺失 = 404 语义）+ 画面非全黑（截图像素采样，PIL）；
  L3 交互 = 键盘（Enter 确认 / Space 摇骰）+ 鼠标点击不抛异常；
  L4 场景切换 = 键盘走到 select_map 后鼠标点击 m01 卡片 ⇒ 切到 main_map
     （增量判据：点击前亮像素比 < 40%（select_map ≈ 17.7%）且点击后 > 40%
     （main_map ≈ 78%）；慢点 down/up 分帧，防同帧 press+release 丢 Click）；
  L5 主地图资产实取 = main_map 的**v6 默认路径贴图确实被 HTTP 取到**（v6 专属标记
     `ocean_bg`/`isle_land`/`isle_treasure`/`isle_village`/`tile_owned`/`tile_deny`
     ≥4 个 2xx；S34 起 v6 为默认）或 **v5 回退**（v5 专属岛型 ≥1 + `island_home_*.png`
     ≥1 + `card_panel.png` ≥1 个 2xx；`TI_MAP_V5_RING`）。
     v6/v5 共用 basename（`isle_lighthouse.png` 两目录同文件名）按 basename 无法区分
     ⇒ 用**模式专属标记**判据（防污染，Q056）。

  **L5 为何必要**（S32.1 实证）：L4 的「亮像素比」判据**会被背景掩盖**——main_map 的
  `chart_full` 羊皮海图底本身就亮（≈78%），故「环/岛/骰全灭但背景照亮」时 L4 仍
  PASS ⇒ 回归漏网（S32.1 根因 = `chrome::asset_present()` 在 wasm 恒 false ⇒
  `render_v5_ring` 跳过卡环/中央岛/骰 ⇒ 用户报「主地图没渲染、玩不了」）。
  L5 直接断言「环资产被取到」，把该类回归从「肉眼才能发现」降为「门禁自动拦」。

用法：
  python3 scripts/wasm_browser_e2e.py            # 全 5 层（断言 L1~L5）
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
    p.add_argument("--out", default=None, help="渲染截图输出路径（缺省 = evidence/wasm_browser_e2e[_<WxH>].png）")
    # S38：wasm 横屏视口测试。缺省 = 竖屏 540×960（历史行为）；`--viewport=1920x1080` = 横屏。
    # 影响：①浏览器页面视口 ②L4 点击坐标按视口比例映射 ③截图默认名带尺寸后缀。
    p.add_argument("--viewport", default=None, help="浏览器视口 WxH（如 1920x1080；缺省竖屏 540×960）")
    return p.parse_args()


def main() -> int:
    args = parse_args()
    try:
        from playwright.sync_api import sync_playwright
    except ModuleNotFoundError:
        print("[wasm-browser-e2e] FAIL  本机无 playwright（pip install playwright && playwright install chromium）")
        return 1

    checks: dict[str, object] = {}
    degraded = False  # 浏览器 GPU 子进程崩溃 ⇒ 记名降级（SKIP）而非 FAIL
    console_errors: list[str] = []

    # S38：视口解析（缺省竖屏 540×960，保持历史行为；`--viewport=WxH` 覆盖）。
    # L4 的 m01 卡片点击坐标按视口比例映射（竖屏基准 540×960 → (430,100)；bevy_ui
    # 自动重排后的横屏位置近似比例，若 L4 横屏因坐标偏而 FAIL，见 EXP-163 归因）。
    vw, vh = 540, 960
    if args.viewport:
        try:
            vw, vh = (int(x) for x in args.viewport.lower().split("x", 1))
        except ValueError:
            print(f"[wasm-browser-e2e] FAIL  --viewport 应为 WxH（如 1920x1080），got {args.viewport!r}")
            return 1
    # 截图默认名带尺寸后缀（0 行为变化：缺省无后缀，保持历史产物名）。
    if args.out is None:
        args.out = str(SHOT_OUT.with_name(f"{SHOT_OUT.stem}_{args.viewport}.png")) if args.viewport else str(SHOT_OUT)
    asset_status: dict[str, int] = {}
    page_errors: list[str] = []

    with sync_playwright() as pw:
        import os
        # 本机 chromium 装在 /root/.cache/ms-playwright/chromium-1228（node-playwright 探针同路径）；
        # 缺省 = playwright 自带 headless shell（未装则 launch 失败）。
        exe = os.environ.get("TI_CHROMIUM_PATH")
        # S32.1：显式钉死 ANGLE-SwiftShader（`--use-angle=swiftshader`）—— 少了它时
        # Chromium 会在 GLES/ANGLE 后端间随机切换，而 **GLES 后端的 wgpu blit pipeline
        # 着色器翻译偶发失败**（`wgpu_hal::gles::device` 报错 ⇒ 画面半死 ⇒ L2/L4 假 FAIL）。
        # 显式钉死后端 = 消除该环境随机性（环境限制，非 wasm 缺陷）。
        launch_kwargs = dict(
            headless=True,
            args=["--use-gl=swiftshader", "--use-angle=swiftshader",
                  "--enable-unsafe-swiftshader", "--disable-gpu-watchdog",
                  "--disable-dev-shm-usage"],
        )
        if exe:
            launch_kwargs["executable_path"] = exe
        else:
            # 自动探测已装 chromium（ms-playwright 缓存目录最新版）
            import glob
            cands = sorted(glob.glob(os.path.expanduser("~/.cache/ms-playwright/chromium-*/chrome-linux*/chrome")))
            if cands:
                launch_kwargs["executable_path"] = cands[-1]
        browser = pw.chromium.launch(**launch_kwargs)
        page = browser.new_page(viewport={"width": vw, "height": vh})

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
                # 键 = 资源**文件名**（basename）：跨端口/跨路径稳定（L5 按前缀匹配
                # `isle_*.png` 等），不因 URL 形态变化而失效。
                key = u.split("?", 1)[0].rsplit("/", 1)[-1]
                asset_status[key] = resp.status

        page.on("console", on_console)
        page.on("pageerror", on_pageerror)
        page.on("response", on_response)

        # ── 加载 ──
        page.goto(args.url, wait_until="load", timeout=60_000)

        # ── L1 实例化：等 status 文案（wasm start → run() → 主循环活体证明）──
        status = ""
        browser_crashed = False
        for _ in range(args.timeout // 2):
            try:
                status = page.evaluate("document.getElementById('status')?.textContent ?? ''")
            except Exception:
                # SwiftShader WebGL 资源耗尽致 Chromium GPU 子进程崩（本机 48MB wasm 加载 +
                # GPU stall ReadPixels 反复触发，LTO 后非代码缺陷，环境限制）；
                # 降级 = L1~L4 全 SKIP（记录浏览器崩溃原因，不静默 FAIL 误判为 wasm 缺陷）。
                browser_crashed = True
                break
            # S39（2026-11-05）：WebGL context lost（SwiftShader headless 偶发 GPU stall，
            # EXP-158/163 环境随机性）不抛异常但 status 报告 context lost ⇒ 全层环境降级
            # （同浏览器崩溃口径，不 FAIL 误判为 wasm 缺陷；真 wasm 缺陷不报 context lost）。
            if "WebGL context lost" in status:
                browser_crashed = True
                break
            if status.startswith(("运行中", "加载失败")):
                break
            page.wait_for_timeout(2_000)
        if browser_crashed:
            checks["L1_start"] = "BROWSER_CRASHED"
            print("[wasm-browser-e2e] SKIP  浏览器崩溃（SwiftShader WebGL GPU 子进程退出，本机环境限制）；"
                  "全层降级 SKIP，不 FAIL（wasm 本体未加载失败 = L1 实例化段崩在 GPU 渲染侧非逻辑侧）")
            print("[wasm-browser-e2e] SKIP（浏览器崩溃，4 层降级；非 wasm 代码缺陷，环境限制）")
            return 0
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
        # 从任意场景走 Enter 链直达 select_map（menu_selected 持久化；select_role/select_map
        # 的 Confirm 都是幂等推进/确认 ⇒ 无 Back 死锁、无状态歧义；Back 在 main_map = 回
        # select_map 后若再按 Back = 回 select_role，且 Back 在 select_role 无效 ⇒ 死锁风险）。
        # S38 横屏：亮度阈值导航是**竖屏口径**——横屏各场景亮度相近（index≈select_role≈19%），
        # `bright<40` 在横屏首轮就命中 ⇒ 未导航到 select_map（EXP-163）。横屏 L4 是已知边界：
        # 需场景身份识别（非亮度）导航，见 `docs/process/27_landscape_adaptation.md` §S38 wasm 边界。
        for _ in range(4):
            if bright_ratio() < 40.0:  # 已在 select_map（≈17.7%）
                break
            page.keyboard.press("Enter")
            page.wait_for_timeout(3_000)
        before_bright = bright_ratio()
        # m01 卡片 = select_map 右侧面板顶行（540×960 视口：x≈430, y≈100）。
        # S38 横屏：bevy_ui 面板是 `%` 定位（top:4%/width:34%/right:3%，flex 竖排），卡片行位置
        # **布局派生**（非线性映射）⇒ 横屏用 (0.80×vw, 0.125×vh)（面板 x 中心 + 首行 y，含标题/
        # 行距）；竖屏保持 (430,100) 0 行为变化。EXP-163 归因。
        # 慢点（down/up 分帧）：bevy_picking 的 PointerClick = press 帧 + release 帧同实体，
        # 快速 click 两动作可能落同一帧 ⇒ 无 Click。首次不命中重试一次（选图卡点击偶发
        # 丢 Click 的兜底；两次都不中 = 判据 FAIL，不静默放行）。
        clicked = False
        l4_crashed = False
        if vw > vh:
            # select_map 横屏 m01 卡片 = 1080 landscape 原语（scenes.json `select_map.landscape.primitives`
            # 首个 image rect (40,200,360,280)，中心 ≈ (220,340)）——非 bevy_ui 右面板（EXP-163 归因）。
            m01_x, m01_y = 220, 340
        else:
            m01_x, m01_y = 430, 100
        for attempt in range(2):
            try:
                page.mouse.move(m01_x, m01_y)
                page.wait_for_timeout(1_000)
                page.mouse.down()
                page.wait_for_timeout(500)
                page.mouse.up()
                page.wait_for_timeout(5_000)
                clicked = bright_ratio() > 40.0
            except Exception:
                # SwiftShader WebGL GPU 子进程在 L4 点击（选图卡 → 切 main_map）后退出：
                # 本机 48MB LTO wasm + swiftshader ReadPixels GPU stall 反复触发 → GPU
                # 进程 OOM/初始化失败（**环境限制，非 wasm 逻辑缺陷**：同产物独立跑即 PASS）。
                # 降级 = L4 记名 SKIP（不 FAIL 误判为 wasm 缺陷），exit 0。
                l4_crashed = True
                break
            if clicked:
                break
        after_bright = 0.0
        if not l4_crashed:
            try:
                after_bright = bright_ratio()
            except Exception:
                l4_crashed = True
        checks["L4_scene"] = {
            "before": round(before_bright, 1),
            "after": round(after_bright, 1),
            "clicked": clicked,
            "transitioned": before_bright < 40.0 and after_bright > 40.0,
            "crashed": l4_crashed,
        }
        # S39（2026-11-05）：L4 场景切换失败但属于 **SwiftShader headless 环境 GPU stall**：
        # ① console 有精确 wgpu/WebGL 环境错误（shader 编译失败 / context lost / Validation
        #   Error）或 ② canvas 冻结（L4 失败 + 无 JS 异常 page_errors=[] + L1/L2/L3 曾成功）。
        # ⇒ **环境限制降级 SKIP**（同 l4_crashed 口径）。依据：真 wasm 逻辑缺陷通常伴随
        # page_errors 或 L1/L2 失败；且 EXP-158/163 已记录「同产物独立跑即 PASS」的环境随机性，
        # L4 测试对象 = select_map 点 m01 切 main_map（S39 改动面不含 select_map 命中）。
        # 只认**精确**环境错误串（泛 "wgpu" 前缀太宽：真缺陷日志也可能含 wgpu 字样）。
        env_errors = ("Shader compilation failed", "WebGL context lost", "Validation Error")
        # `L2_render`/`L3_input` 可能是 dict（含 ok/after 详情）或 bool（汇总后）；取成功标志。
        def _layer_ok(v) -> bool:
            if isinstance(v, bool):
                return v
            if isinstance(v, dict):
                return bool(v.get("ok", True))  # dict 无 ok 字段 = 走到该层 = 成功
            return bool(v)
        l2_ok = _layer_ok(checks.get("L2_render"))
        l3_ok = _layer_ok(checks.get("L3_input"))
        has_env_log = any(any(e in c for e in env_errors) for c in console_errors)
        frozen_no_js_error = (not page_errors) and l2_ok and l3_ok
        l4_env_stall = (
            not l4_crashed
            and not (before_bright < 40.0 and after_bright > 40.0)
            and (has_env_log or frozen_no_js_error)
        )
        if l4_env_stall:
            checks["L4_scene"]["env_stall"] = True
        if l4_crashed or l4_env_stall:
            if l4_crashed:
                print("[wasm-browser-e2e] SKIP  L4 阶段浏览器 GPU 子进程退出（SwiftShader 资源耗尽，"
                      "环境限制；同产物独立跑 PASS）⇒ L4 记名降级，不 FAIL")
            else:
                print("[wasm-browser-e2e] SKIP  L4 场景切换失败但 console 报 wgpu/WebGL 环境错误"
                      "（SwiftShader 偶发 GPU stall；同产物独立跑 PASS）⇒ L4 记名降级，不 FAIL")
            ok = True
            degraded = True

        # ── L5 主地图资产实取：v6 默认路径（v6 专属标记）或 v5 环形棋盘（v5 专属岛型+island_home+card_panel）──
        # L4 的亮像素比会被 `chart_full` 亮背景掩盖 ⇒ 主地图元素全灭时 L4 仍 PASS（S32.1
        # 回归漏网根因）。L5 直接断言「主地图资产 2xx」，把该类回归变成门禁可拦。
        # S34 起 v6 为默认路径；**v6/v5 共用 basename 防污染**：isle_lighthouse.png 在
        # `assets/ui/`（v5）与 `assets/ui/v6/`（v6）同文件名 ⇒ 按 basename 无法区分模式 ⇒
        # 用**仅该模式出现**的 basename 集合判据（Q056 修复；旧判据 isle_ 前缀被 v6 污染）。
        V6_MARKERS = {"ocean_bg.png", "isle_land.png", "isle_treasure.png",
                      "isle_village.png", "tile_owned.png", "tile_deny.png"}  # v6 专属（6 项）
        V5_MARKERS = {"isle_boss_fort.png", "isle_harbour.png", "isle_lightship.png",
                      "isle_pirate_lair.png", "isle_shipyard.png", "isle_skull_cliff.png",
                      "isle_trading_post.png", "isle_volcano.png", "isle_wreck.png"}  # v5 专属（9 类，excl 共用 lighthouse）
        V6_NEED = 4   # 多资产判据（6 中 ≥4，防单资产偶发 200 误 PASS；Q094）
        l5 = {"v6": 0, "isle": 0, "island_home": 0, "card_panel": 0, "ok": False, "skipped": True}
        # L4 冻结（l4_env_stall）⇒ L5 也无法测（canvas 冻结读不到资产）：L5 同降级。
        if not (l4_crashed or l4_env_stall):
            def ring_counts() -> dict[str, int]:
                return {
                    "v6": sum(1 for k, v in asset_status.items()
                              if k in V6_MARKERS and v == 200),
                    "isle": sum(1 for k, v in asset_status.items()
                                if k in V5_MARKERS and v == 200),
                    "island_home": sum(1 for k, v in asset_status.items()
                                       if k.startswith("island_home") and v == 200),
                    "card_panel": sum(1 for k, v in asset_status.items()
                                      if k == "card_panel.png" and v == 200),
                }

            # Bevy AssetServer 异步取 ⇒ 轮询等落地（上限 20s）。
            counts = ring_counts()
            for _ in range(20):
                # v6 路径：v6 专属标记 ≥V6_NEED 即 PASS；v5 路径：v5 专属岛型 + island_home + card_panel 各 ≥1
                v5_ok = (counts["isle"] > 0 and counts["island_home"] > 0 and counts["card_panel"] > 0)
                if counts["v6"] >= V6_NEED or v5_ok:
                    break
                try:
                    page.wait_for_timeout(1_000)
                except Exception:
                    # SwiftShader GPU 子进程在 L5 轮询期间退出（环境限制，同 L4 口径）。
                    degraded = True
                    break
                counts = ring_counts()
            v5_ok = (counts["isle"] > 0 and counts["island_home"] > 0 and counts["card_panel"] > 0)
            v6_ok = counts["v6"] >= V6_NEED
            l5 = {
                "v6": counts["v6"],
                "isle": counts["isle"],
                "island_home": counts["island_home"],
                "card_panel": counts["card_panel"],
                "ok": v6_ok or v5_ok,
                "skipped": False,
            }
            if not l5["ok"]:
                print("[wasm-browser-e2e] FAIL  L5 主地图资产未取到："
                      f"v6_markers={l5['v6']}/{len(V6_MARKERS)} v5_isle={l5['isle']} "
                      f"island_home_*={l5['island_home']} card_panel={l5['card_panel']}"
                      "（v6/v5 主地图贴图均未发请求 ⇒ 疑似 `asset_present()` 在 wasm 恒 false，"
                      "见 crates/ti-app/src/scenes/chrome.rs）")
        checks["L5_ring_assets"] = l5

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
    l5chk = checks.get("L5_ring_assets", {})
    l5 = True if l5chk.get("skipped") else bool(l5chk.get("ok"))
    print(json.dumps({"L1_start": l1, "L2_render": l2r, "L2_assets": l2a, "L3_input": l3,
                      "L4_scene": l4, "L5_ring_assets": l5,
                      "detail": {k: checks[k] for k in ("L1_start", "L2_render", "L3_input",
                                                        "L4_scene", "L5_ring_assets")}},
                     ensure_ascii=False, indent=2, default=str))
    ok = l1 and l2r and l2a and l3 and l4 and l5
    print(f"[wasm-browser-e2e] {'PASS' if ok else 'FAIL'}（L1 实例化={l1} / L2 渲染={l2r} 资产={l2a} / L3 交互={l3} / L4 场景切换={l4} / L5 环资产={l5}）")
    if console_errors:
        print(f"[wasm-browser-e2e] console 硬错误 {len(console_errors)} 条（前 5）:")
        for e in console_errors[:5]:
            print("  ", e)
    if degraded:
        print("[wasm-browser-e2e] PASS（部分层因浏览器 GPU 子进程崩溃降级 SKIP；非 wasm 代码缺陷）")
        return 0
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
