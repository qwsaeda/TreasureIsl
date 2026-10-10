# TEST-AUDIT-FIX 流水线 — 一键运行
# 用法: powershell -ExecutionPolicy Bypass -File scripts\pipeline\run_pipeline.ps1

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location $root

Write-Host "=== TEST-AUDIT-FIX Pipeline ===" -ForegroundColor Cyan
Write-Host ""

# Step 1: Tester
Write-Host "[1/4] Tester: 批量测试..." -ForegroundColor Yellow
$testOutput = cargo test --workspace 2>&1
$testOutput | ForEach-Object { Write-Host $_ }

$failed = $testOutput | Select-String "FAILED|panicked" | Select-Object -First 10
if (-not $failed) {
    Write-Host ""
    Write-Host "=== 全绿，流水线终止 ===" -ForegroundColor Green
    exit 0
}

Write-Host ""
Write-Host "发现失败，进入审计阶段..." -ForegroundColor Red
Write-Host ""

# Step 2: Auditor-1
Write-Host "[2/4] Auditor-1: 根因分析..." -ForegroundColor Yellow
Write-Host "失败清单："
$failed | ForEach-Object { Write-Host "  - $_" }
Write-Host ""
Write-Host "请 Auditor-1 分析上述失败并给出修复方案。"
Write-Host ""

# Step 3: Auditor-2
Write-Host "[3/4] Auditor-2: 独立复核..." -ForegroundColor Yellow
Write-Host "请 Auditor-2 独立验证 Auditor-1 的方案。"
Write-Host ""

# Step 4: Fixer
Write-Host "[4/4] Fixer: 修复..." -ForegroundColor Yellow
Write-Host "请 Fixer 按复核方案修复代码。"
Write-Host ""

Write-Host "=== 流水线一轮完成，请重新运行直到全绿 ===" -ForegroundColor Cyan
