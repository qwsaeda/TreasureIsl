# flywheel_readme.ps1 — 飞轮开轮必读清单（EXP-068 → EXP-070 脚本化）
#
# 用法：
#   powershell -ExecutionPolicy Bypass -File scripts/flywheel_readme.ps1 -Package P6
#   powershell -ExecutionPolicy Bypass -File scripts/flywheel_readme.ps1 -All
#   powershell -ExecutionPolicy Bypass -File scripts/flywheel_readme.ps1 -Check -Quiet
#
# 参数：
#   -Package P1|P2|P3|P4|P5|P6   指定包，读对应白名单 + 过滤本域 EXP
#   -All                          读所有包白名单（不过滤 EXP）
#   -Check                        只验证脚本能跑 + 输出完整，不打印
#   -Quiet                       只打印必读清单，不打印环境事实

param(
    [string]$Package = '',
    [switch]$All,
    [switch]$Check,
    [switch]$Quiet
)

$ErrorActionPreference = 'Continue'
$root = Split-Path -Parent $PSScriptRoot

# $root 现在是仓库根目录（scripts/ 的父目录）

# ── 包域 → EXP 关键词映射 ──────────────────────────────────────────────────
# 读 docs/experience/index.md，按行提取 EXP 号 + 标题，按包域关键词过滤
$pkgExpKeywords = @{
    'P1' = @('战斗', 'battle', 'combat', 'rules_battle')
    'P2' = @('经济', '贸易', '建造', 'trade', 'build', 'economy')
    'P3' = @('地图', '卡牌', 'map', 'cards', 'main_map', 'cards_draw')
    'P4' = @('菜单', '流程', '消息', 'menu', 'selection', 'messages', 'main_loop', 'render', 'flow')
    'P5' = @('存档', 'RNG', 'saveload', 'rng', 'persistence', 'save')
    'P6' = @('表现层', '触摸', '渲染', '场景', 'widgets', 'touch', 'scenes', 'menu_shell', 'game_app', 'platform', '几何', '坐标', '环境事实', '飞轮', '子代理', '会话')
    'D'  = @('数据', 'data', 'balance', 'strings', 'validate_data', 'json', 'i18n')
}

# ── 读 packages.yaml（纯 PowerShell 解析，无需 Python）──────────────────
function Read-PackagesYaml {
    param([string]$path)
    $text = Get-Content -Path $path -Raw -Encoding UTF8
    $packages = @()
    $current = $null
    $inFiles = $false
    $inTests = $false

    foreach ($line in ($text -split "`n")) {
        $trimmed = $line.TrimStart()

        if ($trimmed -match '^-\s+id:') {
            if ($current) { $packages += $current }
            $current = @{ id = $trimmed -replace '^-\s+id:\s*', ''; name = ''; files = @(); tests = @() }
            $inFiles = $false; $inTests = $false
        }
        elseif ($current) {
            if ($trimmed -match '^name:\s+"(.*)"') { $current['name'] = $Matches[1] }
            elseif ($trimmed -match '^files:\s*$') { $inFiles = $true; $inTests = $false }
            elseif ($trimmed -match '^tests:\s*$') { $inTests = $true; $inFiles = $false }
            elseif ($inFiles -and $trimmed -match '^-\s+(.+?)\s*$') { $current['files'] += $Matches[1] }
            elseif ($inTests -and $trimmed -match '^-\s+(.+?)\s*$') { $current['tests'] += $Matches[1] }
            elseif ($trimmed -match '^depends_on:') { $inFiles = $false; $inTests = $false }
            elseif ($trimmed -match '^status:') { $inFiles = $false; $inTests = $false }
        }
    }
    if ($current) { $packages += $current }
    return $packages
}

