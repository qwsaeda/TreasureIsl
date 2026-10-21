# S18.8a：构建 wasm 交付产物到 web/（**Bevy 壳**）。
# 用法：powershell -ExecutionPolicy Bypass -File scripts/build_wasm.ps1
#
# S18.8a 变更：发货 wasm 入口 = `crates/ti-engine` 的 Bevy 壳 `treasure-bevy-wasm`
# （ti-app 的 macroquad wasm 壳 `treasure-wasm` 已退役，miniquad 的 gl.js / quad-snd 的
# audio.js 一并不再需要）。
# 本脚本职责：**构建 wasm 交付产物 + 拷贝资源 + 记录体积**。
# 浏览器发布链（wasm-bindgen JS 胶水 + HTML 页面 + Bevy 资源管线 + 800 KB 预算复验）
# 属 S18.8e；本脚本产出的 `web/` 暂不含可打开的 Bevy 页面（勿当可玩站点）。
param()

$ErrorActionPreference = 'Continue'
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$target = 'wasm32-unknown-unknown'
$outDir = 'web'
$engineManifest = 'crates\ti-engine\Cargo.toml'

function Say([string]$msg) { Write-Host "  $msg" }

# 1) 目标已安装？
$installed = (@(& rustup target list --installed 2>$null) -join ' ')
if ($installed -notmatch $target) {
    Write-Host "  MISSING  未安装 $target；请先运行：rustup target add $target" -ForegroundColor Yellow
    exit 1
}
Say "OK  target $target"

# 2) 编译（release；交付产物口径 = release wasm）
Say "cargo build --release --manifest-path $engineManifest --bin treasure-bevy-wasm --target $target ..."
& cargo build --release --manifest-path $engineManifest --bin treasure-bevy-wasm --target $target 2>&1 |
    Select-Object -Last 15 | ForEach-Object { Say $_ }
if ($LASTEXITCODE -ne 0) { Write-Host "  FAIL  cargo build" -ForegroundColor Red; exit 1 }

# 3) 拷贝 wasm 到 web/
$wasmIn = "crates\ti-engine\target\$target\release\treasure-bevy-wasm.wasm"
if (-not (Test-Path $wasmIn)) { Write-Host "  FAIL  未找到 $wasmIn" -ForegroundColor Red; exit 1 }
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
Say "拷贝 wasm 到 $outDir\ ..."
Copy-Item $wasmIn (Join-Path $outDir 'treasure_bevy_wasm.wasm') -Force

# 3b) 清掉 macroquad 时代的遗留壳文件（gl.js / audio.js / treasure_wasm.wasm / index.html）：
#     它们属于已退役的 macroquad wasm 壳，留着会让人误以为 web/ 仍可玩。
foreach ($stale in @('gl.js', 'audio.js', 'treasure_wasm.wasm', 'index.html')) {
    $p = Join-Path $outDir $stale
    if (Test-Path $p) {
        Remove-Item $p -Force
        Say "移除遗留 $stale（macroquad 壳产物，S18.8a 退役）"
    }
}

# 4) 拷贝真实资源（Bevy 资产管线落地的输入；S18.8e 接进 Bevy 壳的资源加载）
#    美术**不转码**——PCX 已是 RLE+调色板的最优压缩，且 .Map 按精灵索引寻址，
#    转图集会打断寻址；只转必须转的音频（MOD→OGG，见 audio_prep）。
$assetsOut = Join-Path $outDir 'assets'
New-Item -ItemType Directory -Force -Path $assetsOut | Out-Null

# 4a) 首屏必需：字体 + 字符表 + 设置 + 精灵图 + 8 个场景 .Map
$firstAssets = @(
    'GoldLand.Fnt', 'font.dat', 'settings.dat',
    'ui.pak',              # 场景 .Map 索引 + 控件表 + 120 PCX 贴图（SpriteAtlas 数据源）
    'glpics.pak',          # 88 张精灵 PCX（face_*/troop/ship/flag/landmark）
    'index.Map', 'intro.Map', 'selectrole.Map', 'selectmap.Map',
    'main.Map', 'vs.Map', 'msgs.Map', 'saveload.Map',
    'glmap_m01.pak',        # 地形瓦片 PCX（S6.3 TerrainAtlas 数据源）
    'glmapm01.Map'          # 地形布局（cell 记录 + 等距投影坐标）
)

