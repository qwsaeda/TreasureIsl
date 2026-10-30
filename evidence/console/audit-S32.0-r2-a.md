# S32.0 第 2 轮审计 · A/B 合并（完成度 + 一致性）

> 落盘说明：本文件为**第 2 轮审计 findings 的落盘凭证**（会话内审计报告的结构化汇总）。
> 审计方式：只读，不改任何文件。因子代理额度限制，第 2 轮 A/B **合并为一次审计**（同一份报告覆盖
> 「完成度」与「一致性」两个视角），第 3 轮起恢复 A/B 双报告并行。
> 审计对象：S32.0 第 2 版（22 号文档 + S32 WBS + 台账 S32 段）。

## 总判定（第 2 轮）

**FAIL** —— P0×0 / P1×4 / P2×12，共 16 条 findings。
第 3 轮复核结论：**16/16 全部 CLOSED**（第 3 轮两份报告均逐条给出行号证据）。

## P1 findings（4 条）

| ID | 问题 | 修复落点（第 3 版已落） |
|---|---|---|
| P1-1 | §4.2 白名单 `map_pack.py` 参数契约不可执行（实际 = `verify <path>` 位置参数 + `check` 无参，且「只接受枚举/id」与自由路径自相矛盾） | 22 号 §4.2 改为「已装包 id 枚举 → 平台内部拼 path」 |
| P1-2 | 写面边界漏列 `build_wasm.ps1` 的跟踪文件写点（`evidence/S18.8a-wasm-size.log`），且 §8 冻结清单不覆盖 `evidence/` | 22 号 §3.1 + §9：平台**不执行完整构建**，只跑 `-Check` |
| P1-3 | §11.2「门禁自动覆盖」与 §12 CS-1 冲突（CS-1 允许位不含平台读点 ⇒ 平台必然 FAIL；且只同步文档不改门禁脚本） | 22 号 §5 + §11.2：路径集中 `console_sources.json` ⇒ Rust 侧 0 资产字面量 ⇒ CS-1 天然 0 命中、**不扩豁免** |
| P1-4 | 引用了无落盘凭证的审计结论（15 号台账写「26 条 findings」但 `evidence/console/` 为空） | 落盘 r1 报告 + 台账 `rounds: 1` |

## P2 findings（12 条）

| ID | 问题 | 修复落点 |
|---|---|---|
| P2-1 | `cargo run -p ti-tools --bin serve` 会编译整条游戏栈，与零依赖精神相悖 | §4.2 改用平台自身静态服务 |
| P2-2 | token 落 `evidence/console/token`（git 跟踪目录） | 落仓库外 + `.gitignore` 兜底（后续第 3 轮又补了 Windows 路径与取得途径） |
| P2-3 | 浏览器如何取得 token 未定义 | §10.1 token 分发口径（sessionStorage + `X-TI-Token`） |
| P2-4 | S32.9「`web/` sha256 前后一致」与「实跑 build_wasm」冲突 | S32.9 改只读体检（`-Check`） |
| P2-5 | §7.4 与飞轮 §4 收敛判据不等价 | §7.4 改用飞轮三条 + 连续 2 轮 |
| P2-6 | `console_commands.json` 落点未定义 | §4.2 + §12 固定为 `crates/ti-console/console_commands.json` |
| P2-7 | `plan/packages.yaml`（嵌套 YAML）与「纯 std + serde_json」不兼容 | §9 不结构化解析，只给链接 |
| P2-8 | `git diff --exit-code` 漏检已 staged 改动 | §8 改 `git diff HEAD`（第 3 轮再升级为增量口径） |
| P2-9 | 22 号引用 `ti-tools/src/...` 缺 `crates/` 前缀 | 路径补前缀（第 3 轮进一步改为显式枚举冻结路径） |
| P2-10 | 平台 UI 视觉签收无机器兜底口径未写 | §11.1 明确「人工留痕（`check_visual_design_signoff` 只覆盖 scenes/theme）」 |
| P2-11 | 台账标题「4 处登记」与 DoD 实列 5 处不符 | 标题改「5 处登记」 |
| P2-12 | §4.1 称 `serve.rs` 已证明「静态 + JSON 路由 + 轮询」可行（实际只做了静态文件 + 串行 accept） | §4.1 改为「已实测证明 accept 循环 + 静态响应；JSON 路由/POST/轮询为新增面」 |
