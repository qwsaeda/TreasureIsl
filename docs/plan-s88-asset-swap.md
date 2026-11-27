# S88 — 可视化换图（UI 资料更新，业务逻辑零改动）

## 1. 定位（用户 2026-11-27 拍板）

用户诉求："换图片（主要是图片，比如给英雄换形象）都有可视化操作。业务逻辑不动的情况下，不就是 UI 资料的更新吗？"

结论：**换图 = 编辑 `asset_roles.json` 的 `role → 资产路径` 映射**，属内容/UI 资料更新，不碰任何 Rust 业务逻辑。编辑器内完成：选图 → 预览 → 写盘生效。

## 2. 职责/协议单一事实源（先定协议，避免又"修A出B"）

| 职责 | 单一事实源 | 谁写 | 谁读 |
|---|---|---|---|
| 元素该显示哪类图 | `components_v1.json` 元素 `role`（已有写点 apply） | UI 编辑器属性面板 | 引擎 `asset_roles().path(role)` |
| role 指向哪张图 | `asset_roles.json::roles[role] = path`（**本轮新增受控写点**） | 本方案换图写点 | 引擎 + 预览注入 |
| 可上传的图池 | `ui_manifest.json::assets`（387 张已登记，含 role/prompt/params） | `scripts/ui_assets.py add` | 资产选择器 + 校验 |
| 预览联动 | `POST /api/preview` 的 `asset_roles_json` blob（已支持） | 前端换图后发整份映射 | 引擎 `TI_PREVIEW_DIR` 注入 |

## 3. 写点协议（扩展已有 apply 端点，用户拍板）

- `POST /api/ui/components/apply` 增加 body blob `asset_roles_json`（整份 `asset_roles.json` 文本/object）
  + action 仍为 `ui_components_apply`（零新端点、复用 enable_write/校验/回滚/门禁）。
- 后端 `apply_components` 检测 blob 存在 ⇒ **写 `asset_roles.json`**（不写 components），
  写前校验：① 每个 path 须在 `ui_manifest.assets` 登记（禁手写路径）② role 名合法 ③ 文件存在；
  写后跑 `ui_schema_check.py` + `validate_data` 门禁，FAIL 回滚。
- 与 components 写点互斥：body 带 `asset_roles_json` = 换图写；带 `schema_text` = 组件写（现状）。

## 4. 选图来源（用户拍板：支持上传新图）

- **A. 资产池选图（主路径）**：`/api/assets` 的 `ui_manifest.assets` → 编辑器缩略图网格 + 搜索，
  选中即把该 asset 的 `file` 写入 `roles[role]`。覆盖"换英雄形象"（role_drake/panda 等现成立绘）。
- **B. 上传新图（新增）**：前端 file input → 新端点 `POST /api/ui/assets/upload`（**独立于 64KiB 通用 POST
  上限**，走 `read_post_body` 的 256KiB 上限口径需评估；大于 256KiB 的 PNG 用分片或直接提升该端点上限）。
  上传后：落 `assets/ui/<name>.png` + `ui_assets.py` 登记（disk↔registry 双向同步）+ 自动映射到所选 role。
  上传通道写点须登记 `console_sources.json` + 22 号写点表 + 工具契约。

## 5. 前端交互

- 属性面板"资产角色"行旁加「换图」按钮 → 弹资产选择器（网格 + 搜索 + 上传入口）。
- 选中/上传后：本地更新 `S.assetRoles[role]=path` → 点预览时把 `asset_roles_json` 发 `/api/preview`
  → iframe 真渲染出图；点「写盘 apply」走换图写点持久化。

## 6. 验收判据

- 选一张候选图改 `index_hero` → 预览 iframe 英雄形象变化（像素 diff > 阈值）；不改 Rust。
- 上传新 PNG → 登记进 ui_manifest + 映射 role → 预览生效 + 写盘后 native `--shot` 一致。
- 负例：手写未登记路径 → 校验 FAIL 回滚；写后门禁 FAIL → 回滚。
- gate 0 failure + doccheck/doccheck2 0 error + 三方对账。

## 7. 任务（台账）

S88.0 立项+设计稿 → S88.1 换图写点（扩展 apply + 校验/回滚/门禁）→ S88.2 资产选择器 UI →
S88.3 上传通道 → S88.4 预览联动 + 端到端实测 → S88.5 门禁/文档/收口。
