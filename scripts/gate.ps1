# gate.ps1 — 统一门禁（13 项，见 docs/parity.md）
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

Write-Host "=== gate: TreasureIsl (13 items) ==="

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
    & cargo build --workspace --target wasm32-unknown-unknown 2>&1 | Out-Null
    Report ($LASTEXITCODE -eq 0) 'cargo build (wasm32-unknown-unknown)'
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

# 5) 占位扫描（0 命中）
$ph = Scan -pattern 'TODO|FIXME|unimplemented!|todo!|panic!\(' -paths @('crates')
$phList = ($ph | ForEach-Object { "$($_.Path):$($_.LineNumber)" }) -join ', '
Report ($ph.Count -eq 0) 'no placeholders (TODO/FIXME/todo!/unimplemented!/panic!)' $phList

# 6) 库代码 unwrap/expect 扫描（排除集成测试目录，0 命中）
$srcDirs = @(Get-ChildItem -Path crates -Recurse -Filter *.rs -File |
    Where-Object { $_.FullName -notmatch '\\tests\\' })
$uw = @($srcDirs | Select-String -Pattern '\.unwrap\(\)|\.expect\(')
$uwList = ($uw | ForEach-Object { "$($_.Path):$($_.LineNumber)" }) -join ', '
Report ($uw.Count -eq 0) 'no unwrap()/expect() outside tests/' $uwList

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
$valOut = & python scripts/validate_data.py 2>&1
$valOut | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'validate_data (assets/data/*.json)'
#     10b) gen_layout --check：controls.json 与 ref/controls.json 派生产物一致（幂等）
$genOut = & python scripts/gen_layout.py --check 2>&1
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

# 12) 双 agent 审计固化断言（doccheck2）：文档↔代码/数据一致性（第 1~5 轮审计高频项）
#     6 项检查：audio ChannelState API 一致性 / shell 层 oracle_* 残留 / LCG hex / 据点阈值 / i18n 孤儿 key / release 门禁项数漂移
$dc2Out = & cargo run -q -p ti-tools --bin doccheck2 . 2>&1
$dc2Out | ForEach-Object { Write-Host "        $_" }
Report ($LASTEXITCODE -eq 0) 'doccheck2 (audit-codified doc↔code/data consistency)'

Write-Host "=== gate: $($script:fail) failure(s) ==="
if ($script:fail -gt 0) { exit 1 } else { exit 0 }
