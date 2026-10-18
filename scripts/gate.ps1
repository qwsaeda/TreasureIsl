# gate.ps1 — 统一门禁（22 项，见 docs/parity.md）
# 用法：powershell -ExecutionPolicy Bypass -File scripts/gate.ps1
# 任一项 FAIL 即以非 0 退出；未到阶段的构建项打印 SKIP 并按通过处理。
$ErrorActionPreference = 'Continue'
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$script:fail = 0
function Report([bool]$ok, [string]$name, [string]$detail = '') {
    if ($ok) {
        Write-Host "  PASS  $name"
    } else {
        Write-Host "  FAIL  $name  $detail"
        $script:fail++
    }
}
function Skip([string]$name, [string]$why) {
    Write-Host "  SKIP  $name  ($why)"
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

Write-Host "=== gate: TreasureIsl (22 items) ==="

# 1) format
& cargo fmt --all -- --check 2>&1 | Out-Null
Report ($LASTEXITCODE -eq 0) 'cargo fmt --check'

# 2) lint
$lintOut = & cargo clippy --workspace --all-targets -- -D warnings 2>&1
Report ($LASTEXITCODE -eq 0) 'cargo clippy -D warnings'
if ($LASTEXITCODE -ne 0) { $lintOut | Select-Object -Last 30 | ForEach-Object { Write-Host "        $_" } }

# 3) test
$testOut = & cargo test --workspace 2>&1
Report ($LASTEXITCODE -eq 0) 'cargo test --workspace'
if ($LASTEXITCODE -ne 0) { $testOut | Select-Object -Last 30 | ForEach-Object { Write-Host "        $_" } }

# 4) build（范围随阶段解锁：native 起，wasm=S2.5，android=S2.6）
$nativeTarget = 'x86_64-pc-windows-msvc'
& cargo build --workspace --target $nativeTarget 2>&1 | Out-Null
Report ($LASTEXITCODE -eq 0) "cargo build ($nativeTarget)"
$installed = (@(& rustup target list --installed 2>$null) -join ' ')
if ($installed -match 'wasm32-unknown-unknown') {
    # 裸 build 不编 `required-features = ["wasm-shell"]` 的 treasure-wasm（第 4b 项注释的
    # 既有口径有误：第 4 项并不覆盖 wasm bin）。补 wasm 目标的 feature 门控 clippy
    # （wasm-shell：wasm bin + 全部集成测试在 wasm32 编译，EXP-104 盲区实证）。
    & cargo build --workspace --target wasm32-unknown-unknown 2>&1 | Out-Null
    $wasmBuildOk = ($LASTEXITCODE -eq 0)
    & cargo clippy --target wasm32-unknown-unknown -p ti-app --features wasm-shell --all-targets -- -D warnings 2>&1 | Out-Null
    $wasmClippyOk = ($LASTEXITCODE -eq 0)
    Report ($wasmBuildOk -and $wasmClippyOk) 'cargo build+clippy (wasm32-unknown-unknown; wasm-shell 门控 bin)'
} else {
    Skip 'cargo build (wasm32-unknown-unknown)' 'target 未安装，S2.5 解锁'
}
if ($installed -match 'aarch64-linux-android') {
    & cargo build -p ti-app --target aarch64-linux-android 2>&1 | Out-Null
    Report ($LASTEXITCODE -eq 0) 'cargo build (aarch64-linux-android)'
} else {
    # S2.6：APK 在 Mac（已装 Android NDK/SDK + cargo-apk）编译，本机 Windows 不产出。
    # 桌面/浏览器可玩（S5.1-S5.4）为当前收口目标；Android 为后续移动端交付，故 SKIP 不阻塞。
    Skip 'cargo build (aarch64-linux-android)' 'S2.6：APK 在 Mac 编译，本机无 NDK/SDK（用户既定，桌面优先）'
}

# 4b) feature 门控 bin 编译 + lint（EXP-099）
#     第 1~3 项不带 feature，`required-features = ["native-shell"/"headless"]` 的 bin
#     （treasure-native / treasure-headless / element-dump / sprite-step / render-png）
#     从不被编译与 lint —— 它们是**官方排障工具**，静默失效等于没有。
#     实证：element-dump 曾长期编译失败（4×E0308）而 gate 全绿。wasm bin（treasure-wasm）
#     由第 4 项 wasm 分支的 feature 门控 clippy 覆盖（EXP-104：wasm-shell 组合的
#     借用/截断错误 2026-10 实证漏网一次）；android bin 同 android 分支构建行覆盖。
$featOut = & cargo clippy --workspace --all-targets --features ti-app/native-shell,ti-app/headless -- -D warnings 2>&1
Report ($LASTEXITCODE -eq 0) 'clippy feature-gated bins (native-shell/headless; required-features 盲区)'
if ($LASTEXITCODE -ne 0) { $featOut | Select-Object -Last 30 | ForEach-Object { Write-Host "        $_" } }

# 5) 占位扫描（0 命中；tests/ 与 examples/ 除外 —— 集成测试 / demo 可执行体合法用 panic!/unwrap 做断言）
$srcDirsPh = @(Get-ChildItem -Path crates -Recurse -Filter *.rs -File |
    Where-Object { $_.FullName -notmatch '\\(tests|examples)\\' })
$phHits = @($srcDirsPh | Select-String -Pattern 'TODO|FIXME|unimplemented!|todo!|panic!\(')
$phList = ($phHits | ForEach-Object { "$($_.Path):$($_.LineNumber)" }) -join ', '
Report ($phHits.Count -eq 0) 'no placeholders (TODO/FIXME/todo!/unimplemented!/panic!) outside tests/+examples/' $phList

# 6) 库代码 unwrap/expect 扫描（排除集成测试目录 + demo examples 目录，0 命中）
$srcDirs = @(Get-ChildItem -Path crates -Recurse -Filter *.rs -File |
    Where-Object { $_.FullName -notmatch '\\(tests|examples)\\' })
$uw = @($srcDirs | Select-String -Pattern '\.unwrap\(\)|\.expect\(')
$uwList = ($uw | ForEach-Object { "$($_.Path):$($_.LineNumber)" }) -join ', '
Report ($uw.Count -eq 0) 'no unwrap()/expect() outside tests/+examples/ (库代码)' $uwList

# 7) 全格式 0 未知字节（S1 起有实现）
if (Test-Path 'crates/ti-tools/src/bin/verify.rs') {
    & cargo run -q -p ti-tools -- verify --zero-unknown 2>&1 | ForEach-Object { Write-Host "        $_" }
    Report ($LASTEXITCODE -eq 0) 'zero-unknown verify'
} else {
    Skip 'zero-unknown verify' 'S1.x 格式解析器就绪后解锁'
}

# 8) 台账校验（含 verified 的证据文件存在性）
$ledgerOut = & cargo run -q -p ti-tools --bin ledger -- plan/ledger.yaml 2>&1
$ledgerOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'ledger validate'

# 9) 文档一致性（doccheck 独立第 9 项，见 crates/ti-tools/src/doccheck.rs）
$docOut = & cargo run -q -p ti-tools --bin doccheck 2>&1
$docOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'doccheck (docs consistency)'

# 10) 数据表校验 + 硬编码扫描（S8.2，见 docs/process/04_tooling.md §2）
#     10a) validate_data 必须全绿：assets/data/*.json 结构与数值合法域
#          （FIND-201：统一入口 scripts/validate_data.py，已并入 legacy 基础校验
#           balance/strings/scenes/controls + V1/V2 场景标量兜底，旧 legacy 脚本作废）
$valOut = & python scripts/validate_data.py 2>&1
$valOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'validate_data (assets/data/*.json)'
#     10b) gen_layout --check：controls.json 与 legacy/ref/controls.json 派生产物一致（幂等）
$genOut = & python legacy/scripts/gen_layout.py --check 2>&1
$genOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'gen_layout --check (idempotent)'
#     10c) check_hardcode：严格模式（发现硬编码即 FAIL）。2026-10 已清零 37 处债务，
#          玩家可见中文全进 strings.json、布局色全走 COLOR_* 常量；诊断/CLI 经 EXEMPT 豁免。
$hcOut = & python scripts/check_hardcode.py 2>&1
$hcHits = @($hcOut | Where-Object { $_ -match 'hit\(s\)' })
$hcHits | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'check_hardcode (strict; 0 hits required)'

# 11) oracle 边界（S8.8）：shell 层 0 个 oracle_* 函数调用（oracle 仅 rules 内部参照物）
#     check_oracle_boundary.py 区分「oracle_* 函数调用=违规」vs「oracle::xxx::Type 类型重导出=允许」。
$orbOut = & python scripts/check_oracle_boundary.py --check 2>&1
$orbOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'check_oracle_boundary (shell 层禁 oracle_* 函数调用)'

# 13) 双 agent 审计固化断言（doccheck2）：文档↔代码/数据一致性（第 1~6 轮审计高频项）
#     19 项检查：audio ChannelState API / shell oracle_* 残留 / LCG hex / 据点阈值 / i18n 孤儿 /
#     gate 项数 / headless BattleView API / 飞轮回读 / 战斗玩家数值 / 飞轮脚本 /
#     触摸渲染正逆成对 / 场景 band 数 / 场景布局保真 / HD 精灵透明 /
#     map_slot 0 起基准（FIND-203）/ exhaust 双写对账 / 经验卷条数 /
#     运行时 legacy 坐标禁 / 基线镜像恒等（S15.6 B1 裁决③）
$dc2Out = & cargo run -q -p ti-tools --bin doccheck2 . 2>&1
$dc2Out | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'doccheck2 (audit-codified doc↔code/data consistency)'

# 14) 台账三方对账（防虚报，S10.0）：ledger verified ↔ evidence 文件 ↔ 代码符号
#     机器化拦截子代理「已 verified」虚报（S9.5 commands.rs 教训）
$lrOut = & python scripts/check_ledger_reality.py 2>&1
$lrOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'check_ledger_reality (台账三方对账：verified↔evidence↔代码符号)'

# 15) 业务闭环断言（S11）：业务文档零逆向引用 / 表现层零 oracle / 根目录清空 legacy 在位 /
#     oracle 桥接层与归档同源（docs/plan-s11-forward.md §1 闭环判据 1~4）
$fwdOut = & python scripts/check_forward_closed.py --check 2>&1
$fwdOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'check_forward_closed (业务闭环：business 零逆向引用 + legacy 归档就位)'

# 16) 模块并行前置（S11.4）：包白名单互斥（plan/packages.yaml）+ 契约锚点存在
#     （docs/contracts/*.md 校验锚点在 crates/ grep 到）+ 跨 crate 可见面只走契约模块
#     （docs/process/05_module_parallel.md §4 并行纪律 1/2 的机器判定）
$modOut = & python scripts/check_module_contracts.py --check 2>&1
$modOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'check_module_contracts (包白名单互斥 + 契约锚点 + 跨包可见面)'

# 17) 飞轮开轮必读脚本（EXP-070）：验证 scripts/flywheel_readme.ps1 能正常运行且输出完整
$fwOut = & powershell -ExecutionPolicy Bypass -File "scripts/flywheel_readme.ps1" -Check -Quiet 2>&1
$fwOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'flywheel-readme (开轮必读脚本完整性)'

# 18) 视觉回归基准图（EXP-072）：3 场景 headless 逐像素 diff，基准图存在且 0 diff
$vrOut = & powershell -ExecutionPolicy Bypass -File "scripts/visual_baseline.ps1" -Check 2>&1
$vrOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'visual-regression (3 场景 headless 基准图逐像素 diff)'

# 19) 渲染快照门禁（S13.6 契约 §5，docs/process/07_render_pipeline.md）：
#     7 场景（index/selectrole/selectmap/main/vs/msgs/saveload）× 纯代码渲染全帧逐像素基准 diff。
#     基准由现行代码生成（assets/render_baseline/，杜绝 EXP-074/075 自证循环）；
#     全帧 diff=0（EXP-074 背景像素掩已废弃，S15.4/15.5 重锚 1080 代码绘制基线）。
#     快照入口带 TTF 在位断言（S15.6 QA125：缺失字体即 panic，禁止豆腐基线静默入库）。
$rsOut = & cargo test -q -p ti-app --test render_snapshot 2>&1
$rsOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'render-snapshot (7 场景 headless 逐像素基准 diff，契约 §5)'

# 19.5) S14 正向渲染快照门禁（第三套，与第 19 项并列，docs/business/08_layout_design.md §4）：
#     7 场景纯代码渲染逐像素基准 diff。基准 assets/forward_baseline/ 是**业务重设计意图产物**
#     （§4 版面 + §2 配色 + 代码绘制元素），S14 正向开发目标态——立绘/菜单条/HUD/血条全由
#     代码绘制，不依赖逆向素材。S15.4 删 legacy 写帧层后 forward 开关已退役（S15.6 QA017，
#     两态帧恒等），forward_baseline 为 render_baseline 的**字节镜像**（doccheck2
#     baseline-mirror-identity 锁逐字节恒等），防某套被单独改动导致门禁静默分叉。
$frsOut = & cargo test -q -p ti-app --test render_snapshot_forward 2>&1
$frsOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'forward-render-snapshot (S14 正向纯代码渲染 7 场景逐像素基准 diff，§4 版面意图)'

# 20) 数据源门禁（S13.6 契约 §5，docs/process/07_render_pipeline.md）：
#     legacy/data/*.Map / *.pak 相对 HEAD 有改动 → 强制跑 render_snapshot（第 19 项），漂移即 FAIL。
#     无改动时 SKIP（gate 全绿前提）。
$dsOut = & python scripts/check_data_source.py 2>&1
$dsOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'check_data_source (数据源改动 → 渲染快照门禁)'

Write-Host "=== gate: $($script:fail) failure(s) ==="
if ($script:fail -gt 0) { exit 1 } else { exit 0 }
