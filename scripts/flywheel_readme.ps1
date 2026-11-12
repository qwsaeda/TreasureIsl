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
            elseif ($inFiles -and $trimmed -match '^-\s+(.+)$') { $ff = $Matches[1].TrimEnd("`r").Trim(); if (-not ($ff.StartsWith('#'))) { $current['files'] += $ff } }
            elseif ($inTests -and $trimmed -match '^-\s+(.+)$') { $tt = $Matches[1].TrimEnd("`r").Trim(); if (-not ($tt.StartsWith('#'))) { $current['tests'] += $tt } }
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
    # 与 doccheck2 gate_item_actual_count **同口径**（S26.3 修正）：数**全部** `Report ` 调用
    # （trim 后开头，含 tier 条件块内缩进的 Report）按「行内首个引号串 = 项名」去重。
    # 旧实现只数顶格 `^Report`（漏掉条件块内缩进项）⇒ 12 vs 实际 28 误报，已废。
    $names = [System.Collections.Generic.List[string]]::new()
    foreach ($line in ($text -split "`n")) {
        $t = $line.TrimStart()
        if (-not $t.StartsWith('Report ')) { continue }
        $rest = $t.Substring('Report '.Length)
        $first = -1
        for ($i = 0; $i -lt $rest.Length; $i++) { if ($rest[$i] -eq [char]39 -or $rest[$i] -eq [char]34) { $first = $i; break } }
        $last = -1
        for ($i = $rest.Length - 1; $i -ge 0; $i--) { if ($rest[$i] -eq [char]39 -or $rest[$i] -eq [char]34) { $last = $i; break } }
        if ($first -ge 0 -and $last -gt $first + 1) {
            # 飞轮 R3 漂移②修正（2026-11-12）：按「项名前缀」（描述串首个 `（` 前段）去重。
            # 旧口径按完整描述串去重，而同一 gate 项的 PASS/FAIL 双分支描述串不同
            # （如 no-macroquad 的 `cargo tree -p ti-app 扫描` vs `共 N 行`）⇒ 同逻辑项被
            # 拆成 2 行（readme 第 20/21 项重复）。归一 = 取 `（` 前段（无 `（` = 整串），
            # 与 doccheck2 gate_item_actual_count「项名去重」同口径（gate 实为 36 个逻辑项名）。
            $name = $rest.Substring($first + 1, $last - $first - 1)
            # 与 doccheck2 gate_item_actual_count **同口径**：按完整项描述串去重。
            # 双分支项（cargo build / Android 真机 / check_data_driven / no-macroquad）的
            # PASS/FAIL 描述串各占 1 条完整名 ⇒ 去重后 gate = 36 逻辑项（权威基准 =
            # doccheck2 `gate_item_actual_count` + parity.md §108「Report 去重 = 36 项」）。
            if (-not $names.Contains($name)) { $names.Add($name) }
        }
    }
    $itemNum = 0
    foreach ($n in $names) { $itemNum++; $items += @{ n = $itemNum; desc = $n } }
    return $items
}

# ── 读 docs/rules/ 红线文件，按包域过滤（S51.0 第 7 条「开轮协作五步」+ docs/rules 注入）────
# 每个 docs/rules/<x>-rules.md frontmatter `paths:` glob 列表 → 按 -Package 过滤本域相关 1~3 个。
# 过滤口径 = 该红线文件 paths glob 的目录前缀 与 选定 P 包 files 的 crate 前缀 匹配。
function Read-RulesFiles {
    param([string]$RulesDir)
    $rules = @()
    if (-not (Test-Path $RulesDir)) { return $rules }
    # -Filter 对 *-rules.md 形态不可靠（实测只数到 3/5），改用 Where-Object 全量筛
    foreach ($f in Get-ChildItem -Path $RulesDir -File | Where-Object { $_.Name -match '-rules\.md$' }) {
        $content = Get-Content $f.FullName -Raw
        $lines = $content -split "`n"
        $inPaths = $false
        $globs = @()
        foreach ($ln in $lines) {
            $t = $ln.Trim()
            if ($t -eq 'paths:') { $inPaths = $true; continue }
            if ($inPaths -and $t.StartsWith('- ')) {
                $g = $t.Substring(2).Trim().Trim('"')
                if ($g.Length -ge 3) { $globs += $g }
            } elseif ($inPaths -and $t -eq '---') { $inPaths = $false }
        }
        $rules += @{ file = $f.Name; globs = $globs }
    }
    return $rules
}

