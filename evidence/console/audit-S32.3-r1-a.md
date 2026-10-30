# audit-S32.3-r1-a

```
task: S32.3
round: r1
side: a
new: 5
```

## 判据实跑（逐条）

### 1. index.html 7 页 + 左菜单分组 + 0 外部依赖

**在位 ✓**：`pages = ['overview','tasks','evidence','assets','design','map','settings']`（index.html L287）。
左菜单 `.rail` 分组 = 看板组（概览/任务/证据/资产/设计图/地图）+ 平台组（设置）✓。
0 外部 JS/CSS：index.html 无 `<script src>` 或 `<link href>`，全部内联 JS + 内联 CSS ✓。
设计图页组别标签 7 类（主地图/首页/岛型·地图/对战/界面/资产表/其他）+ 搜索框 + lightbox ✓。
地图页包卡片 + 制作指引 `<pre>` 命令面 + 21 号标准链接 ✓。

### 2. routes.rs 端点接线

**在位 ✓**：`/api/overview` `/api/tasks` `/api/evidence?sub=` `/api/assets` `/api/session` `/api/audit` `/api/map` 全接线（routes.rs L45-67）。
`sub=` 参数解析：`url.split('?').nth(1).strip_prefix("sub=")` ✓。

### 3. repo/data.rs

**在位 ✓**：
- `evidence_list` sub 参数 + `..` 段拒（L97-117，`check_relative` 含 `..` 段拒，单测 `evidence_list_lists_dir` 验证 `sub=".."` → Err ✓）
- `map_packs_detail`（L254-309）：`resolve_guarded` `assets/maps/` + 逐包 manifest 摘要 + m04 空骨架不崩（`manifest_present:false` + `note`）+ `.default` 白名单精确项（`default_map_pack` L318-325）✓
- `audit_list`（L182-220）：`resolve_guarded` `evidence/console/` + `audit-*.md` 文件名解析 + §7.3 四行机器字段 ✓

### 4. 机器面实跑

```
cargo build -q -p ti-console          EXIT=0
cargo test -q -p ti-console           25/25 PASS
cargo clippy -q -p ti-console --all-targets   0 warning
cargo tree -p ti-console | grep -E "ti-core|ti-platform|ti-app|ti-engine|ti-render|bevy|macroquad"
                                      → 0 命中（GREP_EXIT=1）
python3 scripts/check_code_standards.py --check
  CS-1 asset_path_hardcode：0 存量违例全在豁免基线内
  CS-2 file_doc_present：1 存量违例（crates/ti-engine/src/bin/android_entry.rs，非 ti-console）
  CS-3 fn_param_naming：134 存量全在豁免基线内（含 ti-engine ease_out 单字母 = S32.3 范围外）
  CS-4 scene_session_ctor：0 违例
  整体：PASS（0 项超基线）
```

### 5. 白名单面

`console_sources.json` 10 条 `repo_sources` ↔ 22 号 §5 表 10 行 1:1 ✓（脚本比对全 OK）。
证据/设计图/地图数据全走 `resolve_guarded` ✓。

### 6. 留痕面

7 张截图在位：
```
evidence/console/ui-s32.3-overview.png   67 KB
evidence/console/ui-s32.3-tasks.png     213 KB
evidence/console/ui-s32.3-evidence.png   80 KB
evidence/console/ui-s32.3-assets.png   189 KB
evidence/console/ui-s32.3-design.png  1197 KB
evidence/console/ui-s32.3-map.png      156 KB
evidence/console/ui-s32.3-settings.png 58 KB
```
`evidence/S32.3.log` §六「用户视觉验收通过」✓；设计图/地图页同步记录 ✓（L120-123 + L152-168）。

### 7. 文档面

- 22 号 §6.1 页面矩阵含「设计图」行（P0 S32.3）+「地图」行（P0 S32.3）✓
- `docs/plan-s32-platform.md` L30 S32.3 行 DoD 含设计图/地图页 ✓
- 15 号第 22 行（平台 UI 视觉签收，台账任务号 S32.3）✓

---

## Findings

### P2-A1：`web/app.js` + `web/_shotpage.js` 为死文件遗留

**现象**：`crates/ti-console/web/` 含 `app.js`（240 行）和 `_shotpage.js`（15 行），但 `index.html` 无 `<script src>` 引用，全部 JS 内联。`app.js` 头部注释写「正式 UI = 导航按钮切换（app.js）」，`_shotpage.js` 写「非正式交付面 —— 正式 UI = 导航按钮切换（app.js）」，与实际交付的 index.html 内联 JS + `.rail` 结构矛盾。

**证据**：
- `git status --porcelain crates/ti-console/web/` → `?? crates/ti-console/web/`（整目录未跟踪，app.js 和 _shotpage.js 一并未跟踪）
- `grep -n "script src" index.html` → 无命中
- `app.js` L200：`document.querySelectorAll(".nav button")` — index.html 实际用 `.rail nav button`（L132-143）
- `app.js` L180 `loadAll()` 一次性并发 fetch 5 端点；index.html 内联 JS 用懒加载 `loaded.{page}` 模式

**修复建议**：删除 `app.js` 和 `_shotpage.js`（一次性出图脚本 `_shotpage.js` 按 15 号 L22「用完即删」口径应当删除；`app.js` 为早期设计稿遗留）。若 `_shotpage.js` 仍用于 S32.3 截图复现，应在 S32.3 verified 后立即删除。

**误报判定依据**：非误报。文件在 `crates/ti-console/web/` 目录内（S32.3 改动面），且注释与实际交付矛盾，属 S32.3 自身留痕不完整。

