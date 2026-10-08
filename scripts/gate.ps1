# gate.ps1 — 统一门禁（8 项，见 docs/parity.md）
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

Write-Host "=== gate: TreasureIsl (8 items) ==="

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
    Skip 'cargo build (aarch64-linux-android)' 'target 未安装，S2.6 解锁'
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

Write-Host "=== gate: $($script:fail) failure(s) ==="
if ($script:fail -gt 0) { exit 1 } else { exit 0 }
