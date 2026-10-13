# visual_baseline.ps1 - visual regression baseline generation/verification (EXP-072)
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts/visual_baseline.ps1           # generate + verify
#   powershell -ExecutionPolicy Bypass -File scripts/visual_baseline.ps1 -Check    # verify only (gate item 18)
$ErrorActionPreference = 'Continue'
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$check = $false
foreach ($a in $args) { if ($a -eq '-Check') { $check = $true } }

$baselineDir = Join-Path $repoRoot 'assets\hd_baseline'
$scenes = @('Index', 'SelectRole', 'MainMap')

Write-Host "=== visual_baseline (EXP-072) ==="

if ($check) {
    foreach ($s in $scenes) {
        $p = Join-Path $baselineDir "$s.png"
        if (Test-Path $p) {
            Write-Host "  PRESENT  $s.png"
        } else {
            Write-Host "  MISSING  $s.png"
            Write-Host "  FAIL  baseline missing: $s.png (run visual_baseline.ps1 without -Check first)"
            exit 1
        }
    }
}

Write-Host "  running cargo test -p ti-app --test visual_regression ..."
& cargo test -p ti-app --test visual_regression 2>&1 | ForEach-Object { Write-Host "        $_" }

if ($LASTEXITCODE -ne 0) {
    Write-Host "  FAIL  cargo test failed (diff > 0 or compile error)"
    exit 1
}

foreach ($s in $scenes) {
    $p = Join-Path $baselineDir "$s.png"
    if (-not (Test-Path $p)) {
        Write-Host "  FAIL  $s.png still missing after test run"
        exit 1
    }
}

Write-Host "  PASS  visual-regression (3 scene baselines present + 0 diff)"
exit 0
