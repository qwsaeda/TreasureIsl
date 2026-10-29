# gate.ps1 — 统一门禁（29 项，见 docs/parity.md；S24.1 分档 -Fast/-Touch/full；S31.3 起第 22 项 = wasm 浏览器端 e2e）
# 用法：
#   powershell -ExecutionPolicy Bypass -File scripts/gate.ps1            # 全量（收口用；语义不变）
#   powershell -ExecutionPolicy Bypass -File scripts/gate.ps1 -Touch     # 开发期：跳过纯构建/grep 项
#   powershell -ExecutionPolicy Bypass -File scripts/gate.ps1 -Fast      # 开发期：只跑秒级项（语法+数据）
#
# 分层（2026-10-26 实测审计：全量 warm 156s，其中 99% 在 8 个 cargo 项上）：
#   keep = 秒级必跑（fmt 语法 / validate_data / gen_layout 幂等）
#   late = 改动完成时跑（clippy / 全量测试 / doccheck2 / GPU 像素项）
#   skip = 只在收口跑（多端构建 + 纯 grep 类断言）
# 每项墙钟随 PASS/FAIL 一起打印（门禁慢时直接看哪一项，不再反推）。
param([switch]$Fast, [switch]$Touch)

$ErrorActionPreference = 'Continue'
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

# Linux 兼容（docs/migration_linux.md §3）：非 Windows 无 `python`（只有 python3）。
# 29 项里所有脚本项经 `& $py` 调用；Windows 上 $py = python，行为不变。
$py = 'python'
if ($PSVersionTable.OS -notlike '*Windows*') { $py = 'python3' }
# Linux 迁移口径：cargo 在 ~/.cargo/bin（不在默认 PATH）→ 绝对路径；Windows 走 PATH 的 cargo。
$cargoCmd = if ($PSVersionTable.OS -notlike '*Windows*') {
    $cb = Join-Path $env:HOME '.cargo/bin/cargo'
    if (Test-Path $cb) { $cb } else { 'cargo' }
} else { 'cargo' }

$script:fail = 0
# 2026-10-26：门禁慢时能直接看出是哪一项（不靠反推）——
# `Report` 在每项命令**之后**调用，故「距上一次 Report 的间隔」= 本项实际耗时。
# 零侵入：不需要在 28 个门禁项调用点插计时器。
$script:lastMark = [DateTime]::Now
# Linux 迁移口径：cargo 在 ~/.cargo/bin（不在默认 PATH）→ 补全（Windows 无副作用）。
if ($env:USERPROFILE -or $env:HOME) {
    $cargoBin = if ($env:HOME) { Join-Path $env:HOME '.cargo/bin' } else { Join-Path $env:USERPROFILE '.cargo/bin' }
    if ((Test-Path $cargoBin) -and (($env:PATH -split ':') -notcontains $cargoBin)) {
        $env:PATH = "${cargoBin}:$($env:PATH)"
    }
}

function Report([bool]$ok, [string]$name, [string]$detail = '') {
    $dt = ([DateTime]::Now - $script:lastMark).TotalSeconds
    $script:lastMark = [DateTime]::Now
    if ($ok) {
        Write-Host ("  PASS  {0}  [{1:N1}s]" -f $name, $dt)
    } else {
        Write-Host ("  FAIL  {0}  [{1:N1}s]  {2}" -f $name, $dt, $detail)
        $script:fail++
    }
}
# 分层判定：full（默认）跑全部；-Touch 跑 keep+late；-Fast 只跑 keep。
$script:full = (-not $Fast) -and (-not $Touch)
function Tier([string]$t) {
    if ($script:full) { return $true }
    if ($t -eq 'keep') { return $true }
    if ($t -eq 'late') { return $Touch }
    return $false
}
function Skip([string]$name, [string]$why) {
    Write-Host "  SKIP  $name  ($why)"
    $script:lastMark = [DateTime]::Now
}
function Scan([string]$pattern, [string[]]$paths) {
    $hits = @()
    foreach ($p in $paths) {
        if (Test-Path $p) {
            $hits += @(Get-ChildItem -Path $p -Recurse -Filter *.rs -File -ErrorAction SilentlyContinue |
                Select-String -Pattern $pattern)
        }
    }
    , $hits
}

