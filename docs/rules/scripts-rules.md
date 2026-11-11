---
paths:
  - "scripts/**"
  - "scripts/*.py"
  - "scripts/*.ps1"
  - "scripts/*.sh"
---
# scripts/ 编码红线（工具脚本域，CCGS `rules/*.md` `paths:` 模式借鉴，S50.3）

> 本文件 = `docs/rules/` 路径作用域红线速查面（给 LLM 子代理开轮读，**非机器拦截**；
> 机器拦截仍走 `check_tooling_contract`（doccheck2 第 27 项）+ `check_tooling_stray_scripts` /
> `check_tooling_registration`（工具台账三处登记 + 仓库外散落脚本）+ `check_dod_commands`（gate 第 25 项）
> + `studio_tests.py`（gate 第 25b 项，元测试面第 3 类断言 = 工具台账三处登记核验）。
> 单一事实源 = `docs/process/14_tooling_catalog.md`（工具唯一事实源 + 工具契约 4 条）+
> `docs/process/04_tooling.md`（速查表），本文件只做路径域速查 + 反例，不重述结论。

## 红线（4~6 条，违反 = 返工）

1. **工具契约 4 条全满足（§14 第 27 项）**：① 落地 `scripts/`（正式工具）/ `scripts/tmp/`（一次性，
   用完即删）② 台账登记 = 14 号表格行（`| \`x.py\` |`）③ 登记要素（用途/用法/幂等/只读模式）
   ④ 只读模式 + 非 0 退出（新工具须含 `--check`/`--verify`/`--report` 之一，违规非 0 退出）。
   反例：写 `scripts/tmp/fix_once.py` 用完不删 / 写 `scripts/gen_ui.py` 无 `--check` 只读模式。
2. **禁写仓库外**：脚本产物一律落仓库内（`evidence/gen_tmp/` / `evidence/ui_concept/` / `dist/` 等
   登记目录），不得写系统临时目录 / 桌面 / 仓库外路径（硬规则 10 + 14 号 §1.2 第 1 条）。反例：
   `ui_assets.py` 把原图写 `/tmp/ui_raw.png`（应落 `evidence/gen_tmp/raw_<name>.png`）。
3. **一次性脚本落 `scripts/tmp/` 且用完即删**：真的一次性改写不包进正式工具；`scripts/tmp/` 下脚本
   用完即删，不得留在仓库（`check_tooling_stray_scripts` 拦截仓库外散落 + 14 号 §1.2 第 1 条）。
   反例：把「这次改 3 处坐标」的一次性脚本 `scripts/patch_coords.py` 留仓不删。
4. **`--check` 只读 + 非 0 退出**：每个正式工具的 `--check`/`--verify`/`--report` 子命令须只读
   （不写仓库跟踪文件）+ 违规时非 0 退出码（门禁靠退出码判定）；幂等性 = 重复跑不产生 diff。
   反例：`map_pack.py sync` 的 `--check` 模式偷偷写了 `board.json`（应只读比对漂移，漂移才非 0）。
5. **命令白名单 / 逐参数非 shell（22 号 §4.2 / §10.3）**：平台/门禁经脚本跑命令时，程序只来自白名单
   （cargo / python3 / pwsh / powershell / node），逐参数 `Command::new(程序).args([...])`，禁 `sh -c` /
   `cmd /c` 字符串拼接（防 RCE）；判据命令载体须可达（gate 第 25 项 `check_dod_commands`：多 bin crate
   须显式 `--bin <name> --`，`python scripts/x.py` 须文件在位）。反例：`publish_web.ps1` 用
   `sh -c "git push origin gh-pages"` 字符串拼接（应 `git push` 逐参数）。

## 正确 / 反例（1~2 条，给子代理开轮速查）

- 正确：`map_pack.py` 带 `pack`/`verify`/`install`/`check` 子命令 + `--check` 只读幂等 + 14 号 §3.1 表格行
  登记 + 产物落 `assets/maps/<id>/` + `dist/maps/<id>.map-pack/`（仓库内登记目录）。
- 反例：`build_wasm.ps1` 写 `web/` 产物前不查 `gate_data.json` 冻结路径指纹（应经 24 号 §4.1 编排器
  `build_artifacts.py` 白名单 kind，写面登记 = `dist/`/`web/` + gitignored 日志，不污染 git 跟踪证据）。
