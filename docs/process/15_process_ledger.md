# 15 · 流程固化台账（把纪律变成可执行对象）

> **本文件 = 「人反复要做的开发动作」的单一事实源**：每条流程登记「现状人工做法 / 复发证据 /
> 自动化形态 / 载体 / 状态」。**状态只允许 4 档**：
>
> | 状态 | 含义 | 强制手段 |
> |---|---|---|
> | `已自动化` | 有脚本或门禁断言兜住，人不做也不出事 | 载体列的工具/断言 |
> | `半自动` | 工具已给产出物，但有一环**必须人看**（目视/裁决） | 载体 + 备注说明人看哪一步 |
> | `待固化` | 反复发生但尚无载体 ⇒ **必须带台账任务号**，否则不允许长期挂账 | 挂台账任务号 |
> | `一次性` | 只做一次，不值得固化 | 说明为何一次性 |
>
> **纪律不执行 = 纪律不存在**。本页每条「必须 / 须」类纪律，要么有载体（断言/工具），
> 要么登记为 `待固化` + **台账任务号**（本轮 6 条待固化项统一挂 `S21.13`），两者都没有 ⇒
> 视为该纪律无效，不得再写进其它文档当约束。
> 台账自洽性由 `doccheck2` 第 `check_process_ledger` 项判定（状态枚举 / 载体存在 / 任务号 / 本页已登记索引）。

## 1. 为什么有本页（复发证据）

| 家族 | 表现 | 根因 | 本页对策 |
|---|---|---|---|
| EXP-113 / EXP-115 | 同一逻辑被写多遍；一次性脚本写到仓库外临时目录，事后无法复用 | **缺工具**（规范要求的动作没有对应脚本） | 「组合优先」写入 `14_tooling_catalog` §1.5；缺工具 ⇒ 补 `scripts/` 工具 + 登记 3 处（第 5 条） |
| EXP-114 | UI 视觉改动先落码后出图，返工堆在收口阶段 | **纪律无载体**（「用户确认」只存在于聊天） | 待固化（S21.9 随出图验收落地断言） |
| EXP-111 / EXP-051 | cmdlet 写 CJK 文件 ⇒ BOM / 坏字节 / 换行被改写 | **写入路径无护栏** | 第 3 条（文本完整性断言）+ `task_close.py` 只用 UTF-8 写盘 |
| 本轮飞轮 R1 实证 | R1 修复 agent 用 cmdlet 写 CJK ⇒ 4 个文件被塞 BOM；另发现存量 1 处 U+FFFD 乱码 + 1 个仓库根垃圾文件 | 同上 | 第 3 条（机器判定，不靠"我记得"） |

## 2. 台账（流程 → 载体 → 状态）

