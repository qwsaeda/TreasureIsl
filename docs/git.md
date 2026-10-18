# 推送规范（git push conventions）

远端：`origin` = `https://github.com/qwsaeda/TreasureIsl.git`，主干 **`master`**（本仓无 `main`；⛔ 不要 `git push -u origin main`，refspec 不匹配）。

## 分支模型

- 主干：`master`（长期分支，保持可构建、gate 全绿）。
- 任务分支：`task/<ID>`（如 `task/S0.2`、`task/S3.5`），每任务一支，完成后合入 `master` 并删除远端分支。
- **推送前必须**：`powershell -ExecutionPolicy Bypass -File scripts/gate.ps1` 全绿（含 doccheck 第 9 项）+ 该任务 `plan/ledger.yaml` 已置 `verified` 且 evidence 已写入。

## 推送规则

| 规则 | 说明 |
|---|---|
| 只推 `master` 与 `task/*` | 其他分支名（`feature/`、`fix/`、`hotfix/`、个人名等）⛔ 禁止 |
| 任务分支生命周期 | 合入 `master` 后 `git push origin --delete task/<ID>` 清理 |
| 禁止 force push | `master` 与任何 `task/*` 均禁止 `--force` / `+ref`；如需重写历史只在任务分支上且经确认 |
| 禁止推未验收 | gate 未全绿或台账未 `verified` 的任务不得推 `master`；任务分支可暂存 WIP 但不得合入 |
| 远端不可变 | 不得删除远端 `master`；不得修改远端分支保护配置 |
| 凭据 | 只用已配置的 `origin`（HTTPS token）；⛔ 不得把 token/凭据写进 commit、文档、脚本 |

## 提交信息规范

格式：`<scope>: <简述>`（中文简述），scope 取值：

| scope | 用途 |
|---|---|
| `spec` / `rules` / `impl` | 玩法逻辑模块（main_loop/render/menu/selectrole/selectmap/mainmap/trade/build/battle/cards/saveload/messages/rng） |
| `fmt` | 格式/行尾/rustfmt 等纯风格 |
| `docs` / `docs+台账` | 文档与 `plan/ledger.yaml` |
| `ci` / `scripts` | `scripts/`、gate、构建脚本 |
| `deps` | Cargo 依赖变更 |

示例（沿用库内既有风格）：

```
sprite_atlas: S6.4 多帧精灵动画（frame_h 整除 + blit_scene_frame 按帧 blit）
docs+台账: S6.4 多帧动画 verified + 文档同步（gamebook/modernization/release）
fmt: 全库规范 CRLF→LF 行尾（rustfmt newline_style=Unix）+ modernization S6.4 预算表同步
```

## 推送频率（完成即自动推送）

- **每完成一个任务合入 `master` 后自动推送**（无需用户逐次指令）：`git push -u origin master`（首次带 `-u`，后续 `git push` 即可）。
- 开发节奏：完成一个任务并 gate 全绿、台账 verified → commit → **自动推远端**，作为进度备份；不要攒多个任务才推一次。
- 网络不通时：本地照常 commit，记录「待推送」，网络恢复后补推（不阻塞本地开发，但不得长期积压不推）。

## 推送检查清单（每次推送前）

```
[ ] git status 无意外文件；data/ 二进制与 evidence/ 按 .gitignore 策略
[ ] powershell -ExecutionPolicy Bypass -File scripts/gate.ps1 全绿（23 项）
[ ] plan/ledger.yaml 目标任务 verified + evidence/<task>.log 已写
[ ] commit 消息符合 <scope>: 格式
[ ] 推送 `master`（本仓无 main）；任务分支合入后删远端 task/*
[ ] 无 force push、无凭据泄漏
```

## 与硬规则的关系

- 本规范是 `AGENTS.md` 硬规则 2（验收才算完成）/ 3（进度只由台账驱动）在远端推送侧的落地：未验收不推 `master`。
- 文档治理：本文登记进 `docs/README.md` 入口与 `AGENTS.md` 路由表（硬规则 6 单一事实源，推送约定只写此处）。
