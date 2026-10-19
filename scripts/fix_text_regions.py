"""S15 v2 版面：render_intent 的 text_regions 对齐新坐标。"""

path = "crates/ti-app/tests/render_intent.rs"
src = open(path, encoding="utf-8").read()
src = src.replace('(0, 0, 176, 40, "标题条")', '(0, 4, 176, 28, "标题条")')
src = src.replace('(4, 208, 172, 220, "状态行")', '(4, 192, 172, 216, "状态行")')
src = src.replace('(4, 208, 172, 220, "提示行")', '(4, 188, 172, 216, "提示行")')
open(path, "w", encoding="utf-8").write(src)
print("done")
