# -*- coding: utf-8 -*-
"""去逆向化标注清理 v4：补充遗漏的 sub_xxxx 引用（第二轮）。"""
import glob
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# 第一轮后剩余的 sub 补充映射（从上下文语义提取）
EXTRA_MAP = {
    "sub_29110": "场景管理器",
    "sub_41354": "主菜单",
    "sub_427E8": "退出",
    "sub_4085C": "选项",
    "sub_4220C": "存读档",
    "sub_42378": "继续",
    "sub_428F0": "新游戏",
    "sub_3F4E4": "帮助",
    "sub_3D4C8": "设置",
    "sub_2101C": "帧更新",
    "sub_1CCE8": "精灵注入",
    "sub_4F904": "目标场景",
    "sub_43D94": "场景注入",
    "sub_3FE84": "角色控件",
    "sub_2FC44": "容器加载",
    "sub_41B5C": "角色头像",
    "sub_3F8FC": "帮助翻页",
    "sub_42390": "选项控件",
    "sub_4290C": "帮助控件",
    "sub_20088": "资源装载",
    "sub_1E830": "容器索引",
    "sub_33548": "按钮注入",
    "sub_3A058": "控件注入",
    "sub_42AE8": "存档写",
    "sub_1F20C": "存档写",
    "sub_52080": "文件存",
    "sub_51F24": "文件读",
    "sub_498A4": "存读档",
    "sub_2077C": "主地图",
    "sub_4D4C4": "存读档",
    "sub_1F7B8": "存档读",
    "sub_48AC0": "资源读取",
    "sub_48654": "据点定位",
    "sub_4D1B0": "资源写入",
    "sub_33E20": "事件",
    "sub_33E3C": "事件",
}

SUB_PATTERN = re.compile(r"sub_([0-9A-Fa-f]{4,6})\b")


def main() -> None:
    dry_run = "--dry-run" in sys.argv
    # 合并第一轮映射 + 补充映射
    from de_reverse_annotate import build_sub_semantic_map
    sub_map = build_sub_semantic_map()
    sub_map.update(EXTRA_MAP)
    print(f"Mapped {len(sub_map)} sub entries (incl. {len(EXTRA_MAP)} extra)")

    target_files: list[str] = []
    for pattern in [
        "crates/ti-core/src/rules/*.rs",
        "crates/ti-core/src/oracle/*.rs",
        "crates/ti-app/src/*.rs",
        "docs/impl/*.md",
        "legacy/docs/design/*.md",
    ]:
        for fp in glob.glob(os.path.join(REPO, pattern)):
            if os.path.isfile(fp):
                target_files.append(fp)

    total = 0
    all_missing: set[str] = set()
    for fp in target_files:
        with open(fp, encoding="utf-8") as fh:
            text = fh.read()

        count = 0
        missing: set[str] = set()

        def repl(m: re.Match) -> str:
            nonlocal count
            full = m.group(0)
            if full in sub_map:
                count += 1
                return sub_map[full]
            missing.add(full)
            return full

        new_text = SUB_PATTERN.sub(repl, text)

        if count > 0:
            if not dry_run:
                with open(fp, "w", encoding="utf-8") as fh:
                    fh.write(new_text)
            rel = os.path.relpath(fp, REPO)
            tag = "[dry-run] " if dry_run else ""
            miss_msg = f" (missing: {', '.join(sorted(missing))})" if missing else ""
            print(f"  {tag}{rel}: {count} refs{miss_msg}")
            total += count
            all_missing.update(missing)

    if all_missing:
        print(f"\nStill missing ({len(all_missing)}): {', '.join(sorted(all_missing))}")
    print(f"\nTotal: {total} sub_xxxx refs replaced")
    if dry_run:
        print("(dry-run: no files written)")


if __name__ == "__main__":
    main()
