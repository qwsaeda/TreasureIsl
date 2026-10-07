# 执行计划 S4：现代化与发布（1~2 周）

| ID | 产出物 | 验收命令 / DoD | deps |
|---|---|---|---|
| S4.1 | 缩放模式设置（整数倍/平滑）、虚拟方向键与手势、按键自定义、横竖屏 | P4-2 单测全绿 + 三端手动清单（存 `evidence/S4.1-checklist.md`） | S3 |
| S4.2 | 存档 v2（导出/导入、IndexedDB 与 Android 存储互通）+ v1 自动迁移 | `cargo test -p ti-core --test save_migration` 往返一致；旧档迁移用例通过 | S3.10 |
| S4.3 | 性能与体积：脏矩形、纹理批处理、wasm 资源分包 + gzip/brotli、启动加载进度 | 桌面 60fps（`--features stats` 打印）；wasm 首屏资源体积记录进 `evidence/S4.3-size.log` 并与预算比对（预算在开工时写入本文件） | S4.1 |
| S4.4 | 全流程回归 + 三端产物：桌面 exe、wasm 静态站点、APK；发布说明 `docs/release.md` | ① P3-3 固定种子全流程脚本三端各跑一遍通过；② `scripts/gate.sh` 8 项全绿；③ 产物路径与 sha256 记录 `evidence/S4.4.log` | S4.2, S4.3 |

## 项目完成 = DoD（全部满足）

1. `plan/ledger.yaml` 全部任务 `verified` 且 evidence 齐全（gate 第 8 项）；
2. `docs/parity.md` **P0~P4 五级全 pass**，各自报告存 `evidence/`；
3. 三端产物生成：桌面 exe / wasm 站点 / APK（S4.4 ③）；
4. 占位扫描 0 命中（gate 第 5、6 项）；
5. 文档齐全：`docs/formats/*`、`docs/spec/*`、`docs/release.md`、`docs/experience.md` 已更新至最终状态。

## 发布后遗留（明确不阻塞完成，但必须登记）

- 未逆出但已确认无影响的字节区（如存在）→ 记入 `docs/reverse-facts.md` 缺口表；
- 可选增强（QEMU-CE 视觉对照、多人/联机、成就）→ 单列到 `docs/release.md` 的「后续」章节，**不建任务**，避免污染台账。