# 4b) 按需补载：进主地图时拉地形/卡牌/商品，音效空闲补载
$lazyAssets = @(
    'glmap_m02.pak',                    # 地形瓦片（m02；m01 已归首屏）
    'glmapm02.Map', 'glmapm03.Map',   # 地形布局（m01 已归首屏）
    'cards.lpk',                          # 卡牌/事件数据
    'goods.lpk',                          # 商品表
    'music.lpk'                           # 音频（MOD→OGG 已转码）
)

# 4c) 其余场景布局（除首屏两个与地形族外，data/ 下全部 .Map）
$sceneAssets = @(Get-ChildItem 'data' -Filter '*.Map' -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -notin $firstAssets -and $_.Name -notlike 'glmapm0*.Map' } |
    ForEach-Object { $_.Name })

foreach ($f in ($firstAssets + $lazyAssets + $sceneAssets)) {
    $src = Join-Path 'data' $f
    if (Test-Path $src) { Copy-Item $src (Join-Path $assetsOut $f) -Force; Say "asset  $f" }
    else { Say "SKIP  $f（data/ 中不存在）" }
}

# 4d) S7.6：HD 资产分包（首屏 1x + 非首屏 webp）
$hdPack = 'assets_hd_pack'
if (Test-Path (Join-Path $hdPack '1x')) {
    $hdOut = Join-Path $outDir 'hd'
    New-Item -ItemType Directory -Force -Path $hdOut | Out-Null
    # 清旧 HD 产物（避免 Copy-Item -Recurse 追加叠加）
    Remove-Item (Join-Path $hdOut '1x') -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item (Join-Path $hdOut 'webp') -Recurse -Force -ErrorAction SilentlyContinue
    # 1x 首屏（全容器）
    $hd1x = Join-Path $hdOut '1x'
    Copy-Item (Join-Path $hdPack '1x') $hd1x -Recurse -Force
    Say "hd/1x  <- assets_hd_pack/1x（首屏 HD 真彩）"
    # N7：HD 注入 manifest（wasm/android bin 启动时读，EXP-048/060 注入数 = 目录全量）。
    $manifestSrc = Join-Path $hdPack 'hd_f1_manifest.json'
    if (Test-Path $manifestSrc) {
        $manifestOut = Join-Path $outDir 'assets_hd_pack'
        New-Item -ItemType Directory -Force -Path $manifestOut | Out-Null
        Copy-Item $manifestSrc (Join-Path $manifestOut 'hd_f1_manifest.json') -Force
        Say "assets_hd_pack/hd_f1_manifest.json（N7 注入清单）"
    } else {
        Say "SKIP  hd_f1_manifest.json（不存在，N7 注入回退 PCX 原路径）"
    }
    # webp 非首屏（体积减 30~50%）
    $hdWebp = Join-Path $hdOut 'webp'
    if (Test-Path (Join-Path $hdPack 'webp')) {
        Copy-Item (Join-Path $hdPack 'webp') $hdWebp -Recurse -Force
        Say "hd/webp <- assets_hd_pack/webp（非首屏 WebP 减体积）"
    }
} else {
    Say "SKIP  HD 资产（assets_hd_pack/1x 不存在，先跑 scripts/pack_hd_assets.py）"
}

$artifacts = Get-ChildItem $outDir -Include '*.html', '*.js', '*.wasm' -Recurse |
    Where-Object { $_.Name -notlike '*.d' }
Write-Host ""
Write-Host "  web/ 产物："
foreach ($a in $artifacts) {
    $hash = (Get-FileHash -Algorithm SHA256 $a.FullName).Hash.Substring(0, 16)
    Say ("{0,-28} {1,10:N0} B  {2}" -f $a.Name, $a.Length, $hash)
}

