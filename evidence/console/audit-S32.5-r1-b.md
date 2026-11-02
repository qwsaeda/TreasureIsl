# S32.5 双审计编排 · r1 · B 侧（一致性/风险审计）

> 只读审计：未改文件/台账，未跑构建。依赖树 clean（ti-console → ti-devmeta + flate2 + serde_json），27 audit 测试过，clippy clean（仅 ti-engine 无关 manifest 警告），硬规则 1 grep 0 命中，八要素单一事实源同步，防穿越防御。

```
总判定: FAIL
新发现: 4
误报: 0
残留: 0
```

## findings 表

| # | 严重度 | 位置 | 结论 |
|---|---|---|---|
| F1 | P1 | `crates/ti-console/src/main.rs:137-142` / `docs/process/22_dev_platform.md:325-326,336-341` / `evidence/S32.1.log:87-89` | 非回环+无 token 未按 22 号 §10.1 拒绝启动（回归 + 三方漂移 + 文档自相矛盾） |
| F2 | P2 | `crates/ti-console/src/main.rs:100-128` / `docs/process/22_dev_platform.md:325,330` | `--token` CLI 参数与 `token` 子命令未实现，被静默忽略 |
| F3 | P2 | `crates/ti-console/src/audit/archive.rs:41-42` | 归档写路径未防符号链接（`std::fs::write` 跟随已有 symlink） |
| F4 | P2 | `crates/ti-console/src/routes.rs:116-123` | >256KB POST body 返回 400 而非 413（与 22 号 §10.1 64KiB 上限 413 语义不符） |

## 判据逐条核对（摘要）

- 架构/crate 边界：`cargo tree -p ti-console` = ti-console → {flate2, serde_json, ti-devmeta}，不含游戏 crate。PASS。
- 单一事实源（八要素）：22 号:232-239 定义一处；template.rs:7-16 逐字同步。PASS。
- 硬规则 1：src/ 全量 grep TODO/unwrap/expect/panic = 0 命中。PASS。
- 回归（GET /api/audit 字段保留 + 新增纯追加）：data.rs:540-551 + 前端 ?? '—' 兼容。PASS。
- 安全（POST 鉴权 403/CT 400/body 413/路径白名单）：与 handle_gate 口径一致。PASS（F3/F4 除外）。
- 依赖/构建：cargo test + cargo clippy 全绿。PASS。

## 汇总

核心编排（模板八要素覆盖 + 机械判定 + 归档防穿越 + 收敛单测）功能正确、测试全绿、依赖解耦；但平台启动/token 安全面存在 1 项回归（F1，P1）与 3 项 P2（F2 --token 缺位 / F3 归档 symlink / F4 413 口径），需主线程复核后决定是否闭环再置 verified。
