# S18.8b：UI 界面预览闭环（改 scenes.json → 出图 → 看效果）。
# 用法：
#   powershell -ExecutionPolicy Bypass -File scripts/ui_preview.ps1                 # 全部 7 场景
#   powershell -ExecutionPolicy Bypass -File scripts/ui_preview.ps1 index           # 单场景
#   powershell -ExecutionPolicy Bypass -File scripts/ui_preview.ps1 index main_map  # 多场景
#
# 产物：
#   evidence/bevy_native_<scene>.png   离屏出图（1080×1920，仓库内）
#   evidence/ui_preview/<scene>.png     缩略图（540×960，便于人眼过目）
#   控制台：每场景的非黑像素统计（0 = 空帧 = 渲染/资产没生效）
$ErrorActionPreference = 'Continue'
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

# Python 解释器（Windows = python；Linux/macOS = python3；同 gate.ps1 `$py` 口径）
$py = if ($env:OS -eq 'Windows_NT') { 'python' } else { 'python3' }

$scenes = @('index', 'select_role', 'select_map', 'main_map', 'vs', 'message', 'saveload')
if ($args.Count -gt 0) { $scenes = $args }

# S21.8 工具契约只读模式：-Check = 既有出图验收（不重渲染 / 不写缩略图；缺图 / 空帧 ⇒ 非 0 退出）
# 用法：powershell -ExecutionPolicy Bypass -File scripts/ui_preview.ps1 -Check [scene...]
if ($args -contains '-Check') {
    $checkScenes = @($args | Where-Object { $_ -ne '-Check' })
    if ($checkScenes.Count -eq 0) { $checkScenes = @('index', 'select_role', 'select_map', 'main_map', 'vs', 'message', 'saveload') }
    $fail = 0
    foreach ($s in $checkScenes) {
        $png = Join-Path 'evidence' "bevy_native_$s.png"
        if (-not (Test-Path $png)) {
            Write-Host "  FAIL  缺出图 $png（先跑默认模式出图）" -ForegroundColor Red
            $fail++
            continue
        }
        # 单引号 here-string（@'...'@）= 不插值 ⇒ Python 代码里的 $i / $png 不被 PS 展开
        $pyCheckCode = @'
from PIL import Image
import sys
png = sys.argv[1]
scene = sys.argv[2]
im = Image.open(png).convert('RGB')
data = im.tobytes()
nb = sum(1 for i in range(0, len(data), 3) if data[i] > 8 or data[i + 1] > 8 or data[i + 2] > 8)
print(f'  [{scene}] nonblack = {nb} / {im.width * im.height}')
raise SystemExit(1 if nb == 0 else 0)
'@
        $pyCheckCode | & $py - $png $s
        if ($LASTEXITCODE -ne 0) { Write-Host "  FAIL  $s 空帧（或无 PIL）" -ForegroundColor Red; $fail++ }
    }
    if ($fail -gt 0) {
        Write-Host "-Check 结束：$fail 个场景未通过（只读：未重渲染 / 未写缩略图）" -ForegroundColor Red
        exit 1
    }
    Write-Host "-Check 结束：既有出图全部在位且非空帧（只读，未写文件）"
    exit 0
}

$thumbDir = Join-Path 'evidence' 'ui_preview'
if (-not (Test-Path $thumbDir)) { New-Item -ItemType Directory -Path $thumbDir | Out-Null }

$fail = 0
foreach ($s in $scenes) {
    Write-Host ""
    Write-Host "=== 预览场景: $s ===" -ForegroundColor Cyan
    # S19.7 D5：main_map 带运行时状态出图（`--spin=5`，固定 seed=42 ⇒ 可复现，
    # 且必落在有落格事件的格上 ⇒ 提示条可见 = 验收证据）
    # 注意：`$extra` 不能带前导空格（否则整个参数变成 " --spin=5" 解析失败）
    $extra = ''
    if ($s -eq 'main_map') { $extra = '--spin=5' }
    # Linux 无 DISPLAY（headless CI）时 main_map / vs 场景需真窗口 ⇒ 自动包 xvfb-run
    # （Windows / 有 DISPLAY 时直连，行为不变）。
    $cargoArgs = @('run', '-q', '-p', 'ti-engine', '--bin', 'treasure-bevy-native',
                   '--', "--scene=$s", '--shot')
    if ($extra) { $cargoArgs += $extra }
    $useXvfb = ($env:OS -ne 'Windows_NT') -and (-not $env:DISPLAY) -and (Get-Command xvfb-run -ErrorAction SilentlyContinue)
    if ($useXvfb) {
        $cmd = @('xvfb-run', '-a', 'cargo') + $cargoArgs
    } else {
        $cmd = @('cargo') + $cargoArgs
    }
    & $cmd[0] $cmd[1..($cmd.Length - 1)] 2>&1 |
        Select-String -Pattern '\[shot\]' | ForEach-Object { Write-Host "  $_" }

    $png = Join-Path 'evidence' "bevy_native_$s.png"
    if (-not (Test-Path $png)) {
        Write-Host "  FAIL  未产出 $png" -ForegroundColor Red
        $fail++
        continue
    }
    # 统计 + 缩略图（用 PIL；无 PIL 则只报尺寸）
    $thumbPath = (Join-Path $thumbDir "$s.png")
    # 单引号 here-string（不插值）；路径经 argv 传入（免 PS 变量展开问题）
    $pyCode = @'
from PIL import Image
import sys
png = sys.argv[1]
thumb = sys.argv[2]
im = Image.open(png).convert('RGB')
data = im.tobytes()
nb = sum(1 for i in range(0, len(data), 3) if data[i] > 8 or data[i + 1] > 8 or data[i + 2] > 8)
print(f'  nonblack = {nb} / {im.width * im.height}' + ('  <-- EMPTY FRAME!' if nb == 0 else ''))
im.thumbnail((540, 960))
im.save(thumb)
'@
    $pyCode | & $py - $png $thumbPath
    if ($LASTEXITCODE -ne 0) { Write-Host "  WARN  统计失败（无 PIL？）" -ForegroundColor Yellow }
}

Write-Host ""
if ($fail -gt 0) {
    Write-Host "预览结束：$fail 个场景未出图" -ForegroundColor Red
    exit 1
}
Write-Host "预览结束：产物在 evidence/bevy_native_*.png + evidence/ui_preview/*.png"
exit 0
