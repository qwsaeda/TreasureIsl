# S32.2 第 1 轮审计 · B（一致性与风险）

```
总判定: PASS
新发现: 4
误报: 0
残留: 1
```

> 落盘说明：r1 报告主线程补写件（r2-A P2-2 纪律面缺件，由 r1 B 侧审计子代理会话内报告
> 全量落盘；只读，实跑 11 条机器判据 + 本机 8 端点 + 越界 5 面；未跑 gate.ps1 全量。
> 并行会话在途面（i18n 孤儿 key / android 壳 / maps manifest）逐条剥离归属。）

## 一致性面 a~h（r1 时点）

a) 22 号 §4.1 HTTP 定案 vs routes.rs：**PASS**（纯 std TcpListener + thread::scope 每连接一线程；
   0 tokio/axum/async）
b) 22 号 §5 数据源表 9 行 vs console_sources.json：**PASS（集合 1:1；r1 时点 JSON = 9 条）**
c) 22 号 §10.2 三条 vs 实现：**FAIL = NEW-1 P1**（「assets/ 只返回登记过的文件，不做任意
   目录列举」未落实：guarded_file 静态回退走 SOURCE_DIRS 5 目录常量 ⇒ `/assets/data/board.json`
   200 放行；实测同 §5 未登记文件面 = 越登记面）
d) 22 号 §8 冻结路径 9 项 vs FROZEN_PATHS：**PASS**（逐字一致；git 调用面 = ls-files --others，
   0 porcelain，禁用口径落实）
e) 22 号 §11.2 L360 信任项备注 vs 现状：**PASS（挂账口径）**（「S32.2 落码后由 ti-console 单测
   兜底，届时改『已断言』」= NEW-4 收口回写挂账）
f) CS-1 口径（0 字面量不扩豁免）：**PASS**（check_code_standards PASS 0 超基线；
   process_ratchet.json cs1_asset_grandfathered = {} 0 条目）
g) routes 静态回退面 vs 登记面：**FAIL = NEW-2 P2**（`/plan/packages.yaml` 200 放行；
   §5 表 plan 面 = 单文件 `plan/ledger.yaml`，packages.yaml 未登记）
h) main.rs repo_root 假设：**NEW-3 P2**（CARGO_MANIFEST_DIR 上两级 = 隐含「crate 在 crates/ 下」
   目录结构假设，0 自检兜底）

## 新发现（4 条：1 P1 + 3 P2；主线程同批修复记录见 S32.2.log §五）

| ID | 严重度 | 问题 | 修复落点（主线程同批） |
|---|---|---|---|
| NEW-1 | P1 | §10.2 第 3 条「assets/ 只返回登记过的文件」未落实（guarded_file 5 目录常量放行任意 assets 文件） | 已修：guarded_file 改 console_sources.json 混合白名单数据驱动（目录项前缀 + 文件项精确）+ SOURCE_DIRS 常量删除 + JSON 扩 assets/ui/ 项 |
| NEW-2 | P2 | `/plan/packages.yaml` 静态回退放行未登记文件 | 已修：plan 面 = 单文件精确项（与 NEW-1 同源修法） |
| NEW-3 | P2 | main.rs repo_root = CARGO_MANIFEST_DIR 上两级 0 自检 | 已修：启动自检（plan/ledger.yaml 存在性，缺失拒启 exit=1）+ 注释面标注隐含假设 |
| NEW-4 | P2 | 22 号 L360 信任项备注 + 15 号 L56 第 23 行「待固化」的收口回写未做 | 收口挂账（主线程随 task_close 回写 22 号 L360「已断言」+ 15 号 L56 转「半自动」） |

## 实跑验证（r1 时点，11 条全绿 + 越界负例）

build EXIT=0 / test 23/23 / clippy 0 warning / cargo tree 0 游戏 crate / ledger 0 error /
doccheck 0 error / doccheck2 0 error（38 项；i18n 孤儿 WARN 归并行线）/ CS PASS /
三方对账全绿 / doc_numbers 0 漂移 / 本机 8 端点 + 越界 400/404 实测。

## 总判定

**FAIL（非收敛轮，0 阻断级）**：一致性面 6 PASS / 2 FAIL（NEW-1 P1 + NEW-2 P2，同源修法
已随主线程第 2 版吸收）+ 新发现 4 条（NEW-3 已修 / NEW-4 收口挂账）⇒ 第 2 轮复核
（r2 双侧 0/0/0 + r3 双侧 0/0/0 = 连续 2 轮收敛后主线程 §7.5 独立复核收口）。

> 并行会话在途面（0 计入 S32.2 新发现）：android/*（S18.8d）/ assets/maps manifest 哈希
> （S32.12 线）/ i18n 孤儿 key（S8.x 数据化线）/ 根 Cargo.toml S18.8d 注释段。
