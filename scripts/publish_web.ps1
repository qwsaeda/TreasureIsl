# S16.3 一键发布 publish-web（一条命令全链路）：
#   gate 全绿前置 → build_wasm 重建 web/ → 本地 serve 静态断言（200 + content-type）
#   → 发布历史（sha256 + git short sha + 日期）入 release.md「发布」节
#   → git worktree 隔离推 gh-pages（不污染 master）→ 线上 URL 复跑 web_playtest。
#
# 口径（task S16.3 dod）：
#   - 全链路通过判据 = 本地静态断言全绿 + gh-pages 推送成功 + 线上 URL 复跑 web_playtest 全交互 0 失败。
#   - Pages 不可用（远端不可写 / 无凭据 / 无 gh）⇒ **实测定态降级**：把实测到的失败原因
#     写进 release.md「发布」节「已知限制」，**不静默绕过**；本地产物 + 静态断言 + 发布历史照写。
#     降级是「记录在案的既成事实」，不是掩码 —— 门禁/文档都能看到这一段。
#   - 硬失败（应 exit 1）= gate 非全绿 / build_wasm 失败 / 本地静态断言失败 / web 产物缺失。
#
# 用法：
#   pwsh -File scripts/publish_web.ps1          # 全链路（Pages 不可写 = 降级实测定态）
#   pwsh -File scripts/publish_web.ps1 -Check   # 只读前置体检（不构建/不推/不写 release.md）
#
# 退出码：0 = 全链路通过或已记录降级；1 = 硬失败（gate/构建/静态断言）。
# 幂等：同一 git short sha 重复跑 = 发布历史不重复追加；worktree 每次重建后清理。

param([switch]$Check)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot
$webOut = Join-Path $repoRoot 'web'
$releaseMd = Join-Path $repoRoot 'docs/release.md'

function Say([string]$m) { Write-Host "  $m" }
function Fail([string]$m) { Write-Host "FAIL  $m" -ForegroundColor Red; exit 1 }

# docs/release.md 是纯 UTF-8（EXP-051/091：编辑 CJK 内容禁 Set-Content，防 BOM/编码漂移）
function Write-TextNoBom([string]$path, [string]$text) {
    $enc = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($path, $text, $enc)
}

# cargo 实际 target 目录（`.cargo/config.toml` 可能改到仓外；不硬编码 target/）
function Get-TargetDir {
    $meta = $null
    try { $meta = (& cargo metadata --no-deps --format-version 1 2>$null | ConvertFrom-Json) } catch { $meta = $null }
    if ($meta -and $meta.target_directory) { return $meta.target_directory }
    return (Join-Path $repoRoot 'target')
}

$ts = Get-Date -Format 'yyyy-MM-dd HH:mm'
$gitShort = (& git rev-parse --short HEAD).Trim()
$remoteUrl = ''
try { $remoteUrl = ((& git remote get-url origin 2>$null) | Select-Object -First 1) } catch { $remoteUrl = '' }
if ($remoteUrl) { $remoteUrl = $remoteUrl.Trim() }
$ghCli = [bool](Get-Command gh -ErrorAction SilentlyContinue)

# ── 0) 只读前置体检（不构建 / 不推送 / 不写 release.md）──
if ($Check) {
    $ok = $true
    foreach ($f in @('scripts/build_wasm.ps1', 'scripts/web_playtest.cjs', 'docs/release.md', 'assets/data/scenes.json')) {
        if (-not (Test-Path $f)) { Say "MISSING $f"; $ok = $false }
    }
    foreach ($cmd in @('git', 'cargo')) {
        if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) { Say "FAIL  $cmd 不在 PATH"; $ok = $false }
    }
    if (-not (Get-Command node -ErrorAction SilentlyContinue)) { Say "WARN  node 缺失（线上复跑 web_playtest 将降级）" }
    if (-not $remoteUrl) { Say 'WARN  无 origin 远端（Pages 段将降级）' }
    if (-not $ghCli) { Say 'WARN  gh CLI 缺失（Pages 建站段降级；推送仍走 git）' }
    Say ("target dir = " + (Get-TargetDir))
    Write-Host ('[publish-web -Check] ' + $(if ($ok) { 'PASS（只读；未构建/未推送/未写 release.md）' } else { 'FAIL' }))
    exit $(if ($ok) { 0 } else { 1 })
}