| # | 流程 | 现状（人工做法） | 复发证据 | 自动化形态 | 载体 | 状态 |
|---|---|---|---|---|---|---|
| 1 | **任务收口**（跑判据 → 写证据 → 置 verified → 同步台账计数） | 手工三步：手敲命令 + `Tee-Object` 写 log + 手改台账行 + 手改 release.md 计数 | 硬规则 2；EXP-058/065（回退/虚报）；本轮「S21.2 先 verified 后 evidence」被 `check_ledger_reality` 当场抓住 | 脚本一条命令 | `scripts/task_close.py` + `scripts/check_ledger_reality.py` | 已自动化 |
| 2 | **EXP 登记**（分卷正文 + 索引行 + 分卷条数列） | 手改 3 处 | EXP-108（索引长期缺行、条数漂移） | 脚本一条命令 | `scripts/exp_log.py` + `check_experience_volume_counts` | 已自动化 |
| 3 | **仓库文本完整性**（BOM / UTF-8 合法 / 无乱码字符） | 靠"用编辑工具写"的自觉 | EXP-051/091/111；本轮 R1 实证（4 文件被塞 BOM）+ 存量 1 处 U+FFFD | 门禁断言 | `doccheck2::check_repo_text_integrity` | 已自动化 |
| 4 | **改动面声明**（每任务声明改哪一块 + 跨模块原因） | 手打进 dod，无任何判定 | `00_index.md` §纪律；台账 200 任务仅 22 处含声明 | 门禁断言 | `doccheck2::check_task_change_scope` | 已自动化 |
| 5 | **新增工具登记**（工具唯一事实源 + 速查表 + 全库索引） | 手改 3 处，机器只查第 1 处 | EXP-113；S20.1/S21.0 dod 自认「五处/八处同改」 | 门禁断言（棘轮：新工具零豁免） | `doccheck2::check_tooling_registration` | 已自动化 |
| 6 | **流程台账自身一致性**（状态枚举 / 载体存在 / 待固化带任务号） | 无 | 本页新增 | 门禁断言 | `doccheck2::check_process_ledger` | 已自动化 |
| 7 | **一次性改写脚本落位 + 用完即删** | 靠自觉（曾写仓库外） | EXP-115 | 门禁断言 | `doccheck2::check_tooling_contract`（`scripts/tmp/` 残留 = FAIL） | 已自动化 |
| 8 | **对抗飞轮开轮**（3 角色 prompt + 100/轮目标 + 停机判据） | 手搓 prompt | 06_flywheel §8 本轮新增 | 文档 + skill 模板 | `docs/process/06_flywheel.md` §8 + `flywheel-kickoff` skill | 已自动化 |
| 9 | **UI 资产批次生产**（登记 → 并行生图 → 抠图 → 预检 → 拼版） | 写临时脚本（仓库外） | EXP-115 | 脚本 | `scripts/ui_asset_batch.py` | 已自动化 |
| 10 | **版面线图 / 出图验收**（坐标表线图 + 真实渲染 + 像素取证） | 内联 `python -c` 手搓 | 本轮 S21.3 实证 | 脚本 | `scripts/ui_layout_sheet.py` + `scripts/ui_preview.ps1` + `scripts/ui_shot_probe.py` | 已自动化 |
| 11 | **读不出的文件静默跳过**（`doccheck` 读文件失败就 continue，坏字节文件整份失声） | 靠「计数漂移」倒查 | EXP-091（release.md 292 处坏字节） | 扩展 `doccheck`（读失败即 ERROR + 列文件名/字节偏移） | `doccheck::read_docs` 读失败即 ERROR（EXP-091 兜住） | 已自动化 |
| 12 | **门禁项数声称同步**（一改要手抄 8 处：parity/git/README/loop/AGENTS/style/playbook/横幅） | 手改 8 处 | S20.0c dod 自认牵动 7 处 | 脚本 `--fix`（从权威源实测回写） | `scripts/doc_numbers.py` — `--fix` 回写 8 处，默认只读体检 | 已自动化 |
| 13 | **视觉改动「设计稿确认」留痕**（A 纪律） | 留痕只在聊天，dod 手写 | EXP-114；S20.3 dod 自认「暂无机械门禁断言」 | 门禁断言（证据须含确认标记） | `doccheck2::check_visual_design_signoff` | 已自动化 |
| 14 | **布局规格坐标机器校验**（8pt / N8 往返 / N3-N6 / 与 scenes.json 一致） | 人手算并写进 evidence | S21.3 dod「坐标一致性自证」 | 脚本（解析规格表 → 比对 scenes.json + manifest） | `scripts/layout_spec_check.py` | 已自动化 |
| 15 | **节点 kind ↔ 资产映射唯一事实源** | 规格文档 → 手工抄进 chrome.rs + manifest | S21.1 批次 15 资产；S21.5 dod | 门禁断言（映射 key 集合 == manifest 资产集合 == 规格表） | `doccheck2::check_node_kind_asset_map` | 已自动化 |
| 16 | **`.ps1` 入口语法 + BOM 自检**（BOM 已由第 3 条覆盖；ParseFile 语法未覆盖） | 无 | EXP-109（playbook/smoke 缺 BOM 从未跑起来） | 脚本（ParseFile 语法自检） | `scripts/check_ps_scripts.ps1` | 已自动化 |
| 17 | **`src` 内联 `#[cfg(test)]`**（内联单测被占位/unwrap 扫描拦下 ⇒ 白跑一轮 gate） | 靠门禁第 5/6 项事后拦 | EXP-052（连踩 2 次） | 门禁断言（存量豁免棘轮） | `doccheck2::check_no_inline_test_mod` + `process_ratchet.json` | 已自动化 |
| 18 | **出图逐屏目视核对**（像素证据已自动化，结论仍须人看） | 人看缩略图 | S19.6/S19.7 判据 | 工具给证据 + 人核 | `scripts/ui_preview.ps1` + `ui_shot_probe.py` | 半自动（人核最后一步） |
| 19 | **模块边界清单留档**（拆上帝文件前的函数清单） | 手工列，不留档 | EXP-112（2181 行拆分编译 10 次） | 扩展 `code_outline.py` 出可留档清单 | `scripts/code_outline.py` | 半自动（清单未留档接口） |
| 20 | **设计分歧裁决 / 需人裁决项**（飞轮审计桶） | 人工裁决后手写 EXP | 06_flywheel §3 | 飞轮流程（人工裁决）+ `scripts/exp_log.py` 落 EXP | — | 半自动（裁决须人） |
| 21 | **每任务双审计**（A 完成度 / B 一致性，只读，连续 2 轮 0 新发现才可 verified） | 主线程手搓两份 prompt、报告散落、收敛轮次靠记忆 | S32.0 首轮即抓出 26 条 findings（虚报/口径缺失/依赖倒挂） | 平台生成审计任务书 + 归档报告 + 机械判定收敛轮次 | `docs/process/22_dev_platform.md` §7（S32.5 落码 `/api/audit`） | 半自动（子代理须主线程起 + 主线程独立复核） |