# ── 读 doccheck2.rs 注册表 RUST_CHECKS + gate_data.json doccheck2_checks，提取实跑断言名 ──
# 飞轮 R3 漂移①修正（2026-11-12）：旧实现数所有 `fn check_` 定义（52，含 8 个未注册的
# 子 helper：check_anim_timing_literals/check_check_inventory_consistency/
# check_image_asset/check_inventory_ids/check_primitive_params/check_primitive_text/
# check_scene_required_pick_keys/check_tooling_contract_one/check_tooling_stray_scripts/
# check_tooling_tmp_dir），与实跑面（gate_data.json doccheck2_checks = 44，单一事实源）漂移。
# 新口径 = gate_data.json `doccheck2_checks[].id`（实跑 45 项，唯一事实源；
#   RUST_CHECKS 42 注册项 + doccheck2_grep_checks 2 项 = 44，飞轮 R3 N-A5 勘误旧注释「42」）：
#   与 `cargo run --bin doccheck2` 输出「across N audit-codified checks」对齐（N = gate_data.json 实测 45）
function Read-Doccheck2Checks {
    param([string]$dc2Path, [string]$gateDataPath)
    $checks = @()
    # 单一事实源 = assets/gate_data.json doccheck2_checks[].id（实跑断言面）
    if (Test-Path $gateDataPath) {
        $gd = Get-Content -Path $gateDataPath -Raw -Encoding UTF8 | ConvertFrom-Json
        if ($gd.doccheck2_checks) {
            foreach ($c in $gd.doccheck2_checks) {
                $checks += @{ n = ($checks.Count + 1); name = $c.id }
            }
        }
    }
    # 兜底：gate_data 读失败时退回 RUST_CHECKS 注册表（仍优于旧的 fn 全定义数）
    if ($checks.Count -eq 0 -and (Test-Path $dc2Path)) {
        $text = Get-Content -Path $dc2Path -Raw -Encoding UTF8
        $i = 0
        foreach ($line in ($text -split "`n")) {
            if ($line -match '^\s*\("[a-z0-9-]+",\s*(check_\w+)') {
                $i++
                $checks += @{ n = $i; name = $Matches[1] }
            }
        }
    }
    return $checks
}

# ── 收集数据 ──────────────────────────────────────────────────────────────
$pkgPath  = Join-Path $root 'plan\packages.yaml'
$expPath  = Join-Path $root 'docs\experience\index.md'
$gatePath = Join-Path $root 'scripts\gate.ps1'
$dc2Path  = Join-Path $root 'crates\ti-tools\src\bin\doccheck2.rs'
$gateDataPath = Join-Path $root 'assets/gate_data.json'  # S51.0+doccheck2 实跑面单一事实源（飞轮 R3 漂移①：gate_data.json doccheck2_checks=44）

$packages = Read-PackagesYaml $pkgPath
$exps     = Read-ExpIndex $expPath
$gateItems = Read-GateItems $gatePath
$dc2Checks = Read-Doccheck2Checks $dc2Path $gateDataPath

$selectedPkgs = @()
if ($All) {
    $selectedPkgs = $packages | Where-Object { $_.id -match '^P\d' }
}
elseif ($Package) {
    $selectedPkgs = @($packages | Where-Object { $_.id -eq $Package })
}

$filteredExps = Filter-Exps $exps $Package $pkgExpKeywords

$dc2Checks = Read-Doccheck2Checks $dc2Path $gateDataPath

