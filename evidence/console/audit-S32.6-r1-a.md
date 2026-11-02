# S32.6 对抗审计报告 · r1 · A 侧（完成度审计）

> 只读审计：未改文件/台账，未跑构建。DoD 逐条核对。

```
总判定: PASS
新发现: 0
误报: 0
残留: 0
```

## findings 表

无 P1/P2 缺陷（1 条 P3 观察 O1，不计入新发现）。

## 判据逐条（摘要）

1. close.rs 结构完整（HUMAN_VIEW_IDS 14 / whitelist_audit / map_cmd_id / close_check / close_run / exp_check / EXP_VOLUMES）。
2. 判据① close_check 实跑 exit 0 + 日志落 evidence/console/logs/close-check-*.log。PASS。
3. 判据② 未登记 id（sh/-c）→ 400。PASS。
4. 判据③ 失败不改台账（sha256 2869ba1c... 前后一致 + 日志「判据失败 ⇒ 不写证据、不改台账」）。PASS。
5. 判据④⑤ 三处同步 + 白名单对账断言（14 id == 14 id，write_required=[close_run]）。PASS。
6. 判据⑥ gate 全绿（33 项）+ wasm-browser-e2e 瞬时 flake 说明（重跑 PASS）。PASS。
7. 判据⑧ evidence/S32.6.log 含 §一~§六。PASS。
8. 改动面声明：routes.rs /api/close 分派 + routes/close_api.rs + web 收口页 + 文档登记三处。PASS。

## 观察（P3，不计入）

| # | 位置 | 观察 |
|---|---|---|
| O1 | evidence/S32.6.log §五 | gate 全绿结论行未独立落盘（建议后续摘录 gate 输出）；达标但记录偏薄。 |

## 结论

无 P1/P2 缺陷，判据全达成；判据⑦（双审计）为流程中间态（r1 任务书已发，B 侧待归档）。
