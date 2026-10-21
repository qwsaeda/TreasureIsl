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

$scenes = @('index', 'select_role', 'select_map', 'main_map', 'vs', 'message', 'saveload')
if ($args.Count -gt 0) { $scenes = $args }

$thumbDir = 'evidence\ui_preview'
if (-not (Test-Path $thumbDir)) { New-Item -ItemType Directory -Path $thumbDir | Out-Null }

$fail = 0
foreach ($s in $scenes) {
    Write-Host ""
    Write-Host "=== 预览场景: $s ===" -ForegroundColor Cyan
    & cargo run -q --manifest-path crates/ti-engine/Cargo.toml --bin treasure-bevy-native -- --scene=$s --shot 2>&1 |
        Select-String -Pattern '\[shot\]' | ForEach-Object { Write-Host "  $_" }

    $png = "evidence\bevy_native_$s.png"
    if (-not (Test-Path $png)) {
        Write-Host "  FAIL  未产出 $png" -ForegroundColor Red
        $fail++
        continue
    }
    # 统计 + 缩略图（用 PIL；无 PIL 则只报尺寸）
    $py = @"
from PIL import Image
im = Image.open(r'$png').convert('RGB')
data = im.tobytes()
nb = sum(1 for i in range(0, len(data), 3) if data[i] > 8 or data[i + 1] > 8 or data[i + 2] > 8)
print(f'  nonblack = {nb} / {im.width * im.height}' + ('  <-- EMPTY FRAME!' if nb == 0 else ''))
im.thumbnail((540, 960))
im.save(r'$thumbDir\$s.png')
"@
    $py | python -
    if ($LASTEXITCODE -ne 0) { Write-Host "  WARN  统计失败（无 PIL？）" -ForegroundColor Yellow }
}

Write-Host ""
if ($fail -gt 0) {
    Write-Host "预览结束：$fail 个场景未出图" -ForegroundColor Red
    exit 1
}
Write-Host "预览结束：产物在 evidence/bevy_native_*.png + evidence/ui_preview/*.png"
exit 0