Write-Host "=== gate: TreasureIsl (29 items) ==="

# 1) format
& $cargoCmd fmt --all -- --check 2>&1 | Out-Null
Report ($LASTEXITCODE -eq 0) 'cargo fmt --check'

# 2) lint
if (Tier 'late') {
    $lintOut = & $cargoCmd clippy --workspace --all-targets -- -D warnings 2>&1
    Report ($LASTEXITCODE -eq 0) 'cargo clippy -D warnings'
    if ($LASTEXITCODE -ne 0) { $lintOut | Select-Object -Last 30 | ForEach-Object { Write-Host "        $_" } }
} else { Skip 'cargo clippy -D warnings' 'dev tier（-Touch/--fast）' }

# 3) test
if (Tier 'late') {
    $testOut = & $cargoCmd test --workspace 2>&1
    Report ($LASTEXITCODE -eq 0) 'cargo test --workspace'
    if ($LASTEXITCODE -ne 0) { $testOut | Select-Object -Last 30 | ForEach-Object { Write-Host "        $_" } }
} else { Skip 'cargo test --workspace' 'dev tier（-Touch/--fast）' }

# 4) build（范围随阶段解锁：native 起，wasm=S2.5，android=S2.6）
#    2026-10-26：去掉 `--target $nativeTarget` —— Windows 上 host triple 就是
#    x86_64-pc-windows-msvc，指定 `--target` 只会把产物挪到**独立目录**（实测
#    libserde 在两个 deps 目录各编一份）⇒ 每次依赖变更都要把整棵依赖树重编第二遍
#    （该项实测 2s(warm) / 71.8s(冷)）。构建语义不变（仍是本机原生构建）。
$nativeTarget = 'x86_64-pc-windows-msvc'
if (Tier 'skip') {
    & $cargoCmd build --workspace 2>&1 | Out-Null
    Report ($LASTEXITCODE -eq 0) "cargo build (native/$nativeTarget)"
} else { Skip 'cargo build (native)' 'dev tier（-Touch/--fast）' }
$installed = (@(& rustup target list --installed 2>$null) -join ' ')
if ($installed -match 'wasm32-unknown-unknown' -and (Tier 'skip')) {
    # S18.8a：**发货 wasm 入口 = ti-engine 的 Bevy 壳**（ti-app 的 macroquad wasm 壳已退役）。
    # 本项直接构建**交付产物本身**（release wasm）+ 对同一 bin 跑 clippy
    # （wasm 交付链此前从不被 lint：EXP-104 盲区在 Bevy 壳上重演 → 一并收口）。
    & $cargoCmd build --release --manifest-path crates/ti-engine/Cargo.toml --bin treasure-bevy-wasm --target wasm32-unknown-unknown 2>&1 | Out-Null
    $wasmBuildOk = ($LASTEXITCODE -eq 0)
    & $cargoCmd clippy --release --manifest-path crates/ti-engine/Cargo.toml --bin treasure-bevy-wasm --target wasm32-unknown-unknown -- -D warnings 2>&1 | Out-Null
    $wasmClippyOk = ($LASTEXITCODE -eq 0)
    # S18.8b：native 交付壳也纳入 clippy（此前只 lint wasm bin → native 侧旧伤长期没人看见，
    # 例：treasure_bevy.rs main() 101 行超限）。两个 bin 同源，native 的 cfg 分支要单独过一遍。
    & $cargoCmd clippy --release --manifest-path crates/ti-engine/Cargo.toml --bin treasure-bevy-native -- -D warnings 2>&1 | Out-Null
    $nativeClippyOk = ($LASTEXITCODE -eq 0)
    Report ($wasmBuildOk -and $wasmClippyOk -and $nativeClippyOk) 'cargo build+clippy (wasm32-unknown-unknown; ti-engine Bevy wasm 壳)'
} else {
    Skip 'cargo build (wasm32-unknown-unknown)' 'dev tier（-Touch/--fast）'
}
if (($installed -match 'aarch64-linux-android') -and (Tier 'skip')) {
    & $cargoCmd build -p ti-app --target aarch64-linux-android 2>&1 | Out-Null
    Report ($LASTEXITCODE -eq 0) 'cargo build (aarch64-linux-android)'
} else {
    # S2.6：APK 在 Mac（已装 Android NDK/SDK + cargo-apk）编译，本机 Windows 不产出。
    # 桌面/浏览器可玩（S5.1-S5.4）为当前收口目标；Android 为后续移动端交付，故 SKIP 不阻塞。
    Skip 'cargo build (aarch64-linux-android)' 'S2.6：APK 在 Mac 编译，本机无 NDK/SDK（用户既定，桌面优先）'
}

