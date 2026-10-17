# 金银岛常用命令集合（按 docs/gamebook.md + AGENTS.md 路由表固化）
# 用法：powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 cmd [args...]
#       powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 -List
#
# 命令清单：
#   run-native       运行桌面版（默认开 HD 真彩，F4 切画质 / F1 缩放 / F2 滤波 / F3 边框）
#   run-headless     无头回归（tests/scripts/full_run.json 16 步）
#   run-wasm         构建 + 起本地 HTTP 预览浏览器版
#   smoke            native+wasm 两端冒烟
#   gate             全门禁（21 项）
#   test             全 workspace 测试
#   fmt              规范格式化
#   clippy           lint（-D warnings）
#   element-dump     元素级渲染验证（逐精灵落点/越界判定）
#   sprite-step      逐精灵渲染（EXP-076 用户确认流，输出 4x PNG）
#   build-wasm       构建浏览器产物到 web/
#   serve            起本地静态服务（预览 web/）
#   push             提交并推送（前置 gate 全绿 + 台账 verified）
#   ledger           台账校验
#   doccheck         文档一致性
#   doccheck2        审计固化断言
#   check-hardcode   硬编码扫描

$ErrorActionPreference = "Continue"
$Command = $args[0]
$ExtraArgs = @()
if ($args.Count -gt 1) { $ExtraArgs = $args[1..($args.Count - 1)] }
Set-Location -LiteralPath (Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path))

function Say([string]$msg) { Write-Host "  $msg" }
function RunCmd([string]$cmdline) {
    Write-Host ""
    Write-Host ">>> $cmdline" -ForegroundColor Cyan
    Invoke-Expression $cmdline
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  FAIL (exit $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

if (-not $Command -or $Command -eq "-List" -or $Command -eq "--help" -or $Command -eq "-h") {
    Write-Host "=== 金银岛命令集合（playbook）==="
    Write-Host ""
    Write-Host "用法: powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 cmd [args...]"
    Write-Host "  命令后跟额外实参（如 element-dump index 3 跑 index 场景记录 3）"
    Write-Host ""
    Write-Host "命令:"
    Write-Host "  run-native     运行桌面版（HD 真彩）"
    Write-Host "  run-headless   无头回归（full_run.json）"
    Write-Host "  run-wasm       构建+起本地 HTTP 预览浏览器版"
    Write-Host "  smoke          native+wasm 两端冒烟"
    Write-Host "  gate           全门禁（21 项）"
    Write-Host "  test           全 workspace 测试"
    Write-Host "  fmt            规范格式化"
    Write-Host "  clippy         lint（-D warnings）"
    Write-Host "  element-dump   元素级渲染验证（scene + record#）"
    Write-Host "  sprite-step    逐精灵渲染（scene + record#，输出 4x PNG）"
    Write-Host "  build-wasm     构建浏览器产物到 web/"
    Write-Host "  serve          起本地静态服务（web/）"
    Write-Host "  push           提交并推送（前置 gate 全绿 + 台账 verified）"
    Write-Host "  ledger         台账校验"
    Write-Host "  doccheck       文档一致性"
    Write-Host "  doccheck2      审计固化断言"
    Write-Host "  check-hardcode 硬编码扫描"
    exit 0
}

switch ($Command) {
    "run-native" {
        RunCmd "cargo run -q -p ti-app --features native-shell --bin treasure-native"
    }
    "run-headless" {
        RunCmd "cargo run -q -p ti-app --features headless --bin treasure-headless -- --script tests/scripts/full_run.json"
    }
    "run-wasm" {
        RunCmd "powershell -ExecutionPolicy Bypass -File scripts/build_wasm.ps1"
        RunCmd "cargo run -q -p ti-tools --bin serve -- --dir web --port 8080"
    }
    "smoke" {
        RunCmd "powershell -ExecutionPolicy Bypass -File scripts/smoke.ps1"
    }
    "gate" {
        RunCmd "powershell -ExecutionPolicy Bypass -File scripts/gate.ps1"
    }
    "test" {
        RunCmd "cargo test --workspace"
    }
    "fmt" {
        RunCmd "cargo fmt --all"
    }
    "clippy" {
        RunCmd "cargo clippy --workspace --all-targets -- -D warnings"
    }
    "element-dump" {
        # 参数：scene [record#]，额外实参透传
        $extra = ""
        if ($ExtraArgs.Count -gt 0) { $extra = $ExtraArgs -join " " }
        $full = ("element-dump " + $extra).Trim()
        RunCmd "cargo run -q -p ti-app --features native-shell --bin element-dump -- $full"
    }
    "sprite-step" {
        # 参数：scene record#
        $extra = ""
        if ($ExtraArgs.Count -gt 0) { $extra = $ExtraArgs -join " " }
        $full = ("sprite-step " + $extra).Trim()
        RunCmd "cargo run -q -p ti-app --features native-shell --bin sprite-step -- $full"
    }
    "build-wasm" {
        RunCmd "powershell -ExecutionPolicy Bypass -File scripts/build_wasm.ps1"
    }
    "serve" {
        RunCmd "cargo run -q -p ti-tools --bin serve -- --dir web --port 8080"
    }
    "push" {
        # 前置检查：gate 全绿 + 工作区无意外文件
        $g = (& powershell -ExecutionPolicy Bypass -File scripts/gate.ps1 2>&1 | Select-Object -Last 3)
        $g | ForEach-Object { Write-Host "  $_" }
        if ($g -notmatch "0 failure\(s\)") {
            Write-Host "  gate not green, push blocked (docs/git.md: unverified no push)" -ForegroundColor Red
            exit 1
        }
        $st = & git status --short
        if ($st.Count -gt 0) {
            Write-Host "  uncommitted changes, commit first" -ForegroundColor Yellow
            $st | ForEach-Object { Write-Host "    $_" }
            exit 1
        }
        RunCmd "git push -u origin master"
    }
    "ledger" {
        RunCmd "cargo run -q -p ti-tools --bin ledger -- plan/ledger.yaml"
    }
    "doccheck" {
        RunCmd "cargo run -q -p ti-tools --bin doccheck"
    }
    "doccheck2" {
        RunCmd "cargo run -q -p ti-tools --bin doccheck2"
    }
    "check-hardcode" {
        RunCmd "python scripts/check_hardcode.py"
    }
    default {
        Write-Host "未知命令: $Command" -ForegroundColor Red
        Write-Host "运行 -List 查看命令清单"
        exit 1
    }
}
