# S32.3 视觉签收留痕（硬规则 9 · 2026-10-30）

- 概念稿 v2 = **用户签收通过**（问答留痕：「v2 通过（推荐）」）
- 签收对象：
  - `evidence/console/ui-p0-sitesheet.png`（8 页拼图：概览/任务/证据/审计/设置/门禁占位/资产占位/设计稿页）
  - `evidence/console/{01_overview,02_tasks,03_evidence,04_audit,05_settings,06_gate_placeholder,07_assets_placeholder,08_design}.png`
- 签收范围：
  1. 平台视觉规范 = 游戏「航海夜·精修」13 色令牌（bg0/bg1/surface/bevel/stroke/gold/gold-warm/blue/text_hi/text_mid/text_lo/success/danger）
     + 8pt 圆角 + 栅格间距；对齐 `docs/process/10_ui_redesign.md` §3 设计令牌 v2（互引）；
  2. 页面结构 = 概览/任务/证据/审计/设置五页（P0）+ 门禁/资产/文档/构建四占位页（P1/P2）+ 设计稿查看页；
  3. **图片网页内直查**（用户 2026-10-30 要求：设计图在 web 页面直接可见 + 点击灯箱放大；证据/资产缩略图同）。
- 落码面 = `crates/ti-console/web/`（原生 JS + CSS，无 npm）；视觉规范随落码进 `web/theme.css`（单一事实源）。
- mockup 临时件 = `crates/ti-console/_mock/`（gitignored；验收后删除）。