# ── 1) gate 全绿前置（发布物必须先过门禁；只跑一次，保留原脚本退出码）──
Say 'gate  全绿前置（scripts/gate.ps1）…'
& pwsh -File scripts/gate.ps1 > $null 2>&1
$gateExit = $LASTEXITCODE
if ($gateExit -ne 0) { Fail "gate 非全绿（exit $gateExit），拒绝发布" }
Say 'gate  PASS'

# ── 2) build_wasm 重建 web/ + 取产物 sha256 ──
Say 'build build_wasm.ps1 → web/ …'
& pwsh -File scripts/build_wasm.ps1 > $null 2>&1
if ($LASTEXITCODE -ne 0) { Fail 'build_wasm 失败' }
$wasmFile = Get-ChildItem $webOut -Filter '*_bg.wasm' -File |
    Sort-Object Length -Descending | Select-Object -First 1
if (-not $wasmFile) { $wasmFile = Get-ChildItem $webOut -Filter '*.wasm' -File | Select-Object -First 1 }
if (-not $wasmFile) { Fail "web/ 下无 .wasm 产物（未找到发布物）" }
$sha = (Get-FileHash $wasmFile.FullName -Algorithm SHA256).Hash
Say "wasm  $($wasmFile.Name) = $((Get-Item $wasmFile.FullName).Length) B  sha256=$sha"

# ── 3) 本地 serve 静态断言（200 + content-type）──
$port = 8137
$serveBin = Join-Path (Join-Path (Get-TargetDir) 'debug') 'serve'
if (-not (Test-Path $serveBin)) { $serveBin = $serveBin + '.exe' }
if (-not (Test-Path $serveBin)) { Fail "serve 二进制缺失（$serveBin；先 cargo build -p ti-tools --bin serve）" }
$proc = Start-Process -FilePath $serveBin -ArgumentList @('--dir', $webOut, '--port', "$port") -PassThru -NoNewWindow
Start-Sleep -Seconds 2
$expect = @{
    '/'                         = 'text/html'
    '/main.js'                  = 'javascript'
    '/treasure-bevy-wasm.js'    = 'javascript'
    '/treasure_bevy_wasm.wasm'  = 'application/wasm'
}
$localOk = $true
try {
    foreach ($path in $expect.Keys | Sort-Object) {
        try {
            $r = Invoke-WebRequest -Uri "http://127.0.0.1:$port$path" -UseBasicParsing -TimeoutSec 30
            $ct = [string]$r.Headers['Content-Type']
            if ($r.StatusCode -eq 200 -and $ct -like "*$($expect[$path])*") {
                Say ("static 200 " + $path + "  (" + $ct + ")")
            } else {
                Say ("static FAIL " + $r.StatusCode + " " + $path + "  (" + $ct + ")"); $localOk = $false
            }
        } catch {
            Say ("static FAIL " + $path + " : " + $_.Exception.Message); $localOk = $false
        }
    }
} finally {
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
}
if (-not $localOk) { Fail '本地静态断言未全绿（产物与 MIME 必须全对，否则浏览器加载必失败）' }