# 5) 体积记录（evidence/S18.8a-wasm-size.log）
#    口径说明：S18.8a 只**记录** Bevy wasm 体积，不做预算判定 ——
#    预算口径 = `docs/process/11_bevy_migration.md` §3（S18.0 重立：wasm 主体 ≤8 MB，
#    旧 800 KB 预算已作废）；LTO/opt 后复验 + 发布属 S18.8e。
$firstScreen = @()
$firstScreen += (Get-ChildItem $outDir -Include 'index.html','gl.js','audio.js' -Recurse -ErrorAction SilentlyContinue)
$assetsDir = Join-Path $outDir 'assets'
$firstScreenAssets = @(
    'GoldLand.Fnt','font.dat','settings.dat','ui.pak','glpics.pak',
    'index.Map','intro.Map','selectrole.Map','selectmap.Map',
    'main.Map','vs.Map','msgs.Map','saveload.Map',
    'glmap_m01.pak', 'glmapm01.Map'
)
$firstScreen += (Get-ChildItem $assetsDir -ErrorAction SilentlyContinue |
    Where-Object { $firstScreenAssets -contains $_.Name })
$nonFirstScreen = @()
$nonFirstScreen += (Get-ChildItem $assetsDir -ErrorAction SilentlyContinue |
    Where-Object { $firstScreenAssets -notcontains $_.Name })

$fsBytes = ($firstScreen | Measure-Object -Property Length -Sum).Sum
$nfsBytes = ($nonFirstScreen | Measure-Object -Property Length -Sum).Sum
$wasmBytes = (Get-Item (Join-Path $outDir 'treasure_bevy_wasm.wasm') -ErrorAction SilentlyContinue).Length

# S7.6：HD 体积（web/hd/1x 首屏 + web/hd/webp 非首屏）
$hd1xDir = Join-Path $outDir 'hd/1x'
$hdWebpDir = Join-Path $outDir 'hd/webp'
$hd1xFiles = @()
$hdWebpFiles = @()
if (Test-Path $hd1xDir) { $hd1xFiles = Get-ChildItem $hd1xDir -Recurse -File -ErrorAction SilentlyContinue }
if (Test-Path $hdWebpDir) { $hdWebpFiles = Get-ChildItem $hdWebpDir -Recurse -File -ErrorAction SilentlyContinue }
$hd1xBytes = ($hd1xFiles | Measure-Object -Property Length -Sum).Sum
$hdWebpBytes = ($hdWebpFiles | Measure-Object -Property Length -Sum).Sum

$sizelog = "evidence\S18.8a-wasm-size.log"
if (-not (Test-Path 'evidence')) { New-Item -ItemType Directory -Path 'evidence' | Out-Null }
$lines = @()
$lines += "S18.8a wasm 体积记录（$((Get-Date).ToString('yyyy-MM-dd'))）"
$lines += "口径：Bevy 交付壳 `treasure-bevy-wasm`（release，strip=symbols，S18.6 门禁重建时叠加 LTO/wasm-opt）。"
$lines += "预算判定（wasm 主体 ≤8 MB，11_bevy_migration §3）属 S18.8e；本文件只记录实测值。"
$lines += ""
$lines += "== wasm 主体 =="
$lines += ("{0,-28} {1,10:N0} B" -f "treasure_bevy_wasm.wasm", $wasmBytes)
$lines += ""
$lines += "== 首屏附属资源（不含 wasm 主体）=="
foreach ($a in $firstScreen) {
    $lines += ("{0,-28} {1,10:N0} B" -f $a.Name, $a.Length)
}
$lines += ("{0,-28} {1,10:N0} B" -f "首屏附属合计", $fsBytes)
$lines += ""
$lines += "== 非首屏资源 =="
foreach ($a in $nonFirstScreen) {
    $lines += ("{0,-28} {1,10:N0} B" -f $a.Name, $a.Length)
}
$lines += ("{0,-28} {1,10:N0} B" -f "非首屏合计", $nfsBytes)
$lines += ""
$lines += "== HD 资产 =="
$lines += ("{0,-28} {1,10:N0} B" -f "hd/1x 合计", $hd1xBytes)
$lines += ("{0,-28} {1,10:N0} B" -f "hd/webp 合计", $hdWebpBytes)
$lines | Out-File -FilePath $sizelog -Encoding utf8
Write-Host ""
Say "wasm 体积已记录到 $sizelog（wasm 主体 ${wasmBytes} B，首屏附属 ${fsBytes} B，非首屏 ${nfsBytes} B）"

# 6) 本地预览
Say "本地预览：cargo run -p ti-tools --bin serve -- --dir $outDir --port 8080"
Say "注意：S18.8a 的 web/ 尚无 Bevy 浏览器页（wasm-bindgen 胶水 + HTML = S18.8e）。"
exit 0
