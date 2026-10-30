# S32.1 第 6 轮审计 · A（完成度）

```
总判定: PASS
新发现: 0
误报: 0
残留: 0
```

> 落盘说明：第 6 轮 A 侧审计落盘凭证（会话内报告全量落盘；B 侧同轮 = `audit-S32.1-r6-b.md`，
> 0 新发现）。只读；实跑 14 条机器判据 + 本机 ti-console 全负例（含 /api/health 200 +
> 400/404 + 非回环拒启 + 单实例锁二次启动）；未跑 gate.ps1 全量（等价机器判据口径）。

## DoD 逐条（S32.1，9 判据面）

1 cargo build -p ti-console 绿：**PASS**（实跑 exit=0）
2 cargo test -p ti-devmeta（8/8）与 -p ti-console（0 用例 PASS）绿：**PASS**
   （补测 ti-tools 0 用例 ok = 薄 re-export 不破）
3 clippy 0 warning：**PASS**（ti-devmeta + ti-console --all-targets 0 warning）
4 /api/health 200 + 默认 127.0.0.1 实测：**PASS**（实跑 `--port 18799`：/api/health = 200
   body `ti-console ok`；`/` = 200；`/../index.html` 穿越 = 400；缺文件 = 404；
   DEFAULT_BIND = 127.0.0.1:8787（main.rs L18 + http.rs L98 双处一致））
5 依赖图断言：**PASS**（cargo tree = serde_json + ti-devmeta；grep 7 项游戏 crate 0 命中）
6 鉴权负例：**PASS**（非回环无 token 拒启 = 本轮重实跑 exit=1；单实例锁二次启动拒起 =
   本轮重实跑 exit=1；无 token 写操作 403 = 骨架期 0 写端点 ⇒ S32.4 POST 面，log 如实标注）
7 工具登记三处：**PASS**（14 号 §3.5 两行〔ti-console L246 / ti-devmeta L247〕+ §1.1 五类
   表行 16 + §1.2 第 1 步 + §1.4 第 1/2/4 条（含第 4 条 cargo bin 豁免注记 L52）/
   04 号 §1 +2 行（L19/L20）/ README +2 行（L121/L122）；console_commands.json +
   console_sources.json `json.load` 实跑 OK）
8 AGENTS.md 同步 + 全库 grep 旧口径 0 残留：**PASS**（L8 = 9 member / L9 9 crate / L35；
   全库 grep「7/8 workspace member / 7/8 开发 crate」0 命中；
   `docs/tutorial/10_project_architecture.md:114`「共 6 个」= 入门教程历史快照
   （自带「以根 Cargo.toml members 为准」自校验注记）⇒ 归维护线，不算 S32.1；
   `docs/release.md:16`「6 个开发 crate」= HEAD 存量历史口径（oracle-rust 删除 S15.8 起
   未回填），非 S32.1 在途改动面，不计数）
9 gate 等价机器判据 + 双审计在途 + evidence：**PASS（在途）**（ledger 262/244/0 error +
   doccheck 0 error + doccheck2 38 项 0 error + CS PASS + 三方对账全绿 + doc_numbers 0 漂移；
   双审计 = r6 本轮 0/0/0，连续 2 轮 = r6 + r7 裁定；evidence/S32.1.log 在位
   （158 行，收口前实跑凭证 + 判据 1~10 全列 + 0 游戏产物变更面））

## r5 B 侧 2 条 P2 闭环：2/2 CLOSED

- P2-1（22 号 §5 行 7 拆 2 行）CLOSED：§5 表现 = 9 行（L138~L146：`assets/maps/<id>/`
  L144 + `assets/maps/.default` L145 各 1 行），与 console_sources.json 9 条 repo_sources
  逐条 1:1（含 gate_data.json L5）；§5 注释（L148「单一事实源 = console_sources.json」）自洽
- P2-2（15 号 L54 合计 215 漂移）CLOSED：L54 现值 = 「合计 199」，与逐份实加口径
  （170+5+4+21 = 200，L54 取 199 = 截至 r4 末实加 − r4-B N4-1 自计数 1）0 漂移；
  两种口径均接受，报实测值 = 199

## 新发现

**0 新发现**（P0×0 / P1×0 / P2×0）。并行会话在途面（i18n 孤儿 key WARN ×2 = S8.x 数据化线
（r5 时 ×4 已收窄，并行线在途）/ 根 Cargo.toml S18.8d 注释段 / android + ti-app + ti-platform
在途改动 / 工作树 175 行 git status 面）均剥离归属，非 S32.1 新发现。

## 总判定

**PASS（收敛轮）**：0 新发现 + 0 误报 + 0 残留，DoD 9/9（判据 9 = 双审计在途，
r6 = 本轮 0/0/0）；r5 2 条 P2 全 CLOSED；第 7 轮（r7）再 0/0/0 ⇒ 连续 2 轮达标，
主线程独立复核收口。
