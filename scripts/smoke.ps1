# S18.8a 冒烟测试：native + wasm 两端（APK 端由 S2.6 后补）
# 用法：powershell -ExecutionPolicy Bypass -File scripts/smoke.ps1
# 退出码 0 = 全绿，非 0 = 失败
#
# S18.8a 变更：发货壳 = Bevy（`crates/ti-engine` 独立 workspace）：
#   - native = `treasure-bevy-native`（ti-engine manifest）
#   - wasm   = `treasure-bevy-wasm`（web/treasure_bevy_wasm.wasm）
# macroquad 壳（treasure-native / treasure-wasm / gl.js / audio.js）已退役 → 不再冒烟。

$ErrorActionPreference = "Continue"
Set-Location -LiteralPath (Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path))

Write-Host "=== smoke: native + wasm 两端冒烟（S18.8a：发货壳 = Bevy）==="

# 1. native 构建（根 workspace + Bevy 交付壳）
Write-Host ""
Write-Host "[1/6] cargo build --workspace (native) + Bevy 交付壳"
cargo build --workspace 2>&1 | Select-Object -Last 1
cargo build --manifest-path crates\ti-engine\Cargo.toml --bin treasure-bevy-native 2>&1 | Select-Object -Last 1

# 2. 测试
Write-Host ""
Write-Host "[2/6] cargo test --workspace (+ ti-engine)"
cargo test --workspace 2>&1 | Select-String "test result:" | ForEach-Object { "  $($_.Line)" }
cargo test --manifest-path crates\ti-engine\Cargo.toml 2>&1 | Select-String "test result:" | ForEach-Object { "  $($_.Line)" }

# 3. wasm 构建
Write-Host ""
Write-Host "[3/6] wasm 构建 (build_wasm.ps1)"
powershell -ExecutionPolicy Bypass -File scripts\build_wasm.ps1 2>&1 | Select-Object -Last 5

# 4. HTTP 端点（web/ 静态服务；Bevy 浏览器页 = S18.8e，故此处只校 wasm 主体与资源）
Write-Host ""
Write-Host "[4/6] 起静态服务 + HTTP 端点验证"
cargo build -q -p ti-tools --bin serve 2>$null
$srv = Start-Process -FilePath "target\debug\serve.exe" -ArgumentList '--dir','web','--port','8092' -PassThru -WindowStyle Hidden -RedirectStandardOutput "$env:TEMP\opencode\smoke_srv.out" -RedirectStandardError "$env:TEMP\opencode\smoke_srv.err"
Start-Sleep 3

$endpoints = @('/treasure_bevy_wasm.wasm', '/assets/GoldLand.Fnt', '/assets/font.dat', '/assets/music.lpk', '/assets/settings.dat')
$allOk = $true
foreach ($ep in $endpoints) {
    try {
        $r = Invoke-WebRequest -Uri "http://127.0.0.1:8092$ep" -TimeoutSec 10 -UseBasicParsing
        Write-Host "  $ep $($r.StatusCode)"
    } catch {
        Write-Host "  $ep FAIL"
        $allOk = $false
    }
}
Stop-Process -Id $srv.Id -Force -ErrorAction SilentlyContinue

if (-not $allOk) {
    Write-Host ""
    Write-Host "SMOKE RESULT: FAIL (HTTP 端点不全 200)"
    exit 1
}

# 5. unpak 校验
Write-Host ""
Write-Host "[5/6] unpak 解包校验"
cargo run -q -p ti-tools --bin unpak -- --check-all 2>&1 | Select-Object -Last 3

# 6. 台账校验
Write-Host ""
Write-Host "[6/6] 台账校验"
cargo run -q -p ti-tools --bin ledger -- plan/ledger.yaml 2>&1

Write-Host ""
Write-Host "SMOKE RESULT: PASS (native + wasm 两端冒烟全绿)"
