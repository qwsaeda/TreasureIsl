---
name: gate-green
description: Use when the user asks to make gate.ps1 all green, diagnose gate failures, or confirm a gate FAIL is from their own change vs a parallel in-progress task. Triggers: "gate 全绿", "gate 失败", "哪几项 FAIL", "FAIL 归因", "半成品", "跑一下门禁".
---

# gate.ps1 全绿 + 失败归因

## 跑 gate

```powershell
powershell -ExecutionPolicy Bypass -File scripts/gate.ps1
```

当前 13 项（见 `docs/parity.md`）。`SKIP` 项（android target / zero-unknown verify 未落地）不算失败。

## 失败归因（关键）

**不要看到 FAIL 就盲目改代码。** 先判断这个 FAIL 是不是本次改动引入的：

1. 跑 `git status --short` 看工作区还有哪些文件被改过（含并行会话未提交的半成品）。
2. 对每个 FAIL 项，定位到具体报错文件/行：
   - `cargo fmt --check` FAIL → 看哪个文件没格式化。若是本次改的文件，跑 `cargo fmt`；若是并行会话的半成品（如 `scenes/mod.rs` 刚加了 trait 没格式化），**不要替它格式化提交**，标注「归因于并行会话 S9.x，非本次改动」。
   - `cargo clippy -D warnings` FAIL → 同上，看 clippy 报错是否在本次改动文件里。
   - `cargo test` / `cargo build` FAIL → 看编译错误源文件。
3. 判定标准：**本次改动文件（`git diff --name-only`）里有没有 clippy/fmt/build 报错**。有 → 本次负责修；没有 → 归因于其他会话，记录后停下。

## 归因后

- 本次负责的 FAIL 修掉，重跑 gate 直到「本次改动文件 0 报错」。
- 其他会话的半成品 FAIL，在最终回复里明确列出（文件名 + 报错摘要 + 哪个 S9.x 任务引入），**不替它修、不替它提交**。
- gate 全绿（本次域内）+ `doccheck2` 0 error 才算收口。

## 常见「假失败」

| 现象 | 真实原因 | 处置 |
|---|---|---|
| `cargo clippy` 报 50+ 个 doc 缺失 | 并行会话加了新 trait/struct 没写 doc 注释 | 归因并行会话，不动 |
| `cargo fmt` 报某行 | 该文件被并行会话改过 | 归因，或仅对本次改动文件跑 `cargo fmt` |
| gate 第 7 项 `zero-unknown verify` SKIP | `verify.rs` 尚未落地（S1.x） | 既定 SKIP，不算失败 |
| gate android 构建 SKIP | 本机无 NDK/SDK，APK 在 Mac 编 | 既定 SKIP |