# 4b) feature 门控 bin 编译 + lint（EXP-099）
#     第 1~3 项不带 feature，`required-features = ["native-shell"/"headless"]` 的 bin
#     （treasure-headless / element-dump / sprite-step / render-png）
#     从不被编译与 lint —— 它们是**官方排障工具**，静默失效等于没有。
#     实证：element-dump 曾长期编译失败（4×E0308）而 gate 全绿。S18.8a 起 macroquad
#     发货壳（treasure-native / treasure-wasm）退役，wasm 交付壳 = ti-engine 的 Bevy 壳，
#     由第 4 项 wasm 分支构建 + clippy 覆盖；android bin 仍**不被任何行覆盖**
#     （android 分支构建行没开 `android-shell` feature → bin 被 cargo 跳过，
#      EXP-107 记录的潜在静默失效，android 壳随 S18.8d 退役）。
if (Tier 'skip') {
    $featOut = & $cargoCmd clippy --workspace --all-targets --features ti-app/native-shell,ti-app/headless -- -D warnings 2>&1
    Report ($LASTEXITCODE -eq 0) 'clippy feature-gated bins (native-shell/headless; required-features 盲区)'
    if ($LASTEXITCODE -ne 0) { $featOut | Select-Object -Last 30 | ForEach-Object { Write-Host "        $_" } }
} else { Skip 'clippy feature-gated bins' 'dev tier（-Touch/--fast）' }

# 5) 占位扫描（0 命中；tests/ 与 examples/ 除外 —— 集成测试 / demo 可执行体合法用 panic!/unwrap 做断言）
# Linux 兼容：正斜杠路径（docs/migration_linux.md §4），路径分隔用 (?i)[/\\] 双兼容
$srcDirsPh = @(Get-ChildItem -Path crates -Recurse -Filter *.rs -File |
    Where-Object { $_.FullName -notmatch '(?i)[/\\](tests|examples)[/\\]' })
$phHits = @($srcDirsPh | Select-String -Pattern 'TODO|FIXME|unimplemented!|todo!|panic!\(')
$phList = ($phHits | ForEach-Object { "$($_.Path):$($_.LineNumber)" }) -join ', '
Report ($phHits.Count -eq 0) 'no placeholders (TODO/FIXME/todo!/unimplemented!/panic!) outside tests/+examples/' $phList

# 6) 库代码 unwrap/expect 扫描（排除集成测试目录 + demo examples 目录，0 命中）
# Linux 兼容：路径分隔 (?i)[/\\] 双兼容（同上第 5 项）
$srcDirs = @(Get-ChildItem -Path crates -Recurse -Filter *.rs -File |
    Where-Object { $_.FullName -notmatch '(?i)[/\\](tests|examples)[/\\]' })
$uw = @($srcDirs | Select-String -Pattern '\.unwrap\(\)|\.expect\(')
$uwList = ($uw | ForEach-Object { "$($_.Path):$($_.LineNumber)" }) -join ', '
Report ($uw.Count -eq 0) 'no unwrap()/expect() outside tests/+examples/ (库代码)' $uwList

# 7) 全格式 0 未知字节（S1 起有实现）
if ((Test-Path 'crates/ti-tools/src/bin/verify.rs') -and (Tier 'late')) {
    & $cargoCmd run -q -p ti-tools -- verify --zero-unknown 2>&1 | ForEach-Object { Write-Host "        $_" }
    Report ($LASTEXITCODE -eq 0) 'zero-unknown verify'
} else {
    Skip 'zero-unknown verify' 'dev tier 或 S1.x 格式解析器就绪后解锁'
}

