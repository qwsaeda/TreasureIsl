# S32.2 第 2 轮审计 · A（完成度）

```
总判定: FAIL
新发现: 2
误报: 0
残留: 7
```

> 落盘说明：第 2 轮 A 侧审计落盘凭证（B 侧同轮 = `audit-S32.2-r2-b.md`，3 条 P2）。
> 只读；实跑 12 条机器判据 + 本机 ti-console 全端点 + 越界 5 面；未跑 gate.ps1 全量。
> ⚠ 环境注记：审计起始时 /disk/target/debug/ti-console 为旧二进制（不含 r1 修复面），
> 工作区源文件被并行会话刷新后重新 build = 全部判据面以新二进制为准；
> 旧二进制中间态（/api/whitelist 404）归环境时序，不计 finding。

## DoD 逐条（S32.2，台账 L310，10 判据面）

1 台账任务数/资产数/已装包数与源文件抽样一致：**PASS**（/api/overview = 262 任务 245 verified
   next_task=S7.7 / 已装包 4 / UI 资产 75 / 数据源面；与 `--bin ledger` 262/245 + ui_manifest 75
   + assets/maps 4 包 + .default m01 + dist 3 件 0 漂移）
2 越界路径 400/404 + 未登记文件 404（§10.2 修复面）：**PASS**（`--path-as-is` 穿越 400 /
   未登记 `/assets/data/board.json` 404 / `/plan/packages.yaml` 404 / 登记面 200（含
   assets/ui/ 目录项 + 文件项精确）/ 符号链接逃逸 canonicalize 兜底 404）
3 0 写盘：**PASS**（repo/ 全模块 0 写盘 API；冻结树 find 断言 0 新文件；唯一写点
   evidence/console/.lock）
4 单测覆盖解析与路径边界：**PASS**（实测 24 条；log 判据 4 段记载 23 = r1 修复前计数，
   修复同批 +1 条混合白名单单测 = 24，0 矛盾）
5 console_sources.json 单一事实源 + CS-1 0 新增：**PASS（数据驱动面成立；**
   **NEW-1 P2 = JSON 11 条 vs §5 表 9 行口径漂移 + `dist/maps/index.json` 登记不存在文件**）**
6 不结构化解析 packages.yaml：**PASS**（0 YAML 解析器；仅注释/测试断言提及）
7 gate 等价机器判据：**PASS**（九项全绿；i18n 孤儿 WARN 归并行线）
8 依赖图断言保持：**PASS**（0 游戏 crate）
9 双审计（r2 本轮）：PASS（本轮落盘；收敛 = 连续 2 轮 0/0/0，r2 非收敛轮）
10 evidence/S32.2.log：PASS（§五 r1 findings 吸收记录在位）

## r1 两侧 7 条 findings 闭环

- NEW-A-1/NEW-1（P1 §10.2 assets 越界）：**CLOSED**（guarded_file 数据驱动混合白名单 +
  SOURCE_DIRS 常量删除 + JSON 扩 11 条；实跑未登记 404）
- NEW-2（plan/packages.yaml）：**CLOSED**（plan 单文件精确项，404）
- NEW-3（repo_root 假设）：**CLOSED**（main.rs L115~130 启动自检 + 注释面标注）
- NEW-A-2（sha256_hex 命名）：**CLOSED**（顶注「非密码学 SHA-256」在位）
- NEW-A-3（抽样面）：**CLOSED（挂 S32.3/S32.4 挂账在位）**
- NEW-4（22 号 L360 + 15 号 L56 回写）：**CLOSED（收口挂账，0 回潮 = 预期态）**

## 新发现（2 条 P2，均 1 行级文档/数据修订，主线程同批已修）

| ID | 严重度 | 问题 | 修复落点（主线程第 3 版） |
|---|---|---|---|
| P2-1 | P2 | `console_sources.json` 第 11 条 `dist/maps/index.json` = 登记指向不存在文件（dist/maps/ 实盘 6 个 .map-pack/.zip 无 index.json；`map_pack.py index` 写的是 `assets/maps/index.json`）+ JSON 11 条 vs 22 号 §5 表 9 行行数漂移（多出 docs/ 目录项 + assets/ui/ + index.json 3 项未入表） | 已修：JSON 删 index.json 条（11→10 条）+ 22 号 §5 表 9→10 行（+ assets/ui/ 行 + dist 行口径注「不登记 index.json 类文件项」） |
| P2-2 | P2 | `evidence/console/` 缺 r1 两份报告文件（§7.3 轮次完整性断言面 = r1×A/B 缺件） | 已修：主线程补写 `audit-S32.2-r1-{a,b}.md`（抬头标注「r1 报告主线程补写件」） |

## 总判定

**FAIL（非收敛轮，2 P2 已同批修，0 阻断级）**：DoD 10/10 产物面 PASS + r1 7/7 闭环；
新发现 2 P2（P2-1 口径漂移 + P2-2 缺件）⇒ 第 3 轮复核（r3 双侧 0/0/0 + r4 双侧 0/0/0
= 连续 2 轮收敛后主线程 §7.5 独立复核收口）。

> 并行会话在途面（0 计入 S32.2 新发现）：i18n 孤儿 key WARN（S8.x 数据化线）/
> android/* + maps manifest 脏面（S18.8d + S32.12 线）/ 台账 verified 计数 245→246
> 漂移（S32.1 verified 推进在途）/ 根 Cargo.toml S18.8d 注释段。
