# S2.5：构建 wasm 产物到 web/（浏览器壳）。
# 用法：powershell -ExecutionPolicy Bypass -File scripts/build_wasm.ps1
#
# 步骤：cargo build (wasm32, release) → 拷 wasm + miniquad gl.js + quad-snd audio.js
#       → 拷真实资源（字体/字符表/音乐包）→ web/
# 前置：rustup target add wasm32-unknown-unknown（脚本会检查并提示）
param()

$ErrorActionPreference = 'Continue'
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$target = 'wasm32-unknown-unknown'
$outDir = 'web'

function Say([string]$msg) { Write-Host "  $msg" }

# 1) 目标已安装？
$installed = (@(& rustup target list --installed 2>$null) -join ' ')
if ($installed -notmatch $target) {
    Write-Host "  MISSING  未安装 $target；请先运行：rustup target add $target" -ForegroundColor Yellow
    exit 1
}
Say "OK  target $target"

# 2) macroquad/miniquad 的 wasm 产物是**自包含**模块（导出 `_start`/`memory`，
#    JS 加载器已内联），因此不需要 wasm-bindgen CLI —— 直接实例化裸 `.wasm`。
Say 'OK  无需 wasm-bindgen（macroquad 输出自包含 wasm）'

# 3) 编译（release）
Say "cargo build --release --target $target ..."
& cargo build -p ti-app --features wasm-shell --release --target $target 2>&1 |
    Select-Object -Last 15 | ForEach-Object { Say $_ }
if ($LASTEXITCODE -ne 0) { Write-Host "  FAIL  cargo build" -ForegroundColor Red; exit 1 }

# 4) 生成 JS 胶水 + 优化 wasm
$wasmIn = "target\$target\release\treasure-wasm.wasm"
if (-not (Test-Path $wasmIn)) { Write-Host "  FAIL  未找到 $wasmIn" -ForegroundColor Red; exit 1 }
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
Say "拷贝 wasm 到 $outDir\ ..."
Copy-Item $wasmIn (Join-Path $outDir 'treasure_wasm.wasm') -Force

# 4b) 拷贝 miniquad 的 JS 宿主驱动 gl.js（导出 load()/wasm_exports，必须静态引入）
$glJs = Get-ChildItem "$env:USERPROFILE\.cargo\registry\src" -Recurse -Filter 'gl.js' -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -match 'miniquad-[\d.]+\\js\\gl\.js$' } |
    Sort-Object FullName -Descending |
    Select-Object -First 1
if ($glJs) {
    Copy-Item $glJs.FullName (Join-Path $outDir 'gl.js') -Force
    Say "gl.js  <- $($glJs.FullName)"
} else {
    Write-Host "  FAIL  找不到 miniquad 的 js/gl.js" -ForegroundColor Red
    exit 1
}

# 4c) 拷贝 quad-snd 的 WebAudio 胶水（macroquad 的 `audio` feature 在 wasm 上
#     必须由它向 gl.js 注册 env.audio_* 导入；缺失会导致 wasm 实例化后
#     is_loaded() 永远为 false → 启动挂起，故这里硬失败）
$audioJs = Get-ChildItem "$env:USERPROFILE\.cargo\registry\src" -Recurse -Filter 'audio.js' -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -match 'quad-snd-[\d.]+\\js\\audio\.js$' } |
    Sort-Object FullName -Descending |
    Select-Object -First 1
if ($audioJs) {
    Copy-Item $audioJs.FullName (Join-Path $outDir 'audio.js') -Force
    Say "audio.js <- $($audioJs.FullName)"
} else {
    Write-Host "  FAIL  找不到 quad-snd 的 js/audio.js（先 cargo build 拉取依赖）" -ForegroundColor Red
    exit 1
}

# 5) 拷贝真实资源（S2.5 DoD：浏览器显示真实资源 + 真实音效）
$assetsOut = Join-Path $outDir 'assets'
New-Item -ItemType Directory -Force -Path $assetsOut | Out-Null
foreach ($f in @('GoldLand.Fnt', 'font.dat', 'settings.dat', 'music.lpk')) {
    $src = Join-Path 'data' $f
    if (Test-Path $src) { Copy-Item $src (Join-Path $assetsOut $f) -Force; Say "asset  $f" }
}

# 6) 产物清单 + sha256（进 evidence）
$artifacts = Get-ChildItem $outDir -Include '*.html', '*.js', '*.wasm' -Recurse |
    Where-Object { $_.Name -notlike '*.d' }
Write-Host ""
Write-Host "  web/ 产物："
foreach ($a in $artifacts) {
    $hash = (Get-FileHash -Algorithm SHA256 $a.FullName).Hash.Substring(0, 16)
    Say ("{0,-28} {1,10:N0} B  {2}" -f $a.Name, $a.Length, $hash)
}
Write-Host ""
Say "本地预览：cargo run -p ti-tools --bin serve -- --dir $outDir --port 8080"
exit 0