# 8) 台账校验（含 verified 的证据文件存在性）
if (Tier 'skip') {
    $ledgerOut = & $cargoCmd run -q -p ti-tools --bin ledger -- plan/ledger.yaml 2>&1
    $ledgerOut | ForEach-Object { Write-Host "        $_" }
    Report ($LASTEXITCODE -eq 0) 'ledger validate'
} else { Skip 'ledger validate' 'dev tier（-Touch/--fast）' }

# 9) 文档一致性（doccheck 独立第 9 项，见 crates/ti-tools/src/doccheck.rs）
if (Tier 'skip') {
    $docOut = & $cargoCmd run -q -p ti-tools --bin doccheck 2>&1
    $docOut | ForEach-Object { Write-Host "        $_" }
    Report ($LASTEXITCODE -eq 0) 'doccheck (docs consistency)'
} else { Skip 'doccheck (docs consistency)' 'dev tier（-Touch/--fast）' }

# 10) 数据表校验 + 硬编码扫描（S8.2，见 docs/process/04_tooling.md §2）
#     10a) validate_data 必须全绿：assets/data/*.json 结构与数值合法域
#          （FIND-201：统一入口 scripts/validate_data.py，已并入 legacy 基础校验
#           balance/strings/scenes/controls + V1/V2 场景标量兜底，旧 legacy 脚本作废）
$valOut = & $py scripts/validate_data.py 2>&1
$valOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'validate_data (assets/data/*.json)'
#     10b) gen_layout --check：controls.json 与 legacy/ref/controls.json 派生产物一致（幂等）
$genOut = & $py legacy/scripts/gen_layout.py --check 2>&1
$genOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'gen_layout --check (idempotent)'
#     10c) check_hardcode：严格模式（发现硬编码即 FAIL）。2026-10 已清零 37 处债务，
#          玩家可见中文全进 strings.json、布局色全走 COLOR_* 常量；诊断/CLI 经 EXEMPT 豁免。
if (Tier 'skip') {
    $hcOut = & $py scripts/check_hardcode.py 2>&1
    $hcHits = @($hcOut | Where-Object { $_ -match 'hit\(s\)' })
    $hcHits | ForEach-Object { Write-Host "        $_" }
    Report ($LASTEXITCODE -eq 0) 'check_hardcode (strict; 0 hits required)'
} else { Skip 'check_hardcode' 'dev tier（-Touch/--fast）' }

#     22) 数据分离门禁（S18.7：界面 = 数据；渲染层不认场景语义；通用原语闭集）
$ddOut = & $py scripts/check_data_driven.py 2>&1
$ddHits = @($ddOut | Where-Object { $_ -match 'ERROR' })
if ($ddHits.Count -gt 0) {
    $ddHits | ForEach-Object { Write-Host "        $_" }
    Report $false 'check_data_driven (generic primitives; renderer has no scene semantics)'
} else {
    Report $true 'check_data_driven (generic primitives; renderer has no scene semantics)'
}

# 10e) 代码标准（docs/process/16_code_standards.md）：CS-1~CS-6 机器检查 + 棘轮基线
#      （存量豁免在 scripts/process_ratchet.json::code_standards；新增违例 = 非 0 退出）。
#      快且全档执行（不加 Tier 门控）：-Fast/-Touch/full 都拦新增违例。
$csOut = & $py scripts/check_code_standards.py --check 2>&1
$csOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'check_code_standards (CS-1~6 代码标准 + 棘轮基线)'

# 11) oracle 边界（S8.8）：shell 层 0 个 oracle_* 函数调用（oracle 仅 rules 内部参照物）
#     check_oracle_boundary.py 区分「oracle_* 函数调用=违规」vs「oracle::xxx::Type 类型重导出=允许」。
$orbOut = & $py scripts/check_oracle_boundary.py --check 2>&1
$orbOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'check_oracle_boundary (shell 层禁 oracle_* 函数调用)'

