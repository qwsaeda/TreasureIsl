# 对抗审计 B（一致性/风险）· S44.3 · r2

```
总判定: FAIL
新发现: 1
误报: 0
残留: 0
```

**结论**：r1 七条 findings（B-F1~F7）全部闭环；FAIL 系新发现 N1（P2，证据链缺口）。功能代码正确、单测覆盖、gate 全绿；唯一阻断 = 最终代码的签收强制路径未端到端演示（e2e 状态证据早于 F2 修复）。另有 P3 低severity 注记（不计新发现）。

## 一、r1 findings 闭环复核（7/7 已修）

| r1 finding | 严重度 | 复核 | 证据 |
|---|---|---|---|
| B-F1 | P1 | ✅ | step_write_paths("layout")→&[]；layout=git diff；layout_requested 已删；30 号 §2.2 step3 对齐 |
| B-F2 | P1 | ✅ | evaluate_criterion 强制 signoff；单测 2 项；30 号 §2.2 step6 对齐（注：e2e 证据早于修复 ⇒ N1） |
| B-F3 | P1 | ✅ | merge_params→Result；is_ident/is_viewport/assets/suffix/域；单测 5 项 |
| B-F4 | P2 | ✅ | handle_wizard_step content_type + CT 400 |
| B-F5 | P2 | ✅ | ui_variant.py --register 后 validate_data |
| B-F6 | P2 | ✅ | 实测 routes.rs 846 < 900 |
| B-F7 | P2 | ✅ | routes.rs:334 单段注释 |

**F-note（局域网 0.0.0.0）**：main.rs DEFAULT_BIND=0.0.0.0 + 非回环未设 token 告警 + 22 号 §5/§10.1 文档充分 = 用户 2026-11-06「内网自用不启用强鉴权」既定取舍，不计新发现。

## 二、审计面核对（7 项全过）

架构（同构 + resolve_action 委托 + handler 抽 mod.rs + routes.rs 846<900）/ 规范（22 号口径一致 + 30 号 §2.2 1:1 + 硬规则 0 命中）/ 文档（22 号/30 号/14 号 + release.md 计数同步 + AGENTS 路由表）/ 门禁（gate 0 failure + doccheck 0 + doccheck2 0 + ccs PASS + audit_code PASS）/ 回归（ti-console 16 向导单测 + map_wizard 8 测试 + 前端 CT 不破坏）/ 安全（auth 403 + CT 400 + id 白名单 + F3 校验 + enable_write 默认关 + 64KiB 413）/ 依赖（无新增 + 无 shell 拼接）。

## 三、Findings

### N1（P2）— e2e 状态证据早于最终代码，签收强制路径未端到端演示

**位置**：`evidence/console/content-ui-u01.json`（mtime 09:48）vs `ui_wizard.rs`（11:01，F2 写入）。zero_rust 步 criterion 不含「签收留痕判据」，证明由含 F1 不含 F2 的中间版本生成；最终代码后 e2e 未重跑。功能正确（signoff 已填含关键词 ⇒ 最终代码下 step6 仍过），但证据链有缺口。

**建议**：重跑平台 e2e（含 signoff）重新生成状态文件，使 zero_rust criterion 含「签收留痕判据」；或证据 §2 补注「signoff 强制路径由单测覆盖，e2e 状态为 F1 后/F2 前中间版本」。

## 四、低severity 注记（P3，不计新发现，不阻断）

| # | 位置 | 注记 |
|---|---|---|
| P3-a | ui_wizard.rs:420-421 | merge_params 重复 doc comment |
| P3-b | ui_wizard.rs:558-567 + :337-341 | layout 步 git diff 跑两次（冗余，只读无害） |
| P3-c | 30 号 §2.2 step 1（:55） | step1 判据写「用户视觉确认留痕」但 design 仅记录 signoff，强制在 step6 |
| P3-d | evidence §10 | 写「9→15」单测，实际 16 |
| P3-e | AGENTS.md 头部 | 存量漂移（296/295 vs 台账 308/306），路由表行正确 |
| P3-f | ui_variant.py:140-151 | validate_data 失败无回滚（内部工具） |

**误报 = 0**；r1 残留 = 0。

**总判定：FAIL**（N1 P2）。修复 N1（重跑 e2e 或证据补注）后可视为收敛。
