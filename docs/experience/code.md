# 经验库 · code

> 代码级教训：文档与实现不一致、类型转换陷阱。条目 ID 全局唯一（EXP-NNN），维护规则与全量索引见 docs/experience/index.md。

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


### EXP-051 工具：PowerShell `Set-Content` / `-Raw` 重写会破坏 UTF-8 源码（本轮踩 2 次）
- 现象：用 `Get-Content -Raw` + 字符串替换 + `Set-Content` 改 `.rs` / `.py` 文件后，中文注释变成乱码且**换行被合并**（`//! a?//! b?//! c?` 挤成一行）；`.py` 直接报 `Non-UTF-8 code starting with '\xe3' in file ... but no encoding declared`。
- 根因：Windows PowerShell 5.1 的 `Set-Content` 默认按系统 ANSI(GBK) 写文件，`-Encoding UTF8` 才写 UTF-8；而 `Get-Content -Raw` 读回时已按 ANSI 解码，中文往返即不可逆损坏。同一 `-Raw` + `-replace` 组合在改完再 `Set-Content -NoNewline` 时还会丢失编码声明。
- 解法：**改源码文件一律不用 PowerShell 文本替换**。三条路：
  1. 优先用编辑工具（Edit/Write），它按 UTF-8 无损读写；
  2. 需要脚本化批量替换时，用 `python -c` / 落盘 `.py` 且**显式 `encoding="utf-8"` 读写**（`io.open(p, encoding="utf-8")`）；
  3. 批量改文本时先落盘 `.py` 再 `python file.py`，别用 `python -c` 内联长脚本（引号转义会再坏一层）。
- 验证：本轮 S8.1 数据表加载层、S8.3.b 经济模块的多次编辑改用 Edit/Write 与 Python 落盘脚本后，gate 第 1 项 `cargo fmt --check` 与 clippy 全绿，文件中文注释完好。
- 教训：**PowerShell 只用来跑命令，不用来改源码**。CJK 仓库里这是高频陷阱（参见 EXP-050 的同类根因：中文路径 + Windows 工具链）。凡是「批量改 N 个文件」，第一反应应该是脚本 + 显式编码，而不是 shell 文本替换。

