"""8-bit hex → RGB565（R11G5B10，round 量化）换算器。

用途：S15 UI 重设计换算羊皮纸/木纹/金色系色值。
**必须脚本换算，勿手算** —— S15 首版手算 9 个色值全部错位，渲染出「橙粉塑料色」，
被模型自审判为最廉价（evidence/ui_concept/review_render_index.txt）。

用法：
    python scripts/rgb565.py                 # 打印本文档 §2 表的全部换算
    python scripts/rgb565.py 8B4513 C9A86A   # 换算指定 hex
"""

from __future__ import annotations

import sys

# docs/business/08_layout_design.md §2 色板（常量名, hex）—— 单一事实源在文档，此处仅供换算核对
PALETTE: list[tuple[str, str]] = [
    ("COLOR_PARCH_DARK", "12141C"),
    ("COLOR_PARCH_LIGHT", "1E2230"),
    ("COLOR_WOOD", "3A4058"),
    ("COLOR_GOLD", "F5B841"),
    ("COLOR_GOLD_GLOW", "42A5F5"),
    ("COLOR_GOLD_BRIGHT", "1E88E5"),
    ("COLOR_MENU_BG", "242838"),
    ("COLOR_MENU_BORDER", "3A4058"),
    ("COLOR_BEVEL_HI", "30364A"),
    ("COLOR_BEVEL_SHADOW", "0A0C12"),
    ("COLOR_TEXT_ON_PARCH", "FFFFFF"),
    ("COLOR_TEXT_ON_WOOD", "FFF0C0"),
]


def to565(value: int) -> int:
    """8-bit 整数 RGB → RGB565（R11G5B10，round 量化）。"""
    r = (value >> 16) & 0xFF
    g = (value >> 8) & 0xFF
    b = value & 0xFF
    return (round(r * 31 / 255) << 11) | (round(g * 63 / 255) << 5) | round(b * 31 / 255)


def back_rgb(value565: int) -> str:
    """RGB565 → 8-bit hex（量化回读，用于验证往返误差）。"""
    r5 = (value565 >> 11) & 0x1F
    g6 = (value565 >> 5) & 0x3F
    b5 = value565 & 0x1F
    r = (r5 * 255) // 31
    g = (g6 * 255) // 63
    b = (b5 * 255) // 31
    return f"#{r:02X}{g:02X}{b:02X}"


def main() -> None:
    args = sys.argv[1:]
    if args:
        items = [(f"#{a.upper()}", a.lstrip("#")) for a in args]
    else:
        items = [(name, hexv) for name, hexv in PALETTE]
    for name, hexv in items:
        v = to565(int(hexv, 16))
        print(f"{name:>22} = 0x{v:04X}   #{hexv.upper()} -> {back_rgb(v)}")


main()
