# S32.12 第 1 轮审计 · B（一致性与风险）

```
总判定: FAIL
新发现: 2
误报: 0
残留: 0
```

> 落盘说明：第 1 轮 B 侧审计落盘凭证（会话内报告全量落盘；F-1 为 P1 阻断项，主线程同批已修复；
> F-2 为 P2，主线程同批已 re-pack）。只读；实跑 list/check/verify + 机器判据全套；
> 未跑 gate.ps1 全量。

## 一致性面 a~f 逐项判定

a) 21 号 §5 I7 不变量 vs `verify_pack` I7 逻辑 vs manifest 字段名：**PASS**
   （三方字段名一致：`file`/`source`/`sha256`/`bytes`；m01~m03 全部 13 项 pool 资产实测 0 漂移；
   `map_pack.py verify m01` = 0 违规 EXIT=0）
b) S32.12 m04 处置三段口径 vs 21 号 §7 + `cmd_check` L415：**PASS**（实跑确认逻辑：
   m04 在 `installed_packs()` 内；`board.maps.m04`(None) == `manifest.map`(None) = True ⇒ 恒通过；
   三段口径与 log §三 判据 3 一致）
c) 一次性写脚本写点面 vs 22 号 §3.1 写点表：**PASS**（脚本唯一写点 = manifest assets 字段；
   0 触及 board/scenes/strings/index.json；`git diff` 证实 0 游戏产物 PNG 变更；
   一次性数据修复脚本非平台白名单动作，不进写点表，留痕 S32.12.log 合规）
d) `assets/maps/index.json` 未被触及：**PASS**（`write_pack_index` 只在 install/uninstall/index 命令调用；
   `git status` 无该文件 M 标记；`index.json` = `{default: m01, ids: [m01,m02,m03]}` 实测在盘）
e) `cmd_sync` 不重写 manifest assets 段 ⇒ 本修复幂等且必要：**PASS**
   （读 `cmd_sync` L454-497：写点 = board/scenes/manifest_v3/strings；只读各包 manifest，从不回写
   `assets/maps/<id>/manifest.json` ⇒ 未来重跑 sync 不会复活本漂移）
f) 15 号 L54 计数 + S32.12 台账行 vs S32.11 verified 状态自洽：**PASS**
   （S32.11 verified rounds=1；S32.12 pending deps=[S32.11] rounds=0 取件合法；
   175 计数 = S32.0a 侧 170 + S32.11 侧 5，S32.12 数字 = 0（本轮出报告中，尚未计入））

## 新发现（1 P1 + 1 P2；F-1/F-2 由主线程同批修复）

| ID | 严重度 | 问题 | 修复落点 |
|---|---|---|---|
| F-1 | P1 | `scripts/tmp/s3212_fix_i7.py` 仍在盘 ⇒ `doccheck2` 第 27 项 `check_tooling_tmp_dir` 报 1 error（log §三 判据 4 贴的「doccheck2: 0 error」为脚本删除前旧输出，与当前盘上态矛盾）；evidence/S32.12.log §四 已写明「收口后 rm」但未执行 | 主线程执行 `rm scripts/tmp/s3212_fix_i7.py` + 复跑 doccheck2 即 0 error（**同批已修复**） |
| F-2 | P2 | `dist/maps/{m01,m02,m03}.map-pack(+.zip)` 仍携带 S27.2 旧哈希 manifest（`verify dist/maps/m01.map-pack.zip` = 22 项不合规）；本修复只刷已装包 manifest，可分发件未 re-pack ⇒ 拿 dist 件 install/verify 会复现同一漂移 | 主线程重跑 `map_pack.py pack m01~m03`（幂等，写 `dist/` 非冻结路径）+ 复跑 verify dist zip 即 0 违规（**同批已修复**） |

（F-3/F-4 存量备注不计入 S32.12 新发现：F-3 = map_pack index 子命令未进 21 号 §8 表 +
I4 无对应工具检查 + m01~m03 invariants 缺 feature/isle_kinds 键，均 HEAD 既有归 S31/i18n 线；
F-4 = i18n 孤儿 key WARN，归 i18n 在途线。）

## 实跑验证（只读项全绿；F-1 修复前 doccheck2 = 1 error，修复后 = 0 error）

| 命令 | 结果（F-1 修复前） | 结果（F-1 修复后） |
|---|---|---|
| `map_pack.py list` | 0 exit；m01~m03 OK；m04 11 项（S16.4 在途） | 同 |
| `map_pack.py check` | 0 漂移 EXIT=0 | 同 |
| `map_pack.py verify m01/m02/m03` | 各 0 违规 EXIT=0 | 同 |
| `doccheck2` | **1 error**（scripts/tmp 残留） | **0 error**（38 项全绿） |
| `doccheck` | 0 error | 0 error |
| `ledger` | 262 任务 242 verified 0 error | 同 |
| `check_code_standards.py --check` | PASS 0 超基线 | 同 |
| `check_ledger_reality.py` | 全绿 242 verified + 0 孤儿 | 同 |
| `doc_numbers.py` | 33 项/262 任务 0 漂移 | 同 |

## 总判定

**FAIL（非收敛轮）**：一致性面 a~f 全 PASS；唯一阻断项 F-1（P1）= 一次性脚本未删导致 doccheck2 红
（修复 = 一行 `rm` + 复跑，主线程同批已执行）；F-2（P2）= dist 分发件旧哈希 re-pack（主线程同批已修复）。
第 2 轮复核（0 新发现 + 0 误报 + 0 残留连续 2 轮才 verified）。
