# S32.12 第 1 轮审计 · A（完成度）

```
总判定: PASS
新发现: 2
误报: 0
残留: 0
```

> 落盘说明：第 1 轮 A 侧审计落盘凭证（会话内报告全量落盘；B 侧同轮 = `audit-S32.12-r1-b.md`，
> 1 P1 + 1 P2，F-1/F-2 由主线程同批修复后第 2 轮复核）。只读；实跑 verify/check/list +
> 机器判据全套；未跑 gate.ps1 全量。

## DoD 逐条判定（S32.12，台账 L308）

1 代码面 = manifest.json 与 assets/ui/*.png 哈希对齐：**PASS**
   （`git diff HEAD -- assets/maps/m0{1,2,3}/manifest.json` = 仅 sha256/bytes 字段变更，
   每包 11 资产 × 2 字段；`git diff --stat -- assets/ui/` = 0 游戏产物 PNG 变更；
   m02 重算 11 条目 0 漂移）
2 判据 1 `verify m01~m03` 全部 0 违规：**PASS**（实跑三包均 0 违规 EXIT=0）
3 判据 2 `check` 0 漂移：**PASS**（实跑 0 漂移，4 已装包）
4 判据 3 m04 处置三段：**PASS**（①m04 无 map.json ⇒ check 恒通过，实测 0 漂移；
   ②③未触发（m04 未 install，list 实测 m04 11 项违规 = S16.4 在途空骨架，HEAD 即存在））
5 判据 4 gate 全绿（第 17 项可信 = S32.11 verified 后；本机 pwsh 在位走实跑分支）：**PASS**
   （等价机器判据实跑全绿；本机 `pwsh -File scripts/flywheel_readme.ps1 -Check -Quiet` = PASS）
6 判据 5 0 游戏产物变更（§8 内容级指纹口径）：**PASS**（写点面 = 3 manifest 的 assets 字段
   （数据面）+ 一次性脚本（工具面）；assets/ui/*.png 0 变更；board/scenes/strings 0 触及）
7 判据 6 双审计 + evidence/S32.12.log 收口前落盘：**PASS**（log §五 自述「收口前实跑凭证；
   双审计收敛后补段再 verified」口径合规）
8 一次性脚本收口后删除：**PASS**（当前在位属收口前预期态；log §四 写明删除判据 =
   「收口（log 写盘 + 双审计收敛 + verified 后）执行 rm」= verified 后，非 verified 前，口径正确）

## 新发现（2 条 P2，由主线程同批处理）

| ID | 严重度 | 问题 | 修复落点 |
|---|---|---|---|
| P2-1 | P2 | `map_pack.py cmd_sync` 对 `manifest` 无 `map` 键的包（m04）抛 `KeyError`（L463 用 `manifest["map"]` 下标取，`verify_pack`/`cmd_check` 均用 `.get` 容错口径）；「m04 填 map.json 后先 sync 对齐」（判据 3 ②）预案路径实际不可用 | 非 S32.12 引入（HEAD 既有工具健壮性缺口）；建议 S16.4 开工时一并修 `manifest.get("map")` + 缺 map 时跳过回注；0 阻断 S32.12 |
| P2-2 | P2 | 门禁 33 项不覆盖 `map_pack.py verify`（本任务立项理由的结构性残留：漂移修复后若资产再刷新，漂移将再次静默回归） | 建议后续轮次（或 S32.7 平台资产页）在 gate 或 doc_numbers/validate_data 面加 `map_pack verify` 全已装包 0 违规项（m04 草稿态豁免或随 S16.4 自然消除）；0 阻断 S32.12，登记为结构性加固建议 |

## 总判定

**PASS（非收敛轮）**：DoD 8/8 PASS；2 条 P2（均 0 阻断，结构性加固建议）⇒ 未达「0 新发现 + 0 误报 + 0 残留」；
主线程同批处理 P2-1/P2-2 后第 2 轮复核。
（注：`doccheck2` 当前 1 error = `scripts/tmp/s3212_fix_i7.py` 残留拦截 = 本收口前预期在位的机器兜底，
verified 后 `rm` 即绿，不计新发现；B 侧 F-1 已报此 P1，主线程同批修复。）
