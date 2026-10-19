"""S15：把 6 场景的 hint 行从 `draw_body_text` 改成 `hint_row`（带浅色底条）。"""

import glob
import re

PAT = re.compile(
    r'chrome::draw_body_text\(\s*host,\s*\n?\s*(i18n::sget\("[^"]+"\)),\s*\n?\s*'
    r"layout\.hint\.x \+ 6,\s*\n?\s*layout\.hint\.y,?\s*\n?\s*\)"
)


def main() -> None:
    for path in glob.glob("crates/ti-app/src/scenes/*.rs"):
        src = open(path, encoding="utf-8").read()
        def repl(m: re.Match[str]) -> str:
            call = m.group(1)
            return (
                f"chrome::hint_row(host, {call}, layout.hint.x, layout.hint.y, "
                f"{call}.chars().count())"
            )
        new, n = PAT.subn(repl, src)
        if n:
            open(path, "w", encoding="utf-8").write(new)
            print(path.split("\\")[-1].split("/")[-1], n)


main()
