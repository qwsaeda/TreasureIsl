# 经验库 · code

> 代码级教训：文档与实现不一致、类型转换陷阱。条目 ID 全局唯一（EXP-NNN），维护规则与全量索引见 docs/experience/index.mdindex.md。

### EXP-015 代码：clippy pedantic 反复触发的模式（workspace `-D warnings`）
- **`assert!(false, ...)`** → 触发 `assertions_on_constants`（`all` 组）。改用 `assert!(matches!(ev, Pattern { .. }))` + `if let Pattern { .. } = ev { ... }`，不留不可达分支。
- **`[T; N]` 栈数组 > 16384 字节** → `large_stack_arrays`（`all`/perf 组）。改用 `Box<[T]>`：`vec![0u16; N].into_boxed_slice()`。
- **`u32 as f32` / `i32 as f32`** → `cast_precision_loss`（`pedantic` 组）。改法：值已知且小时用 f32 字面量（`const LW: f32 = 176.0;`）；否则在函数上加 `#[allow(clippy::cast_precision_loss)]`。
- **`fn new() -> Self` 无 `Default`** → `new_without_default`（`pedantic`）。手动 `impl Default`（数组大时不能 derive，但手动 `Self::new()` 即可）。
- **`let start = off;` 宏展开后可能未用** → `unused_variables`。在 `let` 前加 `#[allow(unused_variables)]`。
- **`use super::*`** → `wildcard_imports`（`pedantic`）在本配置**未触发**（现有代码已有 glob import 且全绿），可继续用。
- 规约已写入 `docs/style.md` §5.10。


### EXP-030 代码：`present_framebuffer` 文档说用 Viewport 算 letterbox，实现却没接 Viewport（S2.5 补实测）
- 现象：浏览器侧 canvas 被拉到 1408×1760，但游戏画面只缩在左上角 168×160 px，右下大片纯黑。CDP 探针报 `css=1408px×1760px rect=1408×1760`（布局正常），PIL 算 blue-bg bbox = x0-168 y0-160。
- 根因：`crates/ti-platform/src/gfx.rs` 的 `present_framebuffer(fb: &Framebuf)` 文档注释明写「窗口比例差异由 `Viewport` 计算 letterbox / pillarbox 偏移」，但**实现里四个顶点写死 `(0,0)-(LW,LH)`，根本没接收 `Viewport` 参数**。`treasure_wasm.rs` 里 `shell.vp = Viewport::fit(...)` 算了却只用于 input 映射（`screen_to_logical`），从未传给 present。桌面 native 因 window 恰好 176×220、scale=1，问题被掩盖；浏览器 canvas 被拉伸后才暴露。
- 解法：`present_framebuffer(fb: &Framebuf, vp: &Viewport)`，四角改用 `vp.logical_to_screen(0,0)` / `vp.logical_to_screen(LW,LH)`。调用点 `treasure_wasm.rs` 传 `&self.vp`。只有一个调用点，签名变更零扩散。
- 验证：重建 wasm 重抓，blue bbox → x0-1392 y0-1416（≈全画布），white 文本 19,843→35,671。
- 教训：**文档注释与实现必须一致**——文档声称的契约（"由 Viewport 算偏移"）若实现没兑现，在 scale=1 的环境里不会暴露，换了尺寸就翻车。验收时别只看「画了什么」，要看「画在画布的哪个 bbox」。

### EXP-043 代码：双实现 + 差分对拍是冗余模式，公式/类型应单一源
- 现象：S3 全部 12 模块沿用「oracle 层（自由函数）+ rules 层（struct 方法）+ diff 测试（逐位对拍）」的三件套。其中 LCG 公式（`214_013 / 2_531_011 / 0x7fff`）在 `oracle/rng.rs` 与 `rules/rng.rs` 各写一份；FNV-1a 64 在 `ti-tools` 三处各写一份。RNG 模块收敛后（`GameRng` 复用 `RngState::step` 作公式单一源），证明：双实现的价值只在「结构不同（自由函数 vs struct 方法）」，公式/类型/常数是冗余的。
- 根因：loop.md §6.2 第 7 条把「oracle 与 impl 用同一批公开类型/常量，仅结构不同」写成规约，但实际 12 个模块里多数是「自由函数 vs struct 方法」两种写法各写一遍逻辑，而非「共享一份公式、两个封装」。业务可判定（逆向对齐）的项目里，公式抄两遍 = 两份都要维护，且容易漂移（S3.12 的 `msvc_lcg_reference` 手算错就是公式漂移的实例）。
- 解法：
  1. **公式单一源**：LCG/FNV 这类「业务可判定的纯函数」只写一份（放 rules 或独立模块），oracle 侧只保留「给定种子 → 确定序列」的差分锚点（`diff_rng` 仍保留，因 LCG 常数是逆向推测值需对拍）；
  2. **类型不重复**：枚举/结构体/常量放 rules，oracle 不重复定义；
  3. **diff 测试保留**：差分对拍本身有价值（验证 spec 直译 vs 业务实现逐位一致），但对比的是「oracle 自由函数」vs「rules struct 方法」，不是「公式 A」vs「公式 B」；
  4. **业务可判 → 成熟库优先**：工具类（哈希/随机/格式化）若业务可判定，直接用 std 或成熟 crate，不手写。
- 验证：RNG 收敛后 `cargo test --workspace` 全绿 + `gate.ps1` 0 failure；FNV-1a 收敛到 `ti_tools::hash` 后 `build_assets`/`pcx_ref` 测试全绿。
- 教训：**逆向对齐项目的双实现，对齐的是「行为」不是「公式」**。公式/类型/常数是单一源，结构（自由函数 vs struct）是双实现。不要为了「对拍」把公式抄两遍。



### EXP-044 代码：macroquad Texture2D 部分上传需 update_part，行级脏标记与之精确对应
- 现象：S4.3 脏矩形优化要求「未变更区域跳过 GPU 上传」，但 macroquad 0.4.16 的 Texture2D 公共 API 只有 from_rgba8（全量）与 update（全量），逐帧新建纹理导致静止画面仍有全帧 GPU 工作。
- 根因：macroquad 的 Texture2D 内部封装了 miniquad 纹理槽，update_from_bytes 全量上传；部分上传需走 update_part(image, x, y, w, h)，其最小粒度是「连续行带」而非任意像素矩形。
- 解法：
  1. Framebuf 用行级脏标记（220 个 bool）而非像素级位图，与 update_part 的行带粒度精确对应；
  2. FramePresenter 持有持久 Texture2D + RGBA8 缓存，脏帧只转换 dirty_row_range() 的行带，调 update_part 一次部分上传，未脏行保留 GPU 旧值；
  3. 无脏帧时 present() 整帧 return（零 GPU 工作），主循环可无条件调用；
  4. 全帧脏（first==0 && last==H-1）走 update_from_bytes，避免 Image 构造开销。
- 验证：cargo test -p ti-platform --test gfx_dirty 7/7（含跳帧计数断言）；桌面 stats 实测 uploads=0 skipped=60（空闲菜单 100% 跳帧）。
- 教训：macroquad 纹理 API 的「部分上传」能力藏在 update_part（需构造 Image），不在 from_rgba8/update 里；做脏区域优化时先确认目标引擎的部分上传粒度（行带 vs 像素），再选脏标记级别。

