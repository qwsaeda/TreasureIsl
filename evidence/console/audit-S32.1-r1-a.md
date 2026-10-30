# S32.1 第 1 轮审计 · A（完成度）

```
总判定: FAIL
新发现: 7
误报: 0
残留: 1
```

> 落盘说明：第 1 轮 A 侧审计落盘凭证（会话内报告全量落盘；B 侧同轮 = `audit-S32.1-r1-b.md`，
> 5 条 P2 建议，同源同批）。只读；实跑 build/test/clippy/cargo tree/ledger/doccheck/doccheck2 +
> 本机 ti-console 实跑（/api/health 200 + 鉴权负例 + 单实例锁）；未跑 gate.ps1 全量。

## DoD 逐条判定（S32.1，台账 L309，9 判据面）

1 cargo build -p ti-console 绿：**PASS**（实跑 exit=0）
2 cargo test -p ti-devmeta / -p ti-console 绿：**PASS**（ti-devmeta 8/8 过 = 上提行为对齐；
   ti-console 0 用例 PASS；ti-tools --all-targets 全绿 = 薄 re-export 0 破坏）
3 clippy 0 warning：**PASS**（ti-devmeta + ti-console 0 warning，workspace lints 含 pedantic）
4 /api/health 200 + 默认 127.0.0.1 实测：**PASS**（本机实跑 200 + body `ti-console ok`；
   静态壳 index 200 / 缺文件 404）
5 依赖图断言：**PASS**（`cargo tree -p ti-console` = serde_json + ti-devmeta，
   grep 游戏 crate 7 项 0 命中）
6 鉴权负例：**PASS**（非回环无 token 拒启 exit=1 实测；配 token 后非回环可绑实测；
   单实例锁二次启动拒起实测；「无 token 写操作 403」= 骨架期由 main.rs host/token 检查承接
   + S32.4 POST 面（log L95-97 如实标注，0 超前宣称））
7 工具登记三处：**PASS**（14 号 §3.5 + §1.1 五类 + §1.2/§1.4 同步 / 04 号 §1 +2 行 /
   README 工具索引 +2 行，实查在位）
8 AGENTS.md 路由表与 crate 清单同步：**PASS**（L8 = 9 workspace member / L9 crate 清单 9 项 /
   L35 路由表 S32 行补 crate 归属；全库 grep「7/8 workspace member」旧口径 0 残留）
9 gate 全绿 + 双审计 + evidence：**PARTIAL**（gate 等价机器判据实跑全绿 = log §判据 9；
   双审计面 = 本轮 r1 报告（本份 + r1-b）落盘，r2 起第 2 轮；残留 1 = 双审计连续 2 轮 0/0/0
   未达成，verified 前置条件未满足，属流程在途非实现缺陷）

## 实现面逐文件对应（0 越界）

- `ti-devmeta/src/ledger.rs` = 原 `ti-tools/src/ledger.rs` 逐字节上提（`git show HEAD:... | diff` = 0 差异）；
- `http.rs` = `serve.rs` 原型行为对齐（resolve_or_404 逐段同构 + 每连接一线程库化，serve.rs 本体 0 改动保留）；
- `main.rs`（默认 127.0.0.1:8787 + 非回环 token 拒启 + 单实例锁 pid 内容 + Ctrl+C 兜底标注 S32.4）；
- `routes.rs`（/api/health 200 + 静态壳映射，复用 ti-devmeta::http）；
- `console_commands.json` 9 条 ↔ 22 号 §4.2 人读视图 9 行逐条一致（id 集合 + 默认启用列 + 写面）；
- `console_sources.json` ↔ 22 号 §5 数据源表逐条一致（P2-1 修复后 gate_data.json 补全 = 8 路径）；
- 根 Cargo.toml members 仅 +2 行（S32.1 面；git diff 混并行会话 S18.8d/e 在途，已剥离归属）。

## 新发现（7 条 P2，0 P0/P1；P2-04/P2-05 归并行会话在途）

| ID | 严重度 | 问题 | 归属/修复落点 |
|---|---|---|---|
| P2-01 | P2 | 单实例锁 Ctrl+C 终止场景 Drop 不执行 + pid 存活检测未实现（main.rs L130 注释已标注「S32.4 补全」，0 虚报） | S32.4 补 `kill(pid,0)` 存活探测 + 30 分钟 stale-lock 判定（22 号 §10.4） |
| P2-02 | P2 | POST 写操作骨架期走静态映射（非 403）；「无 token 写操作 403」由 main.rs host 检查兜底 + S32.4 POST 面承接（已如实标注） | S32.4 补 POST 方法识别 + 403 语义（22 号 §10.1） |
| P2-03 | P2 | console_commands.json 缺 8 条白名单的「参数枚举」字段（仅 gate 有 enum_param），S32.6 对账若升级参数面将漂移 | S32.6 对账断言（id 集合 + 默认启用列）先行；参数枚举字段扩面随 S32.6/S32.7 定 |
| P2-04 | P2（并行） | 根 Cargo.toml git diff 混 S18.8d（android 注释块）+ S18.8e（LTO/profile）在途改动 | 归 S18.8d/S18.8e 收口会话，非 S32.1 引入（S32.1 冻结路径 0 越界已实测） |
| P2-05 | P2（并行） | 4 份文档 + ti-tools 6 文件 + .gitignore 的 git diff 混 S16.4/S2.6/S18.8d 并行在途（14 号 31→33 项同步 / doccheck2 S2 勘误 / .gitignore node_modules 行等） | 归并行会话，非 S32.1 引入（S32.1 判据面 14 号 §3.5/§1.1/§1.2/§1.4 + 04 号 2 行 + README 2 行 + AGENTS L8/L9/L35 逐项落盘 0 越界） |
| P2-06 | P2 | evidence/S32.1.log 判据 9 实跑（ledger 244 verified 等）= 收口前时点值；若收口前并行会话推进台账须重跑回填 | 主线程收口复核面（0 虚报：复跑 244 与 log 一致） |
| P2-07 | P2 | web/index.html L16 占位 `<a href="#">` 链接（非真实路由） | S32.2 只读 API 落地后接真实路径或移除 |

## 总判定

**FAIL（非收敛轮）**：DoD 9 判据面 8 PASS + 判据 9 双审计面 = 本轮 r1 落盘（残留 1 = 连续 2 轮
0/0/0 未达成，流程在途）；实现面 0 P0/P1，7 条 P2 建议（P2-04/05 归并行会话；P2-01/02/03/07
挂 S32.2/S32.4/S32.6 后续面，0 阻断 S32.1 收口；P2-06 主线程收口复核）。
P2-01~03/05/07 的文档面修复（22 号 §10.4 锁规格在途标注 + 14 号 §1.4 第 4 条 cargo bin 豁免注记 +
22 号 §11.2 信任项备注改挂 S32.2 + console_sources.json 补 gate_data.json）已随第 2 版文档吸收，
第 2 轮复核。