## 3. 收口一条命令（第 1 条的用法）

```powershell
# 跑判据 → 写 evidence/<任务号>.log → 置 verified → 同步 release.md 计数 → 三方对账
python scripts/task_close.py --task S21.4 --cmd "cargo test --workspace" --cmd "python scripts/validate_data.py"

# 只校验不写（dry-run）
python scripts/task_close.py --task S21.4 --check

# 已有证据文件，只翻台账 + 同步计数
python scripts/task_close.py --task S21.4 --no-run
```

**为什么要有它**：硬规则 2 要求「实跑输出写入 evidence 后才可置 verified」，但**三步都是手做**⇒
必然出现「先 verified 后补 evidence」「改了状态忘了同步 release.md 计数」「手改整份台账冲掉别人的行」。
本工具把三步收成一步，且**行级改写**（只动目标行），并自带 `check_ledger_reality.py` 复验。

## 4. 新增流程怎么进本页（纪律）

1. 发现「又做了一遍同样的手工活」⇒ 先查本页 §2 是否已有该流程；
2. 没有 ⇒ **要么**补工具（`scripts/` + 3 处登记 + 契约）并把状态改 `已自动化`，
   **要么**登记为 `待固化` + **台账任务号**（不许无任务号长期挂账）；
3. 一次性动作 ⇒ 标 `一次性` + 说明为何一次性（不许假装是流程）。

## 5. 与其它文档的关系

- 工具本身（用途/用法/幂等/退出码）= [`14_tooling_catalog.md`](14_tooling_catalog.md)（工具唯一事实源）；
- 纪律原文（分工/变更面/设计稿先行）= [`00_index.md`](00_index.md) + `AGENTS.md` 硬规则；
- 飞轮 3 角色与停机判据 = [`06_flywheel.md`](06_flywheel.md)（§8 = 定向飞轮变体）；
- **本页 = 「哪些纪律已被机器兜住」的进度面**（不是纪律原文，是纪律的执行状态）。