### EXP-052 流程：门禁扫描 `crates/**` 全量 `.rs`，`src/` 内联 `#[cfg(test)]` 的 `expect`/`panic!` 同样被拦（本轮踩 2 次）
- 现象：在 `crates/ti-core/src/data.rs` / `rules/trade.rs` 内联写 `#[cfg(test)] mod tests` 并用 `.expect(...)` 取值，跑 `gate.ps1` 第 5/6 项直接 FAIL（占位扫描 + unwrap 扫描），而 `cargo test` 全绿。
- 根因：`gate.ps1` 第 6 项的 `$srcDirs` 只排除路径含 `\tests\` 的**目录**，不区分 `src/*.rs` 文件内部的 `#[cfg(test)] mod tests`；第 5 项占位扫描同理扫全文 `panic!`。于是「src 内联测试」和「tests 目录集成测试」被同等对待。
- 解法：**业务单测一律放 `crates/<c>/tests/<name>.rs`**（`tests/` 目录不受第 5/6 项扫描约束），`src/` 只留加载/实现代码。需要断言时用 `match ... { Ok(v)=>v, Err(e)=>panic!("...") }` 也**不行**（`panic!` 仍被第 5 项拦）——正确写法是让夹具函数返回可断言的值，或把测试整体迁到 `tests/`。
- 验证：把 S8.1 数据表测试迁到 `crates/ti-core/tests/data_tables.rs`、S8.3.b 经济测试迁到 `tests/rules_trade.rs` + `tests/rules_build.rs` 后，gate 第 5/6 项恢复 PASS。
- 教训：`docs/parity.md` 第 6 项那句「内联 `#[cfg(test)]` 模块同样禁止 unwrap」是**文档已写明**的规则，踩坑原因是没先读门禁脚本。改代码前先 `Select-String` 看门禁到底扫什么，别凭直觉。

### EXP-053 代码：clippy pedantic 拦 `f64 as u32` 与 `Copy` 结构按引用传（balance 数值计算必踩）
- 现象：经济模块里 `u32::try_from(v.round())` 编译不过（`f64: TryFrom` 未实现）；改成 `as u32` 后 clippy `-D warnings` 报 `cast_possible_truncation` / `cast_sign_loss`；`Bank`（`Copy`）的 `total(&self)` 又报 `trivially_copy_pass_by_ref`。
- 根因：workspace 开了 `clippy::pedantic = warn` + `-D warnings`（`Cargo.toml` `[workspace.lints.clippy]`）。`as` 转换与「按引用传 `Copy` 小类型」都在 pedantic 里。
- 解法：数值换算统一走一个模块内私有 helper：
  ```rust
  #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
  fn f64_to_u32(v: f64) -> Option<u32> {
      if !v.is_finite() || v < 0.0 { return None; }
      Some(v.round().min(f64::from(u32::MAX)) as u32)
  }
  ```
  `#[allow]` 必须挂在 **函数/表达式所在 item** 上（挂 `let` 绑定无效）。`Copy` 结构的 `&self` 方法加 `#[allow(clippy::trivially_copy_pass_by_ref)]`。
- 验证：`cargo clippy --workspace --all-targets -- -D warnings` 0 warning；`gate.ps1` 第 2 项 PASS。
- 教训：balance.json 里的 `markup`/`atk_coef`/`bank_rate`/`event_chance` 都是 f64，凡是「乘/取整回 u32」的结算函数都会重复踩这个。抽成一个共享 helper（`rules/*.rs` 各自私有副本即可）比逐处 `#[allow]` 更省事。

### EXP-054 流程：固化脚本必须「可重复跑 + 幂等 + 有 --report/--check 干跑」，且要真的用来批量干活（S8 教训）
- 现象：S8 改造中反复出现「能脚本化的重复活手工做了」——硬编码中文手改 75 处、
  格式校验靠人眼、数值散落在 .rs。把坑固化成 `scripts/*.py` 后，又发现脚本本身
  写得糙（`check_hardcode.py` 误报 oracle/CLI、`migrate_strings.py` 首版 KEY_HINTS 太少、
  改写后不幂等），导致「写了脚本但没人真用」。
- 根因：两个独立问题——(1) 固化的脚本没有幂等/干跑（`--report`/`--check`）契约，
  跑一次改一次，没法在门禁里反复验证；(2) 写完脚本就停了，没有把它接进
  循环/门禁去驱动真正的批量改动，脚本成了「摆设」。
- 解法（三条，全部已落 `docs/process/04_tooling.md` §2/§5）：
  1. **固化脚本一律三契约**：幂等（连跑两次产物逐字节一致）+ `--report`（只列不写）
     + 退出码非 0（门禁可判 FAIL）。`gen_layout.py --check`、`check_hardcode.py --list`
     是这个范式的样板；
  2. **脚本要驱动改动，不只报告**：`migrate_strings.py --apply` 把命中 KEY_HINTS 的
     字面量批量迁到 `strings.json` + 改写调用点为 `sget(key)`，人只给一次映射表，
     硬编码命中数从 75 → 37（真实减少，非「报告了 75」）；
  3. **把脚本接进门禁**：gate 第 10 项已挂 `validate_data` / `gen_layout --check` /
     `check_hardcode --warn`，下次有人手改坏 JSON/控件表/硬编码会被机器拦。
- 验证：`python scripts/migrate_strings.py --report` 显示 0 待迁（幂等已迁完）；
  `gate.ps1` 10 项全绿；`check_hardcode --list` 分类命中（oracle/CLI 已豁免，只剩真待迁）。
- 教训：**「固化脚本」的验收不是脚本存在，而是它被门禁/循环真正消费**。
  写了脚本却停留在 `--warn` 报告、不驱动任何改动，等于没写。判断标准：
  这个脚本跑完后，仓库里的重复活是否真的变少了（命中数下降 / 文件被自动改写）。

### EXP-055 流程：`pub const` 表不能调非 const 函数（E0015），i18n 用 `key + 运行时解析` 解法
- 现象：`shell_ui::MENU` 是 `pub const &[MenuItem]`，`label` 字段想填
  `sget("menu.help")`（i18n 运行时查表），编译报 `E0015: cannot call non-const function in constants`。
  同时 `sget` 想返回 `&'static str`，但 strings.json 反序列化出的 `String` 不是 `'static`，
  报 `E0638: lifetime may not live long enough`。
- 根因：`const` 上下文不能调任何非 `const fn`（`OnceLock` 惰性展开都不是 const）；
  且 `serde` 反序列化出的 `String` 生命周期不等于 `&'static`。
- 解法（`crates/ti-app/src/i18n.rs` 最终形态，EXP-052 的落地）：
  1. `MenuItem.label` 改存 **i18n key**（`&'static str`，如 `"menu.help"`），不存中文文本；
     常量表保持纯 const（key + event_id）；
  2. 中文文本在**运行时**经 `sget(key)` 解析（`sget` 非 const，`table()` 用 `OnceLock`
     摊平一次）；`shell_ui::menu_labels()` 返回 `Vec<&'static str>` 供渲染层用；
  3. 缺失 key / 整表改坏 → `sget` 回 `key` 本身（`&'static str` 来自 `include_str!` 字符串池，
     生命周期安全，且不 panic、不静默假文本——符合门禁红线）。
- 验证：`cargo build` 通过；`tests/glyph_raster.rs` 经 `menu_labels()` 取中文后仍全绿
  （缺字回退『继』『续』断言保留）；`sget("scene.index.title")` 直接返 `&'static str`。
- 教训：**「数据化的常量表」正确做法是「常量存 key，运行时解析成值」**，而不是
  在 `const` 里调查表。i18n / 数值 / 布局表都要遵守这个形态。

### EXP-057 门禁脚本判「oracle 类型重导出 vs oracle_ 函数调用」须区分，shell 层禁函数调用（S8.8）
- 现象：S8.3.a–e 把 rules 数值迁 alance.json 后，game_app.rs 残留 8 行 use ti_core::oracle::*
  含 4 个 oracle_* 公式函数（boss_hp/battle_round/move/text_line）直调。手 grep 清完
  （S8.8），但若不进门禁，下次回归（有人重抄 oracle 公式）没人抓。
- 根因：oracle:: 前缀下**两类引用**性质不同——oracle::main_map::Dir（类型/枚举重导出，
  属几何/像素事实层，允许保留）vs oracle::main_map::oracle_move(（公式函数，属「推测」业务逻辑，
  S8 起禁）。粗 grep oracle:: 会把类型也判违规（误伤）；纯删 import 又会漏全限定调用
  	i_core::oracle::main_map::oracle_move(。
- 解法：scripts/check_oracle_boundary.py（gate 第 11 项）——只约束 shell 层
  （game_app.rs/widgets.rs/i18n.rs/scenes/*/ins/*），匹配 oracle_[a-z_0-9]+(
  函数调用（含全限定 + 排除 use/注释行）；类型路径（后不跟 (）不算违规。幂等只读
  （--check 非 0 判 FAIL / --report 恒 0 看明细，EXP-054 三契约）。
- 验证：python scripts/check_oracle_boundary.py --check 0 违规；gate.ps1 14 项全绿；
  全仓 grep shell 层 oracle_ 函数调用 = 0。
- 教训：**固化脚本的判定要按「引用语义」分层，不能按前缀一刀切**——oracle 的「类型/几何」
   是事实层（可留），「公式函数」是推测层（禁）。门禁脚本要能区分二者，否则要么误伤、要么漏放。

### EXP-066 表现层数值硬编码须走 balance 注入，但「壳层写死 player Unit」属 P1-P5 白名单外，审计留 unknown（S10.5）
- 现象：`game_app.rs::start_battle` 写死 `Unit { hp: 1000, atk: 150 }`，而
  `balance.json → combat.player` 已定义 `{hp:2000,atk:60}`（`combat.md` 数值单一事实源
  要求玩家属性也走 balance 注入，与 `boss_tiers` 同源）。S10.5 双 agent 审计反向扫描
  （docs/business 可测规则 vs rules/domain 实现）抓到这条数值漂移。
- 根因：S10.2 占位清零阶段只清了 8 处（markup/qty_max/boss_tier/卡片目录等），
  「玩家 Unit hp/atk」这处漏网——它藏在 `start_battle` 而非 `rules/` 内，check_hardcode
  与 check_oracle_boundary 都扫不到（前者只扫 CJK 面量与色常量，后者只扫 oracle_ 调用）。
- 分类（漂移三分类）：属「需人裁决」——修复要改 `game_app.rs` 的 `start_battle` 读
  `balance().combat.player`，但表现层数值注入契约归 P1-P5 白名单（已 verified 的
  rules/domain 层）。S10.5 白名单（doccheck2/experience/evidence）不含 rules/domain 与
  表现层 start_battle，故本轮**留 unknown 不强制修**，仅登记本条 + 写 evidence/S10.5.log。
- 解法（待集成轮）：`start_battle` 改 `Unit { hp: b.combat.player.hp, atk: b.combat.player.atk }`
  （读 balance 失败走 LoadError 硬报错，不 fallback 假值，与 S10.2 第 7 条同口径）；
  并给 `Balance`/`CombatCfg` 补 `player: PlayerUnit { hp, atk }` 解析字段。
- 验证：`grep "combat.player" crates/` 当前 0 命中（未接）；`balance.json` 已含该字段
  （validate_data 通过，说明数据侧就绪、代码侧未消费）。
- 教训：**「数值单一事实源」的红线不止 oracle 调用——壳层直接 new Unit 写死 hp/atk
   同样是漂移**，且这类「数据表已有字段但代码没读」的漂移 check_hardcode/oracle_boundary
   都抓不到，得靠双 agent 反向审计（文档可测规则 vs 实现）补洞。若同类高频，应固化为
   doccheck2 断言（如「balance.json 某 key 存在但 crates/ 无消费方」，需人裁决归属白名单后再固化）。

### EXP-067 跨层几何公式须正逆成对测试：逆变换(正变换(p))==p，单层公式自洽不算数（S10.5）
- 现象：主地图触摸命中 EXP-059 修复后，`iso_grid_cell_at`（逆变换）与
  `iso_x`/`iso_y`（正变换）各自公式正确，但两层几何约定不一致导致触摸点永远不中
  正确格子。S10.5 审计发现：单测只验证了「单层公式自洽」，未验证「正逆变换往返」。
- 根因：`terrain_atlas.rs` 的 `iso_projection_matches_f307` 测试只正向验证
  `iso_x`/`iso_y` 公式，未做 `grid_cell_at(iso_x(p), iso_y(p)) == p` 的逆验证；
  子代理修触摸命中时只对齐了渲染侧公式，未同步改触摸命中侧。
- 解法：跨层几何公式（rules 逻辑坐标 ↔ 渲染投影）须正逆成对测试——
  对任意格 `p`，断言 `逆变换(正变换(p)) == p`；单层公式自洽不算数。
  `iso_projection_matches_f307` 补逆验证后，触摸命中回归自动拦截。
- 验证：`cargo test --workspace` 中 `iso_projection_matches_f307` 含正向 + 往返断言。
- 教训：**几何公式跨层（rules/渲染/触摸）必须成对测试正逆变换往返**，
  单层自洽测试会漏掉「两层公式约定不一致」的漂移。子代理修跨层 bug 时
  必须同时改两侧 + 补往返测试，否则同一条 EXP 会二次踩坑。

### EXP-071 列表场景触摸命中带必须与渲染 `item_rect` 公式正逆成对（S11 审计 N3）
- 现象：`select_role` / `select_map` / `saveload` 的 `on_touch` 用 `touch.rs` 里
  各自硬编码的常量（`LIST_Y0` / `SLOT_Y0` / `SLOT_STEP` / `SLOT_H`）做命中判定，
  而渲染侧用 `ListLayout::item_rect(i)`（来自 `scenes.json`）算高亮位置。
  两套坐标独立维护，改 `scenes.json` 后触摸必然脱节（EXP-059 同类问题第三次出现）。
- 根因：`touch.rs` 的命中区常量与渲染公式无程序化绑定，子代理修渲染时不会
  同步改触摸常量；旧 API `list_index_at` / `slot_at` 已废弃但未删除，新代码
  误用旧常量。
- 解法：
  1. `touch.rs` 删除 `LIST_Y0` / `SLOT_Y0` / `SLOT_STEP` / `SLOT_H` 常量及
     `list_index_at` / `slot_at` 函数；
  2. 新增 `scene_list_bands(x0, x1, y0, step, item_h, count) -> Vec<RectF>` 和
     `slot_bands(y0, step, item_h, count) -> Vec<RectF>`，公式与
     `ListLayout::item_rect(i)` 严格互逆（正逆成对，EXP-067）；
  3. 新增 `scene_list_index_at(&[RectF], x, y)` / `slot_band_index_at(&[RectF], x, y)`
     判定命中带；
  4. 三个场景的 `on_touch` 改读 `ti_core::data::scenes()` 取 `ListLayout`，
     调 `scene_list_bands` / `slot_bands` 推导命中带，不再用常量；
  5. 单测 `scene_list_bands_match_item_rect` / `slot_bands_match_item_rect` 做
     正逆变换往返（渲染 `item_rect` 中心点必须命中对应带）；
  6. `doccheck2` 第 11 项 `check_touch_render_pair` 断言三个场景文件不含
     `list_index_at` / `slot_at(` 且含 `scene_list_bands` / `slot_bands`。
- 验证：`cargo test --workspace` 全绿（含 2 个新往返断言测试）；
  `doccheck2` 0 errors；`gate.ps1` 0 failures。
- 教训：**触摸命中几何必须从渲染同一 `ListLayout` 推导，禁止独立硬编码常量**；
  改 `scenes.json` 后触摸自动跟随，无需同步维护两套坐标。
  子代理修列表场景渲染/触摸时，必须先读 `touch.rs` 的 `scene_list_bands` /
  `slot_bands` 再动手，不得在场景文件里新建命中区常量。

### EXP-073 触摸/渲染 band 数必须 == 渲染行数（`Character::all().len()`），禁止硬编码（S11 飞轮第 3 轮 N8）
- 现象：`select_role.rs` `on_touch` 调 `scene_list_bands(..., 9)` 硬编码 9 条命中带，
  但渲染 `render` 只遍历 `Character::all()`（5 个角色，5 行）。触摸侧多出的第 6–9 条带
  无渲染对应，点落时 `from_index(5..=8) → None → unwrap_or(0)` 静默选中第 0 角色，
  违反 EXP-067 触摸/渲染同坐标系（第 3 次同类变体，EXP-059/EXP-071 之后）。
- 根因：`select_role.rs` 的 `ROLE_COUNT = 5` 常量已正确（与 `Character::all()` 一致），
  但 `on_touch` 写 band 数时误写了 9（疑似参照 `select_map` 的 3 或其他场景旧值），
  未复用 `ROLE_COUNT`。`scene_list_bands` 的 `count` 参数是自由 usize，编译期无约束
  阻止与渲染行数脱节。
- 解法：
  1. `select_role.rs` `on_touch` 的 band 数从硬编码 `9` 改为 `ROLE_COUNT as usize`
     （`ROLE_COUNT` 已声明 = 5，与 `Character::all().len()` 一致）；
  2. `doccheck2` 第 12 项 `check_scene_band_count` 断言 `select_role.rs` 的
     `scene_list_bands` 调用参数 == `Character::all().len()`（即 5），
     若 band 数走常量则检查常量值 == 5。
- 验证：`cargo build -p ti-app --target wasm32-unknown-unknown` 0 error；
  `cargo test --workspace` 全绿；`cargo clippy --workspace --all-targets -- -D warnings` 0 warning；
  `doccheck2` 第 12 项 PASS。
- 教训：**`scene_list_bands` 的 band 数必须 == 渲染循环 `Character::all().len()`，
  禁止硬编码**。band 数应走场景常量（如 `ROLE_COUNT`），不得写魔法数字。
  子代理修列表场景触摸时，必须先核对 band 数 == 渲染行数，不得凭印象填数。


### EXP-076 渲染管线契约 + 越界求交裁剪（贴图覆盖菜单根治，EXP-075 后续）

- **现象**：native 默认 RGB565 路（`QualityMode::Original`），index 场景报「贴图覆盖菜单 / 特定区域错位」。洋红框（EXP-075①）已修但界面仍有贴图压在菜单文字上。
- **根因**：`f1=111`（105×11 长条精灵）在 index.Map 有 3 条记录 x=185/191/207 越界（>176 屏宽）。原版 8-bit blit 例程 `sub_5EAA8` 源窗求交为空 → `sub_5D9C4` 早退**整条不画**（`evidence/audit_zorder.md` A3，高置信）；Rust `blit_sprite` 的 `ox = px_x.min(FB_W-1)` 把越界精灵**拉回屏幕右缘 175 列**画 1 列宽竖条 → 落在菜单区右缘 = 「贴图覆盖菜单」直接根因（`evidence/audit_zorder.md` B4，高置信）。z-order 本身无错（子代理坐实：原版按文件顺序 blit，非 f5 分档、非分帧，`evidence/audit_f5_zorder.md` 高置信）。
- **解法**（职责分离 + 契约固化）：
  1. 新增 `crates/ti-app/src/render_pipeline.rs`：`clip_rect(src_w, src_h, ox, oy) -> Option<ClipRect>` 纯函数（越界早退，6 单测）+ `TRANSPARENT_INDEX` 常量。
  2. `sprite_atlas.rs::blit_sprite`（RGB565）+ `blit_sprite_rgba`（RGBA）接入 `clip_rect`，越界 `None` 直接早退（修掉拉回 bug）。
  3. `docs/process/07_render_pipeline.md` 渲染契约（路径表/z-order 文件顺序/越界求交/PCX index 0 透明位 + §7 元素级验证法），登记进 `docs/README.md` + `AGENTS.md` 路由表。
  4. `tests/render_snapshot.rs` 7 场景逐像素基准 diff（gate 第 19 项，基准由修复后代码生成杜绝 EXP-074/075 自证循环）+ `scripts/check_data_source.py`（gate 第 20 项，数据源改动→快照门禁）。
  5. `element_dump` 长期诊断 bin（`Cargo.toml` 声明）：逐精灵 blit dump 落点/越界判定/像素贡献，定位「漂的是哪个元素」（契约 §7）。
- **验证**：`element_dump index` 显示记录 11/20/21（f1=111 越界条）= SKIP、贡献 0 像素；洋红残留 = 0；headless 全流程 16/16 OK；`render_snapshot` 7 场景 diff=0；gate 20 项全绿。
- **教训**：渲染链路关键行为（越界裁剪/透明位/z-order）散在多文件无契约 → 每轮瞎猜。**必须**：(a) 渲染行为提成纯函数可单测；(b) 契约文档登记进路由表，改渲染代码前必读；(c) 快照门禁锁「最终像素」+ 数据源门禁当场拦；(d) 修 bug 前先按 native 真实链路逐层定位（哪条 quality 路 + 越界/透明语义），不靠猜；(e) 报「特定区域错位」先用 `element_dump` 逐元素定位，别全帧 diff 盲猜。


