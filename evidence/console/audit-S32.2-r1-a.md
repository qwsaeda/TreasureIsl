# S32.2 第 1 轮审计 · A（完成度）

```
总判定: PASS
新发现: 3
误报: 0
残留: 0
```

> 落盘说明：r1 报告主线程补写件（r2-A P2-2 纪律面缺件，由 r1 审计子代理会话内报告
> 全量落盘；r1 A 侧 = 3 条新发现 〔1 P1 + 2 P2〕，r1 B 侧 = 4 条 〔1 P1 + 3 P2〕，
> 见 `audit-S32.2-r1-b.md`。只读；实跑 12 条机器判据 + 本机 ti-console 全端点负例；
> 未跑 gate.ps1 全量（等价机器判据口径，同 S32.1/S32.11/S32.12 先例）。

## DoD 逐条（S32.2，台账 L310，10 判据面）

1 台账任务数/资产数/已装包数与源文件抽样一致：**PASS**（/api/overview = 262 任务 245 verified
   next_task=S7.7 + 已装包 4 + UI 资产 75 + 数据源 9/11 条；与 `--bin ledger` + ui_manifest +
   assets/maps 实扫 0 漂移）
2 越界路径（..、符号链接）400/404：**PASS**（`--path-as-is` 穿越 400 / 缺失 404 / 白名单外 404
   / 符号链接逃逸 unix 实跑 404；**§10.2「assets/ 只返回登记过的文件」未落实 = P1（NEW-A-1）**）
3 0 写盘：**PASS**（repo/ 全模块 0 写盘 API；唯一写点 = evidence/console/.lock）
4 单测覆盖解析与路径边界：**PASS**（r1 时点 23 条；r1 修复同批 +1 = 24 条）
5 console_sources.json 单一事实源 + 0 assets 字面量（CS-1）：**PASS**（pick_source 数据驱动 +
   CS-1 0 新增违例；**NEW-A-1 的 P1 侧 = CS-1 正则面（.json 后缀）与「0 assets 路径字面量」
   表述边界不精确 = P1 口径面**）
6 不结构化解析 packages.yaml：**PASS**（全 crate 0 YAML 解析器；仅注释/测试断言提及）
7 gate 等价机器判据：**PASS**（九项全绿；红项 = i18n 孤儿 key ×N WARN 归并行 S8.x 数据化线）
8 依赖图断言保持：**PASS**（cargo tree 0 游戏 crate）
9 双审计（r1 本轮）：PASS（本轮落盘；收敛 = 连续 2 轮 0/0/0）
10 evidence/S32.2.log：PASS（收口前实跑凭证口径）

## 实现面逐文件对应（0 越界）

- `repo/config.rs`（console_sources.json 解析，RepoSources + Default）；
- `repo/guard.rs`（白名单 + canonicalize 防穿越 + 7 条单测）；
- `repo/data.rs`（overview/tasks/evidence/assets/session 5 端点 + 8 条单测；
  **P1 面 = guarded_file 静态回退走 5 目录常量放行未登记 assets 文件**）；
- `repo/snapshot.rs`（§8 载体：冻结路径 9 项 + 内容指纹 + 未跟踪清单 + 6 条单测；
  FNV-1a 双轮折叠非密码学 sha256 = 已知限制 1 如实标注）；
- `routes.rs`（8 端点 + 静态回退 + 400/404 语义）；
- `main.rs`（repo_root 上两级 = 隐含目录假设 = NEW-3 P2）。

## 新发现（3 条：1 P1 + 2 P2；主线程同批修复记录见 S32.2.log §五）

| ID | 严重度 | 问题 | 修复落点 |
|---|---|---|---|
| NEW-A-1 | P1 | guarded_file 静态回退走 `SOURCE_DIRS` 5 目录常量 ⇒ 未登记 assets 文件（board.json 等）200 放行，违 22 号 §10.2 第 3 条「assets/ 只返回登记过的文件」 | 已修：改 console_sources.json 混合白名单数据驱动（目录项前缀 + 文件项精确）+ JSON 9→11 条（docs/ 目录项 + assets/ui/ + dist/maps/index.json）+ 22 号 §5 表同步 |
| NEW-A-2 | P2 | `sha256_hex` 命名误导（实 = FNV-1a 双轮折叠，非密码学 SHA-256） | 已修：函数顶注补「非密码学 SHA-256 + S32.4 升级须同步 22 号 §8 口径」 |
| NEW-A-3 | P2 | `/api/snapshot` 指纹抽样 = BTreeMap 字典序前 8（8 冻结路径不全覆盖） | 挂 S32.3/S32.4 消费面定案（0 游戏产物变更判定靠 diff 全量面，抽样面仅防体积） |

## 总判定

**FAIL（非收敛轮，0 阻断级）**：DoD 10/10 产物面 PASS；1 P1（NEW-A-1 §10.2 越界面）+
2 P2（NEW-A-2/3）⇒ 主线程同批修复后第 2 轮复核（r2 双侧 0/0/0 + r3 双侧 0/0/0 = 连续 2 轮收敛）。

> 并行会话在途面（0 计入本任务新发现）：i18n 孤儿 key WARN（S8.x 数据化线）/
> android/* + assets/maps manifest 脏面（S18.8d + S32.12 线）/ doccheck2 S2 勘误（旁支）。