# 13) 双 agent 审计固化断言（doccheck2）：文档↔代码/数据一致性（第 1~6 轮审计高频项）
#     19 项检查：audio ChannelState API / shell oracle_* 残留 / LCG hex / 据点阈值 / i18n 孤儿 /
#     gate 项数 / headless BattleView API / 飞轮回读 / 战斗玩家数值 / 飞轮脚本 /
#     触摸渲染正逆成对 / 场景 band 数 / 场景布局保真 / HD 精灵透明 /
#     map_slot 0 起基准（FIND-203）/ exhaust 双写对账 / 经验卷条数 /
#     运行时 legacy 坐标禁 / 基线镜像恒等（S15.6 B1 裁决③）
if (Tier 'late') {
    $dc2Out = & $cargoCmd run -q -p ti-tools --bin doccheck2 . 2>&1
    $dc2Out | ForEach-Object { Write-Host "        $_" }
    Report ($LASTEXITCODE -eq 0) 'doccheck2 (audit-codified doc↔code/data consistency)'
} else { Skip 'doccheck2' 'dev tier（-Touch/--fast）' }

# 14) 台账三方对账（防虚报，S10.0）：ledger verified ↔ evidence 文件 ↔ 代码符号
#     机器化拦截子代理「已 verified」虚报（S9.5 commands.rs 教训）
$lrOut = & $py scripts/check_ledger_reality.py 2>&1
$lrOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'check_ledger_reality (台账三方对账：verified↔evidence↔代码符号)'

# 15) 业务闭环断言（S11）：业务文档零逆向引用 / 表现层零 oracle / 根目录清空 legacy 在位 /
#     oracle 桥接层与归档同源（docs/plan-s11-forward.md §1 闭环判据 1~4）
$fwdOut = & $py scripts/check_forward_closed.py --check 2>&1
$fwdOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'check_forward_closed (业务闭环：business 零逆向引用 + legacy 归档就位)'

# 16) 模块并行前置（S11.4）：包白名单互斥（plan/packages.yaml）+ 契约锚点存在
#     （docs/contracts/*.md 校验锚点在 crates/ grep 到）+ 跨 crate 可见面只走契约模块
#     （docs/process/05_module_parallel.md §4 并行纪律 1/2 的机器判定）
if (Tier 'skip') {
    $modOut = & $py scripts/check_module_contracts.py --check 2>&1
    $modOut | ForEach-Object { Write-Host "        $_" }
    Report ($LASTEXITCODE -eq 0) 'check_module_contracts (包白名单互斥 + 契约锚点 + 跨包可见面)'
} else { Skip 'check_module_contracts' 'dev tier（-Touch/--fast）' }

# 17) 飞轮开轮必读脚本（EXP-070）：验证 scripts/flywheel_readme.ps1 能正常运行且输出完整
if (Tier 'skip') {
    $fwOut = & powershell -ExecutionPolicy Bypass -File "scripts/flywheel_readme.ps1" -Check -Quiet 2>&1
    $fwOut | ForEach-Object { Write-Host "        $_" }
    Report ($LASTEXITCODE -eq 0) 'flywheel-readme (开轮必读脚本完整性)'
} else { Skip 'flywheel-readme' 'dev tier（-Touch/--fast）' }




# 19) 渲染结构不变式门禁（S18.7b：取代三套像素基线 diff）：
#     7 场景 × 结构不变式断言（非黑覆盖率 ≥ 90% / 场景间内容签名互异 /
#     暖色板在位 / TTF 在位）——**不维护 golden PNG**（像素级微调会让基线频繁失效）。
if (Tier 'late') {
    $riOut = & $cargoCmd test -q -p ti-app --test render_invariants 2>&1
    $riOut | ForEach-Object { Write-Host "        $_" }
    Report ($LASTEXITCODE -eq 0) 'render-invariants (7 场景结构不变式断言，无像素基线)'
} else { Skip 'render-invariants' 'dev tier（-Touch/--fast）' }

# 20) 数据源门禁（S13.6 契约 §5，docs/process/07_render_pipeline.md）：
#     legacy/data/*.Map / *.pak 相对 HEAD 有改动 → 强制跑 render_invariants（第 19 项），不变式失败即 FAIL。
#     无改动时 SKIP（gate 全绿前提）。
$dsOut = & $py scripts/check_data_source.py 2>&1
$dsOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'check_data_source (数据源改动 → 渲染不变式门禁)'