# 红线文件（docs/rules/，S51.0 第 7 条「开轮协作五步」+ docs/rules 注入）
# 末尾是目录（无扩展名分隔），Linux 下 Join-Path 反斜杠会被当字面文件名，须显式正斜杠
$rulesDir = $root + '/docs/rules'
$allRules = Read-RulesFiles $rulesDir
# 按 -Package 过滤：P 包 files 的 **crate 前缀**（前 2 段 crates/<name>）匹配红线文件
# globs 里任一 glob 的 crate 前缀。P1~P5 files 全在 crates/ti-core/… ⇒ 命中 ti-core-rules.md；
# P6 files 在 crates/ti-app/… ⇒ 命中 ti-app-scenes-rules.md；ti-engine/scripts/assets 包域 = 0（
# 引擎/工具/资产域无 P 包持有，红线速查给全域开轮读，不挂 P 包过滤）。
# P 包 files 的 crate 前缀集合（避免 Where-Object 管道里 hashtable 按值传丢 files 键的坑；
# .Trim() 双向防 CRLF 的 \r 污染：$p.id 可能 "P1\r"、files 项可能 "crates/...\r"）
$pkgCratePrefixes = @()
if ($Package) {
    foreach ($p in $packages) {
        if (($p.id | Out-String).Trim() -eq $Package) {
            foreach ($ff in @($p.files)) {
                $ff = ($ff | Out-String).Trim()
                $ffParts = $ff -split '/'
                if ($ffParts.Count -ge 2) { $pkgCratePrefixes += ($ffParts[0] + '/' + $ffParts[1]) }
            }
        }
    }
}
$selectedRules = if ($Package) {
    $sel = @()
    foreach ($r in $allRules) {
        $match = $false
        foreach ($g in $r.globs) {
            $gParts = $g -split '/'
            if ($gParts.Count -ge 2) {
                $gCrate = $gParts[0] + '/' + $gParts[1]
                if ($pkgCratePrefixes -contains $gCrate) { $match = $true; break }
            }
        }
        if ($match) { $sel += $r }
    }
    $sel
} else {
    @($allRules)
}

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
$output += "--- 必读文档（7 条，逐条路径 + 一句话要点）---"
$output += "[1] docs/process/00_index.md — 分工表 + 标准工作流"
$output += "[2] docs/business/00_index.md — 业务唯一事实源"
$output += "[3] docs/process/01_architecture.md + 02_conventions.md — 分层 + 数据驱动 + 命名 + 错误 + 测试"
$output += "[4] docs/contracts/00_index.md + 涉及契约 c1/c2/c3 — 跨包公共面"
$output += "[5] docs/experience/index.md — EXP 总表（重点读本包域相关 EXP）"
$output += "[6] scripts/gate.ps1 + crates/ti-tools/src/bin/doccheck2.rs — 门禁项 + 断言清单"
$output += "[7] 开轮协作五步（03_templates.md §7：提问→方案→拍板→草稿→批准；只读角色豁免草稿/批准两步）"
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
$output += "--- 本域红线（docs/rules/，按包域 glob 过滤；CCGS paths frontmatter 模式，S51.0）---"
if ($selectedRules.Count -eq 0) {
    $output += "（无命中包域红线文件，改前查 docs/rules/ 5 个全）"
}
foreach ($r in $selectedRules) {
    $output += "  $($r.file)：paths = " + ($r.globs -join ', ')
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
    if ($gateItems.Count -lt 21) { $ok = $false; $msg += "gate 项仅 $($gateItems.Count)（要求 ≥21，与 gate.ps1 顶层固定项 21 对齐，doccheck2 gate_item_actual_count 同口径）" }
    if ($dc2Checks.Count -lt 15) { $ok = $false; $msg += "doccheck2 断言仅 $($dc2Checks.Count)（要求 ≥15）" }
    if ($packages.Count -lt 7) { $ok = $false; $msg += "packages.yaml 仅 $($packages.Count) 包（要求 ≥7）" }
    # 7 条必读关键词（S51.0 起含 [7] 开轮协作五步）
    $required7 = @(
        'docs/process/00_index.md',
        'docs/business/00_index.md',
        'docs/process/01_architecture.md',
        'docs/contracts/00_index.md',
        'docs/experience/index.md',
        'scripts/gate.ps1',
        '开轮协作五步'
    )
    foreach ($kw in $required7) {
        if ($text -notmatch [regex]::Escape($kw)) { $ok = $false; $msg += "缺关键词 $kw" }
    }
    if ($allRules.Count -lt 5) { $ok = $false; $msg += "docs/rules 红线仅 $($allRules.Count)（要求 ≥5）" }
    if (-not $ok) {
        Write-Host "flywheel_readme -Check FAIL:"
        foreach ($m in $msg) { Write-Host "  - $m" }
        exit 1
    }
    Write-Host "flywheel_readme -Check PASS: 7 条必读 ✓, gate $($gateItems.Count) 项, doccheck2 $($dc2Checks.Count) 断言, packages $($packages.Count) 包, rules $($allRules.Count) 红线"
    exit 0
}

Write-Host $text
