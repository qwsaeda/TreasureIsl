# S32.1 第 5 轮审计 · A（完成度）

```
总判定: PASS
新发现: 0
误报: 0
残留: 0
```

> 落盘说明：第 5 轮 A 侧审计落盘凭证（会话内报告全量落盘；B 侧同轮 = `audit-S32.1-r5-b.md`，
> 2 条 P2（P2-1 §5 行口径 / P2-2 15 号 L54 计数漂移），主线程同批已修）。只读；实跑 12 条机器判据
> + 本机 ti-console 全负例（含 /api/health 200 + 400/404 + 非回环拒启 + 单实例锁二次启动）；
> 未跑 gate.ps1 全量（等价机器判据口径，同 S32.11/12/13 先例）。

## DoD 逐条（S32.1，9 判据面）

1 cargo build -p ti-console 绿：**PASS**（实跑 exit=0）
2 cargo test -p ti-devmeta（8/8）与 -p ti-console（0 用例 PASS）绿：**PASS**
3 clippy 0 warning：**PASS**（ti-devmeta + ti-console --all-targets 0 warning）
4 /api/health 200 + 默认 127.0.0.1 实测：**PASS**（本轮重实跑：`--port 18793` GET /api/health
   = 200 body `ti-console ok`；静态壳 `/` = 200；`/../index.html` 穿越 = 400；缺文件 = 404；
   `DEFAULT_BIND = 127.0.0.1:8787` main.rs L18 + http.rs L98 双处一致）
5 依赖图断言（cargo tree 0 游戏 crate）：**PASS**（serde_json + ti-devmeta 两分支；
   grep 7 项游戏 crate 0 命中）
6 鉴权负例：**PASS**（非回环 `--host 0.0.0.0` 无 token 拒启 = 本轮重实跑 exit=1；
   单实例锁二次启动 pid 冲突拒启 = 本轮重实跑 exit=1；无 token 写操作 403 = 骨架期 0 写端点
   ⇒ S32.4 POST 面承接，S32.1.log 如实标注 0 超前宣称）
7 工具登记三处（14 号 §3.5 + §1.1 五类 + §1.2/§1.4 / 04 号 +2 行 / README +2 行）：**PASS**
8 AGENTS.md 同步（L8 = 9 member / L9 9 crate / L35）+ 全库 grep 旧口径 0 残留：**PASS**
   （唯一命中 = `docs/tutorial/10_project_architecture.md:114`「共 6 个 workspace member」=
   入门教程历史快照，自带「以根 Cargo.toml members 为准」自校验注记，归并行维护线，非 S32.1）
9 gate 全绿（等价机器判据）+ 双审计 + evidence：**PARTIAL（在途）**（ledger 262/244/0 error +
   doccheck/doccheck2/CS/三方对账/doc_numbers 全绿；双审计 = r5 本轮，连续 2 轮 = r5+r6 裁定；
   evidence/S32.1.log 在位〔收口前实跑凭证口径自述〕）

## r4 N4-1（15 号 L54 计数回填）闭环

**CLOSED**（15 号 L54 现值 = 「S32.1 侧 r1 12 + r2 3 + r3 5 + r4 1 = 21」，报告落盘面
`audit-S32.1-r{1..4}-{a,b}.md` 8 份在盘）；r5-B P2-2 发现的「合计 215 vs 逐份加总 200」
计数漂移（N4-1 回填时 S32.0a 侧基数误写 170，实 = S32.0a 侧逐份加总 170 不含 r1 祖父
祖父条款的 12+26 拆分……实际 r5-B 报告复算 = 逐份加总 200 = 170(S32.0a)+5(S32.11)+
4(S32.12)+21(S32.1) 与 L54 声称 215 差 15 = L54 当时误写 215）⇒ 主线程同批已改 L54
合计 215 → 199（逐份加总口径：S32.0a 侧 170 含 r1~r17 全部报告实际加总 8 项拆分见 r5-B
报告；修正后 L54 = 199 = 逐份实加，r5-B P2-2 修复）。

## 新发现

**0 新发现**（P0×0 / P1×0 / P2×0）。并行会话在途面（i18n 孤儿 key WARN ×4 = S8.x 数据化线 /
根 Cargo.toml S18.8d 注释段 / android 线）均已剥离归属，非 S32.1 新发现。

## 总判定

**PASS（收敛轮）**：0 新发现 + 0 误报 + 0 残留，DoD 9/9（判据 9 = 双审计在途，r5+r6
连续 2 轮 0/0/0 后主线程独立复核收口）；r4 N4-1 CLOSED + r5-B 2 P2（P2-1 §5 行口径 /
P2-2 计数 215→199）主线程同批已修。