# ── 4) gh-pages 推送（git worktree 隔离，不污染 master）──
$pagesStatus = 'DEGRADED'
$pagesDetail = ''
if (-not $remoteUrl) {
    $pagesStatus = 'SKIP'
    $pagesDetail = '无 origin 远端'
} else {
    $wtDir = Join-Path $repoRoot '.git-worktrees/gh-pages'
    & git worktree remove $wtDir --force 2>&1 | Out-Null
    Remove-Item $wtDir -Recurse -Force -ErrorAction SilentlyContinue
    & git worktree add --detach $wtDir 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        $pagesStatus = 'DEGRADED'
        $pagesDetail = 'git worktree add 失败（见 git 报错）'
    } else {
        # 工作树清空 → 铺 web/ 产物 → 提交 → 推到 gh-pages（detached HEAD，不留本地分支）
        Get-ChildItem -LiteralPath $wtDir -Force |
            Where-Object { $_.Name -ne '.git' } | Remove-Item -Recurse -Force
        Copy-Item -Path (Join-Path $webOut '*') -Destination $wtDir -Recurse -Force
        Write-TextNoBom (Join-Path $wtDir '.nojekyll') ''
        & git -C $wtDir add -A 2>&1 | Out-Null
        & git -C $wtDir -c user.name=publish-web -c user.email=publish-web@local `
            commit -m "pages: publish web @ $gitShort ($sha)" 2>&1 | Out-Null
        $env:GIT_TERMINAL_PROMPT = '0'
        $pushOut = (& git -C $wtDir push origin 'HEAD:refs/heads/gh-pages' 2>&1) -join "`n"
        $pushExit = $LASTEXITCODE
        & git worktree remove $wtDir --force 2>&1 | Out-Null
        Remove-Item (Split-Path -Parent $wtDir) -Force -ErrorAction SilentlyContinue
        if ($pushExit -eq 0) {
            $pagesStatus = 'PUSHED'
            # 线上 URL 复跑 web_playtest（dod：线上全交互 0 失败）
            $onlineUrl = ''
            if ($remoteUrl -match 'github\.com[:/]([^/]+)/') {
                $onlineUrl = "https://$($Matches[1]).github.io/TreasureIsl/"
            }
            if ($onlineUrl -and (Get-Command node -ErrorAction SilentlyContinue)) {
                Say "pages 线上复跑 web_playtest @ $onlineUrl …"
                $onlineOut = (& node scripts/web_playtest.cjs --url $onlineUrl 2>&1) -join "`n"
                $onlineOk = ($LASTEXITCODE -eq 0)
                if ($onlineOk) { $pagesStatus = 'PUSHED + 线上 web_playtest PASS' } else {
                    $pagesStatus = 'PUSHED + 线上 web_playtest FAIL'
                    $pagesDetail = ($onlineOut -split "`n" | Select-Object -Last 1)
                }
            } else {
                $pagesStatus = 'PUSHED（线上 URL 未能推导或 node 缺失，复跑降级）'
            }
        } else {
            # 实测定态：把 git 的原话记下来（不静默绕过）
            $pagesDetail = (($pushOut -split "`n") | Where-Object { $_ -match '\S' } | Select-Object -First 1)
            if (-not $pagesDetail) { $pagesDetail = "git push exit $pushExit" }
        }
    }
}
Say "pages $pagesStatus  $pagesDetail"

# ── 5) 发布历史入 release.md「发布」节（幂等：同 git short sha 不重复追加）──
$histRow = "| $ts | $gitShort | $sha | web/（$($wasmFile.Name)） | $pagesStatus |"
$md = [System.IO.File]::ReadAllText($releaseMd)
$nl = "`n"
if ($md -notmatch "## 发布") {
    $md = $md.TrimEnd() + $nl + $nl + '## 发布（publish-web，S16.3）' + $nl + $nl +
        '| 日期 | git short | wasm sha256 | 产物 | 线上（GitHub Pages） |' + $nl +
        '|---|---|---|---|---|' + $nl + $histRow + $nl
    Say 'release.md 新建「发布」节'
} elseif ($md -match [regex]::Escape("| $gitShort |")) {
    Say "release.md 已含 $gitShort（幂等跳过）"
} else {
    $lines = $md -split $nl
    $hdr = -1
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^\| *日期 *\|') { $hdr = $i; break }
    }
    if ($hdr -ge 0) {
        # 表头 + 分隔行之后插一行
        $insertAt = $hdr + 2
        $newLines = @($lines[0..($insertAt - 1)]) + @($histRow) + @($lines[$insertAt..($lines.Count - 1)])
        $md = ($newLines -join $nl)
    } else {
        $md = $md.TrimEnd() + $nl + $nl + '| 日期 | git short | wasm sha256 | 产物 | 线上（GitHub Pages） |' + $nl +
            '|---|---|---|---|---|' + $nl + $histRow + $nl
    }
    Say "release.md 发布历史 += $gitShort"
}
Write-TextNoBom $releaseMd $md

# ── 6) 降级时把实测原因写成「已知限制」（不静默绕过；同文案不重复追加）──
if ($pagesStatus -ne 'PUSHED' -and $pagesStatus -notlike 'PUSHED*') {
    $limLine = "- **Pages 线上段降级（S16.3 publish-web 实测定态 @ $gitShort，$ts）**：" +
        "$pagesStatus — $pagesDetail。" +
        '线上（GitHub Pages）发布需要 origin 写凭据（本机仅可读 `git ls-remote`）或 `gh` 登录态；' +
        '降级下本地产物 + 静态断言（200/content-type 全绿）+ 发布历史仍有效，线上 URL 复跑留待有凭据环境补做。'
    if ($md -notmatch [regex]::Escape("S16.3 publish-web 实测定态 @ $gitShort")) {
        $md = [System.IO.File]::ReadAllText($releaseMd).TrimEnd() + $nl + $nl + $limLine + $nl
        Write-TextNoBom $releaseMd $md
        Say 'release.md 已知限制 += Pages 降级实测'
    }
}

Write-Host "[publish-web] DONE：本地静态断言全绿 + 发布历史已入 release.md + Pages 段 = $pagesStatus"
exit 0
