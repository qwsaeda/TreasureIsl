# PowerShell 入口脚本体检（语法自检 + UTF-8 BOM 校验 + CJK 解析冒烟）。
#
# 为什么有这个工具（docs/process/15_process_ledger.md 第 16 条流程）：
#   EXP-109 —— playbook.ps1 / smoke.ps1 缺 UTF-8 BOM ⇒ PowerShell 5.1 按 ANSI(GBK) 解析，
#   中文提示里的字节被误判成引号 ⇒ 整文件 ParserError；门禁不跑这两个入口 ⇒
#   「一键入口从未真正跑起来」长期无人发现。本工具把「所有 .ps1 入口可被 PS 5.1 解析」变成可判定项。
#
# 检查三项（可离线跑）：
#   ① UTF-8 BOM 存在（PS 5.1 解析含 CJK 的 .ps1 需要，EXP-109）；
#   ② 语法自检：调 [System.Management.Automation.Language.Parser]::ParseFile，无 ParserError；
#   ③ CJK 提示可解析：脚本含 CJK 时解析错误数须为 0（②③ 合并判定）。
#
# 用法：
#   powershell -ExecutionPolicy Bypass -File scripts/check_ps_scripts.ps1
#   powershell -ExecutionPolicy Bypass -File scripts/check_ps_scripts.ps1 -Check   # 工具契约别名，只读
#
# 退出码：0 = 全部通过；1 = 有脚本不合格（逐条列出）。
# 幂等：只读，不改任何文件。
param(
    [switch]$Check
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$scriptDir = Join-Path $root 'scripts'
$fail = 0

function Test-Script([System.IO.FileInfo]$file) {
    $bytes = [System.IO.File]::ReadAllBytes($file.FullName)
    $hasBom = ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF)
    $rel = $file.FullName.Substring($root.Length + 1)
    if (-not $hasBom) {
        Write-Host "  ERROR $rel : 缺 UTF-8 BOM（PS 5.1 解析 CJK 需要；EXP-109）" -ForegroundColor Red
        return $false
    }
    $tokens = $null
    $errors = $null
    $null = [System.Management.Automation.Language.Parser]::ParseFile(
        $file.FullName, [ref]$tokens, [ref]$errors)
    if ($errors -and $errors.Count -gt 0) {
        Write-Host "  ERROR $rel : 语法解析失败（$($errors.Count) 处 ParserError）" -ForegroundColor Red
        foreach ($e in $errors | Select-Object -First 3) {
            Write-Host "        行 $($e.Extent.StartLineNumber): $($e.Message)"
        }
        return $false
    }
    Write-Host "  PASS  $rel : BOM ok + 语法 ok"
    return $true
}

Write-Host "=== PowerShell 入口体检（$scriptDir）==="
$files = Get-ChildItem -LiteralPath $scriptDir -Filter *.ps1 -File | Sort-Object Name
if (-not $files) {
    Write-Host "  WARN scripts/ 下无 .ps1 入口"
}
foreach ($f in $files) {
    if (-not (Test-Script $f)) { $fail++ }
}
if ($fail -gt 0) {
    Write-Host "check_ps_scripts: $fail 个脚本不合格" -ForegroundColor Red
    exit 1
}
Write-Host "check_ps_scripts: PASS（$($files.Count) 个 .ps1 全部 BOM ok + 语法 ok）"
exit 0
