# S32.12 第 2 轮审计 · B（一致性与风险）

```
总判定: PASS
新发现: 0
误报: 0
残留: 0
```

> 落盘说明：第 2 轮 B 侧审计落盘凭证（会话内报告全量落盘）。只读；实跑 10 条命令全 EXIT=0；
> 未跑 gate.ps1 全量；独立重算三方哈希（已装包 manifest ↔ dist 目录 ↔ dist zip ↔ assets/ui 盘上）。

## 第 1 轮 B 侧 F-1/F-2 闭环：2/2 CLOSED

F-1（一次性脚本未删，doccheck2 红）CLOSED：`ls scripts/tmp/` = 仅 README.md（目录在位，内容空，
符合 14 号 §tmp 口径）；实跑 doccheck2 = 0 error（38 项）；log §三 判据 4 当时贴的「0 error」为
删脚本前旧输出，§四 已如实补「脚本删除后复跑 = 0 error」段，留痕自洽。
F-2（dist 分发件旧哈希）CLOSED：实跑 verify dist/maps/{m01,m02,m03}.map-pack.zip 各 0 违规
（r1 时 m01 = 22 项不合规，现已闭环）；独立重算：dist 目录 manifest == 已装包 manifest（三包逐字节
相等）；zip 内 manifest == 目录版；dist pool/POOL_INDEX.json（zip 内与目录版一致）11 条目 =
已装包 manifest pool 段 11/11 哈希+字节数匹配；zip 内 pool/ 下 11 枚 PNG 逐一 sha256/字节数 ==
POOL_INDEX == 盘上 assets/ui/*.png 实测值；每包 pool 条目与包专属 island_home_mNN.png 对应，
无串包；dist 三套 mtime = 10-30 02:01（S32.12 修复批），与 log §四 记载顺序吻合。

## 一致性面 a~e：全 PASS

a) 21 号 §7 运行期加载契约 vs dist re-pack 后 POOL_INDEX：**PASS**（POOL_INDEX = manifest pool 段
   投影，11 条，哈希/字节数与盘上文件一致；zip 内与目录版一致；与 §7 直读契约无矛盾）
b) `cmd_pack` L230-260 POOL_INDEX 更新逻辑 vs 实际内容：**PASS**（pack 先 verify_pack I7 →
   pool 副本 copy2 → 重写 POOL_INDEX；幂等同输入同产物；re-pack 0 新矛盾）
c) S32.12 DoD「gate 全绿」等价机器判据口径 vs 台账原文：**PASS**（log §三 判据 4 = 六项等价分解
   + 第 17 项可信度 = S32.11 verified；m04 三段口径 log §三 判据 3 与台账 DoD 原文逐段吻合）
d) 15 号 L54 计数 179 可复算：**PASS**（S32.0a 侧 170 + S32.11 侧 5 + S32.12 r1 A 2 + B（F-1+F-2）= 4
   ⇒ 179；逐份实读确认）
e) 全库 grep 残留（S32.12 相关「未修/在途」旧口径）：**PASS**（`grep -rn s3212` docs/AGENTS.md = 0 命中；
   14 号 `scripts/` 无散落一次性 .py（doccheck2 38 项 0 error 兜住）；`scripts/tmp/` 空（README 除外））

## 新发现

**0 新发现**（P0×0 / P1×0 / P2×0）。

排查面（均无 S32.12 引入矛盾）：dist re-pack 三方 manifest 全等；cmd_pack pool 分支逻辑与产物一致；
m04 处置 11 项违规 = S16.4 在途（r1 已标注，非 S32.12 面）；ledger 243 vs log 落盘时 242 = 并行
会话收口计数漂移（声称点已随动，doc_numbers 0 漂移）；i18n 孤儿 key WARN = 在途线存量（r1 F-4 同源）；
r1 B 侧 F-3/F-4（index 子命令未进 21 号 §8 表 + I4 无工具检查 + i18n 孤儿 key）= HEAD 存量，
r1 已显式排除出 S32.12 计数，维持归 S31/i18n 线。

## 实跑验证（10 条全 EXIT=0，0 红项）

map_pack.py list = m01~m03 OK + m04 11 项（S16.4 在途）/ check = 0 漂移（4 包）/
verify assets/maps/m0{1,2,3} 各 0 违规 / verify dist/maps/m0{1,2,3}.map-pack.zip 各 0 违规 /
doccheck 0 error / doccheck2 0 error（38 项）/ ledger 0 error（262 任务 243 verified）/
check_code_standards PASS / check_ledger_reality 全绿 / doc_numbers 0 漂移（实测 33 项）。

## 总判定

**PASS（收敛候选轮）**：第 1 轮 F-1/F-2 全 CLOSED；一致性面 a~e 全 PASS；实跑 10 条全绿；
0 新发现 + 0 误报 + 0 残留。按 §7.4 连续 2 轮 0/0/0 口径，r2 + r3 = 连续 2 轮达标后
主线程独立复核（§7.5）+ log 补「双审计通过」段 → S32.12 置 verified。