# ── 读 EXP index.md，提取 EXP 号 + 标题 + 域关键词 ────────────────────────
function Read-ExpIndex {
    param([string]$path)
    $text = Get-Content -Path $path -Raw -Encoding UTF8
    $exps = @()
    foreach ($line in ($text -split "`n")) {
        if ($line -match '\|\s*EXP-(\d+)\s*\|') {
            $num = $Matches[1]
            # 标题是最后一列（去掉行尾 |）
            $cols = $line -split '\|'
            $title = ''
            for ($j = $cols.Count - 1; $j -ge 0; $j--) {
                $c = $cols[$j].Trim()
                if ($c.Length -gt 0) { $title = $c; break }
            }
            if ($num -match '^\d+$' -and $title -ne '' -and $title -ne 'status' -and $title -notmatch '^\$') {
                $exps += @{ id = "EXP-$num"; title = $title; raw = $line }
            }
        }
    }
    return $exps
}

function Filter-Exps {
    param($exps, [string]$pkg, $pkgExpKeywords)
    if ($pkg -eq '' -or $pkg -eq 'All') { return $exps }
    $kws = $pkgExpKeywords[$pkg]
    if (-not $kws) { return $exps }
    $filtered = @()
    foreach ($e in $exps) {
        $haystack = ($e.raw) + ' ' + $e.title
        foreach ($kw in $kws) {
            if ($haystack -match [regex]::Escape($kw)) {
                $filtered += $e
                break
            }
        }
    }
    return $filtered
}

# ── 读 gate.ps1，提取项描述 ───────────────────────────────────────────────
function Read-GateItems {
    param([string]$path)
    $text = Get-Content -Path $path -Raw -Encoding UTF8
    $items = @()
    $itemNum = 0
    foreach ($line in ($text -split "`n")) {
        $trimmed = $line.TrimStart()
        if ($trimmed -match "^Report\b") {
            $itemNum++
            # 提取引号内的描述
            if ($trimmed -match "Report\s+\(.*\)\s+'(.+?)'") {
                $items += @{ n = $itemNum; desc = $Matches[1] }
            }
            elseif ($trimmed -match 'Report\s+\(.*\)\s+"(.+?)"') {
                $items += @{ n = $itemNum; desc = $Matches[1] }
            }
            else {
                $items += @{ n = $itemNum; desc = $trimmed }
            }
        }
    }
    return $items
}

# ── 读 doccheck2.rs，提取 fn check_* 函数名 ──────────────────────────────
function Read-Doccheck2Checks {
    param([string]$path)
    $text = Get-Content -Path $path -Raw -Encoding UTF8
    $checks = @()
    $i = 0
    foreach ($line in ($text -split "`n")) {
        if ($line -match '^\s*fn\s+(check_\w+)') {
            $i++
            $checks += @{ n = $i; name = $Matches[1] }
        }
    }
    return $checks
}

# ── 收集数据 ──────────────────────────────────────────────────────────────
$pkgPath  = Join-Path $root 'plan\packages.yaml'
$expPath  = Join-Path $root 'docs\experience\index.md'
$gatePath = Join-Path $root 'scripts\gate.ps1'
$dc2Path  = Join-Path $root 'crates\ti-tools\src\bin\doccheck2.rs'

$packages = Read-PackagesYaml $pkgPath
$exps     = Read-ExpIndex $expPath
$gateItems = Read-GateItems $gatePath
$dc2Checks = Read-Doccheck2Checks $dc2Path

$selectedPkgs = @()
if ($All) {
    $selectedPkgs = $packages | Where-Object { $_.id -match '^P\d' }
}
elseif ($Package) {
    $selectedPkgs = @($packages | Where-Object { $_.id -eq $Package })
}

$filteredExps = Filter-Exps $exps $Package $pkgExpKeywords

