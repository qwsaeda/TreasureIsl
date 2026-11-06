# 对抗审计 B（一致性/风险）· S44.3 · r1

```
总判定: FAIL
新发现: 7
误报: 0
残留: 0
```

## 审计面核对

- 架构：ui_wizard.rs 与 map_wizard.rs 同构；共享 resolve_action（content/mod.rs:16-42）行为保持（无 map 向导回归）；平台自身面与白名单动作边界清晰。
- 规范：22 号 §5/§10.1 局域网口径 ↔ main.rs DEFAULT_BIND=0.0.0.0 一致；30 号 §2.2 六步 ↔ UI_STEPS 1:1（但 §2.2 step 3/6 判据与实现不一致，见 F1/F2）；硬规则全过。
- 文档：release.md:239 计数同步（306 verified）；30 号 §2.2 与实现部分不一致。
- 门禁：gate 0 failure；doccheck 0 error；doccheck2 0 error（41 checks）；check_code_standards PASS；audit_code PASS（routes.rs 实测 891 行，非证据所写 887，见 F6）。
- 回归：gate 第 4 项 cargo test --workspace PASS（含 content_ui_wizard_tests 9 项）；map_wizard 委托 + --scene=main_map 修复无破坏。
- 安全：POST /api/content/ui/step 经 handle_content_wizard_step auth_ok→403 = 有；id ^[a-z0-9_]{1,16}$ + state_path join 防 .. = 有；enable_write 默认关 = 有；但 scene/viewport/assets/suffix 未校验（见 F3）。
- 依赖：无新增依赖（Cargo.toml 未改）；无 shell 拼接。

## Findings

| # | 严重度 | 位置 | 发现 | 建议 |
|---|---|---|---|---|
| F1 | P1 | `ui_wizard.rs:476-501`（layout 执行面）；`:424-428`（layout_requested）；`:224-228`；`web/index.html:1392-1406`（uwParams 无 layout 字段）；`routes.rs:392-415`（无 scenes.json 写端点）；30 号 §2.2 step 3 | 「版面（可选）」写路径未实现。规格：平台表单改 scenes.json 原语 → 写 scenes.json。实现：layout 步仅 layout_requested 时跑 validate_data.py --check scenes（只读检查），否则 git diff --quiet scenes.json（跳过）。前端 uwParams() 从不收集 layout ⇒ layout_requested 恒 false；routes.rs 无 scenes.json 写端点；代码注释「实际写 scenes.json 由前端表单完成」失实。后果：平台无法经向导改版面；手动改 scenes.json 跑版面步不受 enable_write/写点探针门控。另 layout 跳过路径把 git diff --quiet 跑两次，冗余。 | 实现 scenes.json 写（表单 + API + enable_write 门 + write_probe 豁免点）或 降格版面步为「仅断言/跳过」并同步 30 号 §2.2 step 3 判据 + 删死分支 layout_requested；删除失实注释。 |
| F2 | P1 | `ui_wizard.rs:194-197`（zero_rust 判据）；`:416-420`（signoff 仅存 state 未校验）；`:150-157`（写 evidence/console/content-ui-*.json 非 evidence/<id>.log）；30 号 §2.2 step 6；doccheck2.rs:3351-3406 | 步骤 6「零 Rust + 签收」判据未含视觉签收留痕。规格：判据 = git diff crates/ 空 + evidence/<id>.log 含「用户视觉验收/过目/确认/签收」。实现：parse_criterion("zero_rust") 仅 exit==0 && out.trim().is_empty()；signoff 存 state 但从不校验；向导写 evidence/console/content-ui-<id>.json 而非 evidence/<id>.log；doccheck2 第 35 项只查台账任务证据的签收，不校验向导步 6。后果：无 signoff 也可全 6 步通过；A 纪律硬规则 9 机器兜底缺位。 | 在 step 6 判据强制 state.signoff 含签收关键词（或非空）+ 产 evidence/<id>.log 签收记录；对齐 30 号 §2.2 step 6 判据。 |
| F3 | P1 | `ui_wizard.rs:389-421`（merge_params 无校验）；`:529-533`（shot_output_path 用 scene+viewport 拼文件路径）；`:537-553`（--scene=/--viewport= 参数）；`:451`（--assets=）；`:460`（--suffix=）；ui_variant.py:98-99 | scene/viewport/assets/suffix 用户可控字符串未校验 → 路径穿越/命令参数注入。id 有白名单，但 scene/viewport/assets/suffix 从 params 直取（仅判空）。scene/viewport 直达文件系统写路径（出图 PNG 文件名）与子进程参数；suffix 直传 ui_variant.py（UI_DIR / f"{name}{suffix}.png" 可 .. 逃逸 assets/ui/）。后果：持 token 调用方可令出图写 evidence/ 之外、变体写 assets/ui/ 之外。 | validate_request/merge_params 校验：scene ^[a-z0-9_]{1,16}$（或场景白名单）、viewport 空或 ^[0-9]{2,4}x[0-9]{2,4}$、assets ^[a-z0-9_,]+$、suffix ^[A-Za-z0-9_]*$。 |
| F4 | P2 | `routes.rs:339-381`（handle_content_wizard_step 无 Content-Type 检查）vs `:614-629`（handle_gate 有）；22 号 §10.1 L360 | wizard 步（map/ui 共用）未强制 Content-Type: application/json，与 handle_gate/taskbook/archive 三处不一致。服务端接受任意 Content-Type 的合法 JSON body。 | 在 handle_content_wizard_step 开头镜像 handle_gate 的 Content-Type 检查（400）。 |
| F5 | P2 | `ui_wizard.rs:170-179`（variant 判据）；30 号 §2.2 step 2；ui_variant.py:61-140（无 validate_data 调用） | variant 步判据未含「validate_data 0 违规」。规格判据 = 资产生成 + ui_manifest 登记一致（disk↔registry validate_data 0 违规）。实现判据 = exit==0 && out 含「已更新」；ui_variant.py 全程不调 validate_data。 | ui_variant.py 跑 validate_data --check 且违规时非 0 退出，或向导 variant 步追加 validate_data 并纳入「0 违规」判据。 |
| F6 | P2 | `evidence/S44.3.log §9`；提交 64c7ee5；gate L210 | routes.rs 行数证据与实值不符：证据/提交写「887 行」，实测 891（wc -l + gate L210）。均 <900 ⇒ audit_code 过，仅证据数字失真。 | 更正证据为 891。 |
| F7 | P2 | `routes.rs:334-337` | 重复文档注释（两段近乎相同 doc comment）。 | 合并为一段。 |

**F-note（局域网绑定，P2 建议，非阻断）**：main.rs DEFAULT_BIND=0.0.0.0 + auth_ok token 未设「非空头即可」= 局域网任一主机可发任意非空 X-TI-Token 头触发写步骤。文档已充分说明（22 号 §10.1 + §5 + main.rs 启动告警 +「外网/生产部署须设 token 作强校验」），属 2026-11-06 用户拍板「内网自用不启用强鉴权」既定取舍，不计新发现；建议共享/不可信网络部署务必设 TI_CONSOLE_TOKEN。

**结论**：核心 e2e 达标，但 3 项 P1（版面写路径缺失、步骤 6 签收判据未机器化、scene/viewport/assets/suffix 未校验）+ 4 项 P2。建议修复后复审计。
