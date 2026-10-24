# -*- coding: utf-8 -*-
"""S7.4 多分辨率打包：assets_hd/ → assets_hd_pack/（1x/2x/4x + WebP 转码）。

读 manifest_hd.json（S7.2 产物）+ assets_hd/ 全量 PNG-32，
按分辨率档切分：
  - 1x/  = 原始逻辑尺寸真彩（首屏加载，移动端 1x）
  - 2x/  = 2x 放大（当前 assets_hd 默认产物，移动端/平板）
  - 4x/  = 4x 放大（桌面/PC，从 2x 再 NEAREST 2x 派生，零质量损失）
  - webp/ = wasm 侧 WebP 转码（减体积，Pillow 的 WebP 编码器）

产物结构：
  assets_hd_pack/
    1x/<容器>/<name>_hd.png
    2x/<容器>/<name>_hd.png     (= assets_hd 默认产物，硬链接/拷贝)
    4x/<容器>/<name>_hd.png     (从 2x NEAREST 2x 派生)
    webp/<容器>/<name>_hd.webp
    manifest_hd.json            (汇总 1x/2x/4x/webp 全部文件 FNV-1a64 + 尺寸，幂等)

多帧动画（_f0~_f4）与九宫格（_9s）同样进 4 档。
用法：python scripts/pack_hd_assets.py [--no-webp]
      python scripts/pack_hd_assets.py --check   # 只读校验：源在位 + pack 产物/哈希一致（不写文件）
零新增依赖（标准库 + Pillow）。
退出码：0 = 成功/校验通过；1 = 无法删除 pack 目录 / --check 校验失败（缺源、未打包、哈希漂移）。
"""
import json
import shutil
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
HD = REPO / "assets_hd"
PACK = REPO / "assets_hd_pack"
# manifest 与 assets_hd 同目录（S7.2 产物落 assets_hd/manifest_hd.json）
MANIFEST_SRC = HD / "manifest_hd.json"
MANIFEST_OUT = PACK / "manifest_hd.json"


def fnv1a64(b: bytes) -> str:
    h = 0xCBF29CE484222325
    for x in b:
        h ^= x
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return format(h, "016x")


def nearest_2x(rgba: bytes, w: int, h: int) -> tuple[bytes, int, int]:
    out = bytearray((w * 2) * (h * 2) * 4)
    for y in range(h):
        for x in range(w):
            s = (y * w + x) * 4
            for dy in range(2):
                for dx in range(2):
                    d = ((y * 2 + dy) * (w * 2) + (x * 2 + dx)) * 4
                    out[d : d + 4] = rgba[s : s + 4]
    return bytes(out), w * 2, h * 2


def open_rgba(path: Path):
    from PIL import Image

    img = Image.open(path).convert("RGBA")
    return img, img.width, img.height


def sanitize_alpha(img):
    """EXP-075：清理 alpha 半透明 + 透明区脏 RGB（预乘缩采样前置）。

    原版 PCX 精灵的背景像素是**调色板色**（本项目里是洋红），alpha=0。
    直接对 RGBA 做 Lanczos 缩采样会：
      1. 把 alpha=0 像素的 RGB（洋红）混进邻域 → 边缘洋红脏边；
      2. alpha 通道被重采样成 0..255 半透明 → blit 侧（只跳 a==0）当不透明画上去，
         表现就是「人物/立绘被洋红矩形包住」（EXP-075）。

    修法（与 ``scripts/enhance_portraits.py`` 的 alpha 二值化同口径）：
      - alpha 二值化（>=128 → 255，否则 0），杜绝半透明；
      - alpha=0 处 RGB 清零，杜绝透明像素带色参与插值。

    返回新图（不改调用方的原图语义）。
    """
    from PIL import Image

    w, h = img.size
    data = img.tobytes()
    out = bytearray(len(data))
    for i in range(0, len(data), 4):
        a = data[i + 3]
        if a >= 128:
            out[i] = data[i]
            out[i + 1] = data[i + 1]
            out[i + 2] = data[i + 2]
            out[i + 3] = 255
        else:
            # 全透明：RGB 一并清零（否则 Lanczos 会把这里的脏色渗进邻域）
            out[i] = 0
            out[i + 1] = 0
            out[i + 2] = 0
            out[i + 3] = 0
    return Image.frombytes("RGBA", (w, h), bytes(out))


def save_png(rgba: bytes, w: int, h: int, path: Path) -> None:
    from PIL import Image

    path.parent.mkdir(parents=True, exist_ok=True)
    Image.frombytes("RGBA", (w, h), rgba).save(path)