**级别**：P2（不影响功能/安全性，纯死文件清理）。

---

### P2-A2：`routes.rs` L7-8 注释与实际不符

**现象**：`routes.rs` L7 写「看板 UI v1（概览/任务/证据/审计/设置 + **P1/P2 占位页**）」，但 S32.3 实际交付 7 页全功能（无占位页）；L8 写「视觉规范单一事实源 = `web/theme.css`」，但 index.html 不引用 theme.css（全部内联 CSS）。

**证据**：
- `sed -n '7,8p' crates/ti-console/src/routes.rs`
- `grep -n "theme.css" crates/ti-console/web/index.html` → 0 命中
- index.html L8-122 全部为内联 `<style>` 块

**修复建议**：将 L7 改为「看板 UI v1（概览/任务/证据/资产/设计图/地图/设置 7 页）」；将 L8 的 theme.css 描述删去或改为「视觉规范 = 内联 CSS（设计令牌：航海夜·精修）」。

**误报判定依据**：非误报。注释是 S32.3 代码改动面内的文件，描述与实际交付不符。

**级别**：P2（stale 注释，无运行时影响）。

---

### P2-A3：`data.rs` L175-176 两个 `///` 文档块合并（格式瑕疵）

**现象**：`session()` 的 `///` 文档块后紧跟 `/api/audit` 的 `///` 文档块，两者之间无代码项，形成一个"双文档块"（`///` 直接接 `///`）。

**证据**：
```
L175: /// `/api/session`：平台会话信息（...）。
L176: /// 只读：不读 token 值...
L177: /// `/api/audit`：双审计报告清单（...）。   ← 应为 audit_list 的文档，紧跟 session() 结尾
L182: pub fn audit_list(...)
```

**修复建议**：`session()` 和 `audit_list()` 是两个独立函数，L177-181 的 `///` 块应移到 `audit_list` 函数声明的正上方（当前它紧跟 `session()` 的文档块，虽然 Rust 编译器会把它绑定到 `audit_list`，但阅读体验差）。

**误报判定依据**：非误报（格式瑕疵，非 bug；编译器绑定到正确的 `audit_list`）。

**级别**：P2。

---

### P2-A4：22 号 §5 表 `assets/maps/<id>/` vs. `console_sources.json` `assets/maps/` 语义差异

**现象**：22 号 §5 表第 7 行首列写 `assets/maps/<id>/`（语义 = 每个已装包目录），但 `console_sources.json` 白名单项 = `assets/maps/`（前缀，覆盖 `assets/maps/` 本身 + 所有子路径，含 `index.json`）。`map_pack.py` 的 `install` 写 `index.json`，平台需读它才能知道 `default` 字段；白名单 `assets/maps/` 前缀是**有意**的（让 `index.json` 和 `.default` 可读），但 22 号 §5 表写的 `<id>/` 暗示只有包目录，语义与白名单实际放行范围不一致。

**证据**：
- `console_sources.json` L9：`"assets/maps/"`（前缀项）
- `docs/process/22_dev_platform.md` L145：`| assets/maps/<id>/ |`
- `map_pack.py` install 写 `assets/maps/index.json`（白名单前缀 `assets/maps/` 放行）

**修复建议**：22 号 §5 表第 7 行改为「`assets/maps/`（已装包 + `index.json` + `.default` 指针）」，与白名单前缀实际语义对齐；或在表中注明「白名单前缀 = `assets/maps/`，`<id>` 语义由 `map_pack.py installed_packs()` 自扫描收口」。

**误报判定依据**：非误报（文档 ↔ 白名单语义不一致，可能误导后续开发）。

**级别**：P2。

---

### P2-A5：`evidence/console/.lock` 残留（pid 3260474 已不存在）

**现象**：`evidence/console/.lock` 内容为 pid 3260474（文件 mtime 2026-10-30 10:40:16），该 pid 已不存在（`ps -ef | grep ti-console` 无命中），但锁文件未被清理。任何后续 `cargo run -p ti-console` 启动都会报「单实例锁被占用（pid 3260474...）」，需手动删除锁文件才能重启。

**证据**：
- `cat evidence/console/.lock` → `3260474`
- `ps -ef | grep ti-console | grep -v grep` → 空
- `cargo run -q -p ti-console` 实测：「单实例锁被占用（pid 3260474，当前 326XXXX）；先删 evidence/console/.lock 再启动」

**修复建议**：① 立即删除 `evidence/console/.lock`（一次性操作）；② `main.rs` `write_lock` 增加 stale 检测：若锁文件中的 pid 已不存在（`/proc/<pid>` 不可读）则自动接管，避免下次启动被卡（S32.4 规格中「30 分钟 stale 接管」可在此一并实现）。

**误报判定依据**：非误报（影响 S32.3 收口后 `cargo run -p ti-console` 的可用性；`.gitignore` 已含 `/evidence/console/.lock`，不会被 commit，但本地残留会阻塞开发）。

**级别**：P2（操作性问题，不影响已交付功能，但影响续工）。

---

## 误报排除说明

- ti-engine CS-3 `ease_out` 单字母参数：S32.3 范围外（ti-engine 非 S32.3 改动面）→ 不计入
- i18n 孤儿 key：S32.3 范围外 → 不计入
- android 脏面（`crates/ti-app/src/bin/treasure_android.rs` deleted）：S32.3 范围外 → 不计入

## 收敛判定

new = 5（全部 P2，无 P0/P1）；误报 = 0。
**非收敛轮**（5 > 0，需 r2 确认清零）。