# ── 输出 ─────────────────────────────────────────────────────────────────
$output = @()
$output += "=== 飞轮开轮必读清单（EXP-068 → EXP-070 脚本化）==="
$output += ""
if (-not $Quiet) {
    $output += "--- 环境事实 ---"
    $workDir = $root
$output += "工作目录: $workDir"
    $output += "Shell: PowerShell 5.1（; 分隔，不用 &&）"
    $output += "crate 前缀: ti-（不是 ty-）"
    $output += "逆向产物: legacy/（rules/ 与 domain/ 层 0 oracle 引用）"
    $output += ""
}
$output += "--- 必读文档（6 条，逐条路径 + 一句话要点）---"
$output += "[1] docs/process/00_index.md — 分工表 + 标准工作流"
$output += "[2] docs/business/00_index.md — 业务唯一事实源"
$output += "[3] docs/process/01_architecture.md + 02_conventions.md — 分层 + 数据驱动 + 命名 + 错误 + 测试"
$output += "[4] docs/contracts/00_index.md + 涉及契约 c1/c2/c3 — 跨包公共面"
$output += "[5] docs/experience/index.md — EXP 总表（重点读本包域相关 EXP）"
$output += "[6] scripts/gate.ps1 + crates/ti-tools/src/bin/doccheck2.rs — 门禁项 + 断言清单"
$output += ""
$output += "--- 在途并行包白名单（读 plan/packages.yaml 机器生成）---"
if ($selectedPkgs.Count -eq 0) {
    foreach ($p in $packages) {
        $output += "$($p.id) $($p.name) = " + ($p.files -join ' ')
        if ($p.tests.Count -gt 0) { $output += "  tests: " + ($p.tests -join ', ') }
    }
} else {
    foreach ($p in $selectedPkgs) {
        $output += "$($p.id) $($p.name) = " + ($p.files -join ' ')
        if ($p.tests.Count -gt 0) { $output += "  tests: " + ($p.tests -join ', ') }
    }
}
$output += ""
$output += "--- 本域相关 EXP（按包域过滤，来自 docs/experience/index.md）---"
if ($Package -eq '' -and -not $All) {
    $output += "（未指定包，显示全部 EXP 条目）"
}
foreach ($e in $filteredExps) {
    $output += "  $($e.id): $($e.title)"
}
$output += ""
$output += "--- gate 项清单（当前 $($gateItems.Count) 项）---"
foreach ($g in $gateItems) {
    $output += "  $($g.n). $($g.desc)"
}
$output += ""
$output += "--- doccheck2 断言清单（当前 $($dc2Checks.Count) 项）---"
foreach ($c in $dc2Checks) {
    $output += "  $($c.n). $($c.name)"
}

$text = $output -join "`n"

if ($Check) {
    # 验证输出完整性
    $ok = $true
    $msg = @()
    if ($output -notcontains '[1] docs/process/00_index.md — 分工表 + 标准工作流') { $ok = $false; $msg += '缺 [1] 必读条目' }
    if ($gateItems.Count -lt 16) { $ok = $false; $msg += "gate 项仅 $($gateItems.Count)（要求 ≥16）" }
    if ($dc2Checks.Count -lt 15) { $ok = $false; $msg += "doccheck2 断言仅 $($dc2Checks.Count)（要求 ≥15）" }
    if ($packages.Count -lt 7) { $ok = $false; $msg += "packages.yaml 仅 $($packages.Count) 包（要求 ≥7）" }
    # 6 条必读关键词
    $required6 = @(
        'docs/process/00_index.md',
        'docs/business/00_index.md',
        'docs/process/01_architecture.md',
        'docs/contracts/00_index.md',
        'docs/experience/index.md',
        'scripts/gate.ps1'
    )
    foreach ($kw in $required6) {
        if ($text -notmatch [regex]::Escape($kw)) { $ok = $false; $msg += "缺关键词 $kw" }
    }
    if (-not $ok) {
        Write-Host "flywheel_readme -Check FAIL:"
        foreach ($m in $msg) { Write-Host "  - $m" }
        exit 1
    }
    Write-Host "flywheel_readme -Check PASS: 6 条必读 ✓, gate $($gateItems.Count) 项, doccheck2 $($dc2Checks.Count) 断言, packages $($packages.Count) 包"
    exit 0
}

Write-Host $text
