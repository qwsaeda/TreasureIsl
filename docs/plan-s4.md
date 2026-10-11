# 执行计划 S4：现代化与发布（1~2 周）

| ID | 产出物 | 验收命令 / DoD | deps |
|---|---|---|---|
| S4.1 | 缩放模式设置（整数倍/平滑）、虚拟方向键与手势、按键自定义、横竖屏 | P4-2 单测全绿 + 三端手动清单（存 `evidence/S4.1-checklist.md`） | S3 |
| S4.2 | 存档 v2（导出/导入、IndexedDB 与 Android 存储互通）+ v1 自动迁移 | `cargo test -p ti-core --test save_migration` 往返一致；旧档迁移用例通过 | S3.10 |
| S4.3 | 性能与体积：脏矩形、纹理批处理、wasm 资源分包 + gzip/brotli、启动加载进度 | 桌面 60fps（`--features stats` 打印）；wasm 首屏资源体积记录进 `evidence/S4.3-size.log` 并与下方预算比对 | S4.1 |
| S4.4 | 全流程回归 + 三端产物：桌面 exe、wasm 静态站点、APK；发布说明 `docs/release.md` | ① P3-3 固定种子全流程脚本三端各跑一遍通过；② `scripts/gate.ps1` 全绿（S4 时点 8 项，现 13 项）；③ 产物路径与 sha256 记录 `evidence/S4.4.log` | S4.2, S4.3 |

## S4.3 体积预算（开工时写入，DoD 比对基准）

> 2026-10 修订（资源改造 Step 1，`docs/modernization.md` §9）：此前 `web/` 只拷字体+音频，
> 美术层整条链是空的（全量美术 380 KB 未进包）。首屏/非首屏预算按补齐后的分包重列。

| 项 | 预算 | 说明 |
|---|---|---|
| wasm 主体（`treasure_wasm.wasm`） | ≤ 800 KB | S2.5 实测 669,815 B；S4.3 允许增长但不得超 800 KB |
| 首屏附属资源（index.html + gl.js + audio.js + `GoldLand.Fnt` + `font.dat` + `settings.dat` + `ui.pak` + `glpics.pak` + `index.Map` + `intro.Map`） | ≤ 350 KB | 首屏 = 阻塞首帧渲染的非 wasm 资源 |
| 非首屏资源（地形瓦片/地形布局 + `cards.lpk` + `goods.lpk` + 其余 18 个场景 `.Map` + `music.lpk`） | ≤ 450 KB | 进对应场景时或空闲补载，不阻塞首屏 |

- **美术不转码**：PCX 已是最优压缩（RLE + 调色板，88 张精灵压缩态 309 KB），
  且 `.Map` 记录按精灵索引 `f1` 寻址，转图集会打断寻址并引入像素偏差
  （P4-1 要求逐像素一致）。唯一必须转码的是音频（MOD → OGG/Opus，S1.7b）。
- gzip/brotli 说明：`build_wasm.ps1` 不引入压缩依赖（`ti-tools` 零依赖原则，§5.9）；
  浏览器对 `application/wasm` 与 JS 普遍启用传输层压缩（静态服务器 gzip 即生效），
  产物体积以未压缩字节记录，传输层压缩作为部署侧关注项记入 `docs/release.md`。
- 启动加载进度：wasm 壳 `index.html` 已有「加载 wasm…」状态行；
  分包后首屏 = wasm + 字体 + 主菜单必需美术，进度语义 = 首屏资源加载完成，
  无需新增进度条（主菜单即首屏，首帧渲染后状态行已显示资源计数）。

## 项目完成 = DoD（全部满足）

1. `plan/ledger.yaml` 全部任务 `verified` 且 evidence 齐全（gate 第 8 项台账校验；S4 时点门禁 8 项，现 13 项）；
2. `docs/parity.md` **P0~P4 五级全 pass**，各自报告存 `evidence/`；
3. 三端产物生成：桌面 exe / wasm 站点 / APK（S4.4 ③）；
4. 占位扫描 0 命中（gate 第 5、6 项）；
5. 文档齐全：`docs/formats/*`、`docs/spec/*`、`docs/release.md`、`docs/experience/index.md` 已更新至最终状态。

## 发布后遗留（明确不阻塞完成，但必须登记）

- 未逆出但已确认无影响的字节区（如存在）→ 记入 `docs/facts/index.md` 缺口表；
- 可选增强（QEMU-CE 视觉对照、多人/联机、成就）→ 单列到 `docs/release.md` 的「后续」章节，**不建任务**，避免污染台账。
