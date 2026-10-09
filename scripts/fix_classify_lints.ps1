# 一次性修复 classify.rs 的 clippy -D warnings 违例（gate 第 2 项）。
# 用法：powershell -ExecutionPolicy Bypass -File scripts/fix_classify_lints.ps1
$ErrorActionPreference = 'Stop'
$f = Join-Path $PSScriptRoot '..\crates\ti-tools\src\bin\classify.rs'
$t = [System.IO.File]::ReadAllText($f)
$enc = New-Object System.Text.UTF8Encoding($false)
$applied = 0

function Sub([string]$label, [string]$old, [string]$new) {
    if (-not $script:t.Contains($old)) { throw "MISS: $label" }
    $script:t = $script:t.Replace($old, $new)
    $script:applied++
    Write-Host "  ok  $label"
}

# ---------------------------------------------------------------- 1) api_category
# 文件可能是 LF 或 CRLF（git autocrlf），统一按实际内容匹配。
$nl = if ($t.Contains("`r`n")) { "`r`n" } else { "`n" }
Write-Host "  ..  newline style: $(if ($nl.Length -eq 2) { 'CRLF' } else { 'LF' })"

# 把 6 张表整体搬到模块级：先抽出每张 const 块
$tables = [ordered]@{}
$names = @('CRT', 'IO', 'WIN_CE', 'GFX', 'AUDIO', 'THREAD')
foreach ($n in $names) {
    $pat = "(?s)    const ${n}: &\[&str\] = &\[(.*?)\n    \];\n"
    $m = [regex]::Match($script:t, $pat)
    if (-not $m.Success) { throw "MISS const $n" }
    $tables[$n] = $m.Groups[1].Value
    $script:t = $script:t.Remove($m.Index, $m.Length)
    $script:applied++
}
Write-Host "  ok  hoisted $($names.Count) const tables out of api_category"

# 组装新的函数体
$bodies = @()
foreach ($n in $names) {
    $cat = switch ($n) {
        'CRT' { 'crt' }
        'IO' { 'io' }
        'WIN_CE' { 'win_ce' }
        'GFX' { 'gfx' }
        'AUDIO' { 'audio' }
        'THREAD' { 'thread' }
    }
    $bodies += "    (`"${n}_NAMES`, `"${cat}`"),"
}
$moduleConsts = @(
    '/// API 名表 → 类目的对照表（与 `scripts/classify.py` 的 `API_CATEGORY`/`GX_APIS` 对齐）。',
    'const API_TABLES: &[(&[&str], &str)] = &[',
    ($bodies -join $nl),
    '];',
    '',
    'const CRT_NAMES: &[&str] = &[' + $tables['CRT'] + $nl + '];',
    '',
    'const IO_NAMES: &[&str] = &[' + $tables['IO'] + $nl + '];',
    '',
    'const WIN_CE_NAMES: &[&str] = &[' + $tables['WIN_CE'] + $nl + '];',
    '',
    'const GFX_NAMES: &[&str] = &[' + $tables['GFX'] + $nl + '];',
    '',
    'const AUDIO_NAMES: &[&str] = &[' + $tables['AUDIO'] + $nl + '];',
    '',
    'const THREAD_NAMES: &[&str] = &[' + $tables['THREAD'] + $nl + '];',
    ''
) -join $nl

$newFn = @(
    'fn api_category(api: &str) -> Option<&\''static str> {',
    '    for (names, category) in API_TABLES {',
    '        if names.contains(&api) {',
    '            return Some(category);',
    '        }',
    '    }',
    '    if api == "rand" || api == "srand" {',
    '        return Some("game_logic");',
    '    }',
    '    None',
    '}',
    ''
) -join $nl

$sig = "fn api_category(api: &str) -> Option<" + "&'static str> {"
$start = $script:t.IndexOf($sig)
if ($start -lt 0) { throw 'MISS api_category signature' }
$end = $script:t.IndexOf("`n}", $start)
if ($end -lt 0) { throw 'MISS api_category end' }
$script:t = $script:t.Remove($start, $end + 2 - $start).Insert($start, $newFn.TrimEnd())
$script:applied++
Write-Host '  ok  rewrote api_category as table-driven'

# 把模块级 const 插到 api_category 的文档注释之前
$anchor = '/// 已知 API 名 → 类目'
Sub 'insert module consts' $anchor ($moduleConsts + $anchor)

# ---------------------------------------------------------------- 2) classify_fallback 的 CRT_NAMES（items_after_statements）
$pat2 = '(?s)    const CRT_NAMES: &\[&str\] = &\[(.*?)\n    \];\n'
$m2 = [regex]::Match($script:t, $pat2)
if (-not $m2.Success) { throw 'MISS classify_fallback CRT_NAMES' }
$script:t = $script:t.Remove($m2.Index, $m2.Length)
$script:applied++
Write-Host '  ok  removed duplicate CRT_NAMES from classify_fallback'

[System.IO.File]::WriteAllText($f, $script:t, $enc)
Write-Host "classify.rs: applied $applied edits"