def check() -> int:
    """S21.8 工具契约只读模式：源在位 + pack 产物与 manifest 一致（不写任何文件）。"""
    if not MANIFEST_SRC.is_file():
        print(f"[pack_hd_assets --check] ERROR 缺 manifest 源: {MANIFEST_SRC}")
        return 1
    manifest_src = json.loads(MANIFEST_SRC.read_text())
    missing_src = [rel for rel in sorted(manifest_src) if not (REPO / rel).is_file()]
    if missing_src:
        print(f"[pack_hd_assets --check] FAIL 缺源文件 {len(missing_src)} 个（首 10）: {missing_src[:10]}")
        return 1
    out_manifest = PACK / "manifest_hd.json"
    if not out_manifest.is_file():
        print("[pack_hd_assets --check] FAIL 未打包（缺 assets_hd_pack/manifest_hd.json，先跑默认模式）")
        return 1
    packed = json.loads(out_manifest.read_text())
    missing_pack = [k for k in sorted(packed) if not (PACK / k).is_file()]
    if missing_pack:
        print(f"[pack_hd_assets --check] FAIL pack 产物缺失 {len(missing_pack)} 个（首 10）: {missing_pack[:10]}")
        return 1
    bad_hash = []
    for k, meta in sorted(packed.items()):
        fnv = meta.get("fnv1a64", "")
        if fnv and fnv1a64((PACK / k).read_bytes()) != fnv:
            bad_hash.append(k)
    if bad_hash:
        print(f"[pack_hd_assets --check] FAIL 哈希漂移 {len(bad_hash)} 个（首 10）: {bad_hash[:10]}")
        return 1
    print(f"[pack_hd_assets --check] OK（{len(manifest_src)} 源在位；{len(packed)} pack 产物 + 哈希一致；只读未写文件）")
    return 0


def main() -> int:
    if "--check" in sys.argv:
        return check()
    from PIL import Image

    no_webp = "--no-webp" in sys.argv
    manifest_src = json.loads(MANIFEST_SRC.read_text())

    if PACK.exists():
        import time

        for _ in range(10):
            try:
                shutil.rmtree(PACK)
                break
            except OSError:
                time.sleep(0.5)
        else:
            print(f"[pack_hd_assets] ERROR 无法删除 {PACK}（文件被占用？）")
            return 1
    (PACK / "1x").mkdir(parents=True)
    (PACK / "2x").mkdir(parents=True)
    (PACK / "4x").mkdir(parents=True)
    if not no_webp:
        (PACK / "webp").mkdir(parents=True)

    out_manifest: dict[str, dict] = {}
    counts = {"1x": 0, "2x": 0, "4x": 0, "webp": 0}

    for rel, meta in sorted(manifest_src.items()):
        container = meta.get("container", "misc")
        base_name = Path(rel).name  # e.g. 01_hd.png / 01_hd_9s.png / 01_hd_f0.png
        # 1x：从 2x 反向缩放（Lanczos 0.5x），保持与 2x 同源
        src = REPO / rel
        if not src.is_file():
            print(f"[pack_hd_assets] WARN 缺源 {rel}")
            continue
        img, w, h = open_rgba(src)
        # EXP-075：缩采样前置清理（alpha 二值化 + 透明区 RGB 清零），
        # 否则 1x 档 Lanczos 会把透明像素的洋红背景色渗进邻域。
        img = sanitize_alpha(img)
        rgba = img.tobytes()

        # 1x（首屏）：从 2x 反向 Lanczos 0.5x
        img1 = img.resize((max(1, w // 2), max(1, h // 2)), Image.LANCZOS)
        # EXP-075：缩采样后再二值化一次（插值仍可能产生 1..254 的软 alpha）
        img1 = sanitize_alpha(img1)
        r1, w1, h1 = img1.tobytes(), img1.width, img1.height
        p1 = PACK / "1x" / container / base_name
        save_png(r1, w1, h1, p1)
        out_manifest[f"1x/{container}/{base_name}"] = {
            "class": meta.get("class"),
            "container": container,
            "name": meta.get("name"),
            "width": w1,
            "height": h1,
            "fnv1a64": fnv1a64(p1.read_bytes()),
        }
        counts["1x"] += 1

        # 2x（= 源）
        p2 = PACK / "2x" / container / base_name
        p2.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(str(src), str(p2))
        out_manifest[f"2x/{container}/{base_name}"] = {
            "class": meta.get("class"),
            "container": container,
            "name": meta.get("name"),
            "width": w,
            "height": h,
            "fnv1a64": fnv1a64(p2.read_bytes()),
        }
        counts["2x"] += 1

        # 4x（从 2x NEAREST 2x 派生，零质量损失）
        r4, w4, h4 = nearest_2x(rgba, w, h)
        p4 = PACK / "4x" / container / base_name
        save_png(r4, w4, h4, p4)
        out_manifest[f"4x/{container}/{base_name}"] = {
            "class": meta.get("class"),
            "container": container,
            "name": meta.get("name"),
            "width": w4,
            "height": h4,
            "fnv1a64": fnv1a64(p4.read_bytes()),
        }
        counts["4x"] += 1

        # WebP（wasm 侧减体积；2x 档转码，宽高与 2x 一致）
        if not no_webp:
            pw = PACK / "webp" / container / (Path(base_name).stem + ".webp")
            pw.parent.mkdir(parents=True, exist_ok=True)
            img.save(pw, format="WEBP", quality=90)
            out_manifest[f"webp/{container}/{Path(base_name).stem}.webp"] = {
                "class": meta.get("class"),
                "container": container,
                "name": meta.get("name"),
                "width": w,
                "height": h,
                "fnv1a64": fnv1a64(pw.read_bytes()),
            }
            counts["webp"] += 1

    MANIFEST_OUT.write_text(json.dumps(out_manifest, ensure_ascii=False, indent=2, sort_keys=True))
    print(f"[pack_hd_assets] {sum(counts.values())} 文件（1x={counts['1x']} 2x={counts['2x']} 4x={counts['4x']} webp={counts['webp']}）→ {PACK}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