# 22) 代码结构门禁（S20.0c，docs/style.md §5.12）：文件规模（>900 行）/ 函数长度（>140 行）/
#     重复模式（同一骨架 ≥6 次）；阈值与**棘轮基线** = scripts/code_budget.json
#     （新增违规 ⇒ FAIL；整改后从基线删除对应条目 ⇒ 只降不升）
if (Tier 'skip') {
    $auditOut = & $py scripts/audit_code.py --check 2>&1
    $auditOut | ForEach-Object { Write-Host "        $_" }
    Report ($LASTEXITCODE -eq 0) 'audit_code (代码结构：文件规模/函数长度/重复模式，棘轮基线)'
} else { Skip 'audit_code' 'dev tier（-Touch/--fast）' }

# 21) UI 场景叙事约束（v3，docs/ui-redesign-v3/02_spec_lang.md）：
#     assets/data/ui_schema/scene_manifest_v3.json 的 mood/重心/节奏/密度/HUD 上限断言
#     （N1-N8；scenes.json focus_anchor 键未落地时 N8 SKIP，v3.1 起全量）。
$snOut = & $py scripts/check_scene_narrative.py 2>&1
$snOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'check_scene_narrative (UI 场景叙事约束 DSL v3：重心+节奏+密度+HUD 上限)'

# 21.5) CPU↔Bevy 内存对拍（S18.16，**无 golden 文件**）：
#     ti-engine 独立 example（S18.2）真实 GPU 离屏渲染 vs `Framebuf` CPU 期望逐像素对照，
#     判定：通道差 ≤ 8 LSB 且超差比例 < 0.5%；退出码 0=通过 / 1=失败 / 2=无 GPU 读回链路（SKIP）。
if (Tier 'late') {
    $parityOut = & $cargoCmd run -q --manifest-path crates/ti-engine/Cargo.toml --example bevy_vs_cpu_pixel 2>&1
    $parityOut | ForEach-Object { Write-Host "        $_" }
    $parityExit = $LASTEXITCODE
    if ($parityExit -eq 2) {
        Write-Host "        SKIP: GPU 读回链路不可用（本机无 GPU 设备）"
    }
} else { $parityExit = 0; Skip 'bevy-cpu-parity' 'dev tier（-Touch/--fast）' }
# 顶格 Report（项数计量口径：只数第 0 列的 Report）→ 这项固定计入；
# 退出码 1（对照失败）= FAIL；2（无 GPU）= 记名 SKIP 但不失败（硬件不可验）。
Report ($parityExit -ne 1) 'bevy-cpu-parity (GPU/CPU 像素对照【仅游戏世界，UI 层已排除】：通道 ≤8 LSB 且超差 <0.5%；无 GPU → 记名 SKIP)'

# 22) wasm 浏览器端 e2e（S31.3，4 层判据：L1 实例化 / L2 渲染 / L3 交互 / L4 场景切换）：
#     依赖 serve 静态服务在跑 + Playwright Chromium + Pillow（14_tooling_catalog §3.2 登记）；
#     无 serve / 无 Playwright / 无 Pillow ⇒ 记名 SKIP（硬件/环境不可验，同 GPU 口径）；
#     有前置且断言失败 ⇒ FAIL（退出码非 0）。
$e2eOut = & $py scripts/wasm_browser_e2e.py --check 2>&1
$e2eOut | ForEach-Object { Write-Host "        $_" }
$e2eExit = $LASTEXITCODE
# 无 serve / 无 Playwright / 无 Pillow ⇒ 记名 SKIP（硬件/环境不可验，同 GPU bevy-cpu-parity 口径）；
# 有前置且断言失败 ⇒ FAIL。项数计量 = 机器口径（Read-GateItems 数全部 Report 去重项名，29 项）。
$e2eEnvMissing = $e2eOut -match "FAIL  本机无 playwright|ModuleNotFoundError"
if (-not $e2eEnvMissing) {
    Report ($e2eExit -eq 0) 'wasm-browser-e2e (wasm 浏览器端 4 层：L1 实例化 / L2 渲染 / L3 交互 / L4 select_map 鼠标点 m01 切 main_map)'
} else {
    Write-Host "        SKIP: 本机无 Playwright（pip install playwright pillow）"
    Skip 'wasm-browser-e2e' '本机无 Playwright 环境（pip install playwright pillow）'
}

Write-Host "=== gate: $($script:fail) failure(s) ==="
if ($script:fail -gt 0) { exit 1 } else { exit 0 }
