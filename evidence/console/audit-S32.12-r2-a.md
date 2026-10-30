# S32.12 第 2 轮审计 · A（完成度）

```
总判定: PASS
新发现: 0
误报: 0
残留: 0
```

> 落盘说明：第 2 轮 A 侧审计落盘凭证（会话内报告全量落盘；B 侧同轮 = `audit-S32.12-r2-b.md`）。
> 只读；实跑 verify/check/list + verify dist zip ×3 + doccheck/doccheck2/ledger/
> check_code_standards/check_ledger_reality/doc_numbers；未跑 gate.ps1 全量。

## DoD 逐条（S32.12，8/8 PASS）

1 改动面=数据，manifest.json 与 assets/ui/*.png 哈希对齐 ✓（m01~m03 diff = 仅 assets 段
   sha256/bytes 字段，每包 11 资产；`git diff --stat -- assets/ui/` = 0 PNG 变更）
2 判据 1 verify m01~m03 全部 0 违规 ✓（实跑三包均 0 违规 EXIT=0）
3 判据 2 check 0 漂移 ✓（实跑 0 漂移，4 已装包）
4 m04 处置三段 ✓（①无 map.json ⇒ check 恒通过，实测；②③未触发，m04 未 install；
   m04 verify 11 项 = S16.4 在途空骨架，HEAD 既有，非 S32.12 引入）
5 判据 4 gate 全绿（等价机器判据口径，log §三 判据 4 实跑 + S32.11 verified 后第 17 项可信）✓
6 判据 5 0 游戏产物变更（§8 指纹口径）✓（写点面 = manifest 字段数据面 + 一次性脚本工具面（已删）；
   assets/ui/*.png 0 变更；board/scenes/strings 0 触及；dist re-pack 非冻结路径）
7 判据 6 双审计 + evidence/S32.12.log ✓（log §一~§五 全在位；r1/r2 双报告在盘）
8 一次性脚本收口后删除 ✓（`ls scripts/tmp/` = 仅 README.md，`s3212_fix_i7.py` 已删；
   log §四 已补「rm + dist re-pack ×3 + verify dist zip 0 违规 + doccheck2 0 error」实跑输出）

## 第 1 轮 P2/F-1/F-2 闭环判定

| ID | 判定 | 证据 |
|---|---|---|
| F-1（P1，一次性脚本未删，doccheck2 红） | **CLOSED** | `ls scripts/tmp/` = 仅 README.md；实跑 doccheck2 0 error（38 项，含第 27 项 tooling 契约面） |
| F-2（P2，dist 分发件旧哈希） | **CLOSED** | 实跑 verify dist/maps/{m01,m02,m03}.map-pack.zip 各 0 违规；zip 内 manifest = 新哈希（与已装包一致）；dist mtime = 10-30 本收口批 |
| P2-1（cmd_sync 对无 map 键包抛 KeyError，L463） | **在途挂账（非 S32.12 阻断）** | 15 号第 24 行已新增（挂 S32.7；S32.7 收口时顺带修，1 行改动）；P2-1 修复后 15 号第 24 行转「半自动」 |
| P2-2（门禁不覆盖 map_pack verify 结构性缺口） | **在途挂账（非 S32.12 阻断）** | S32.7 DoD「校验状态只调 map_pack.py verify，已装包集合 = 平台自扫描」= 扩面消费方；
  21 号 §11 门禁耦合点表无 verify 全已装包 0 违规行（归 S32.7 收口时补）；r1-A 建议落点 S16.4 已
  verified 失效，改挂 S32.7 |

## 其余核验面（0 新发现）

m01~m03 verify + check + list 全绿；dist 三包 zip verify 全绿（POOL_INDEX 哈希索引已刷新）；
全库 grep 残留 0（`assets/maps/m04` 11 项 = S16.4 在途；`_template` verify 7 项 = 21 号 §add 口径
（HEAD 既有，归 S31 存量）；`git diff` 面 = 仅 4 包 manifest（`_template` 删 invariants 段 =
S16.4 Linux 续工在途，不算 S32.12 面）；并行会话在途改动面（ti-engine/ti-platform/android 线 +
i18n 孤儿 key WARN + 门禁 33 项同步线）红项均非 S32.12 文档面，标注归属不计数。

## 新发现

**0 新发现**（P0×0 / P1×0 / P2×0）。

## 总判定

**PASS（非收敛轮）**：DoD 8/8 PASS；第 1 轮 F-1/F-2 全 CLOSED；P2-1/P2-2 在途挂账（15 号第 24 行
+ S32.7 扩面）；0 新发现 + 0 误报 + 0 残留；第 3 轮复核（连续 2 轮 0/0/0 后 verified）。
