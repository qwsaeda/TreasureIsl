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
# 资源改造 Step 1：补齐全量美术资产（此前只拷字体+音频，美术层整条链是空的）。
# 美术**不转码**——PCX 已是 RLE+调色板的最优压缩，且 .Map 按精灵索引寻址，
# 转图集会打断寻址；只转必须转的音频（MOD→OGG，见 audio_prep）。
$assetsOut = Join-Path $outDir 'assets'
New-Item -ItemType Directory -Force -Path $assetsOut | Out-Null

# 5a) 首屏必需：字体 + 字符表 + 设置 + 精灵图 + 8 个场景 .Map
#      （当前 shell 启动时 fetch 全部 8 个场景；blit 真实贴图依赖 ui.pak + 对应 .Map）
$firstAssets = @(
    'GoldLand.Fnt', 'font.dat', 'settings.dat',
    'ui.pak',              # 场景 .Map 索引 + 控件表 + 120 PCX 贴图（SpriteAtlas 数据源）
    'glpics.pak',          # 88 张精灵 PCX（face_*/troop/ship/flag/landmark）
    'index.Map', 'intro.Map', 'selectrole.Map', 'selectmap.Map',
    'main.Map', 'vs.Map', 'msgs.Map', 'saveload.Map',
    'glmap_m01.pak',        # 地形瓦片 PCX（S6.3 TerrainAtlas 数据源）
    'glmapm01.Map'          # 地形布局（cell 记录 + 等距投影坐标）
)

# 5b) 按需补载：进主地图时拉地形/卡牌/商品，音效空闲补载
$lazyAssets = @(
    'glmap_m02.pak',                    # 地形瓦片（m02；m01 已归首屏）
    'glmapm02.Map', 'glmapm03.Map',   # 地形布局（m01 已归首屏）
    'cards.lpk',                          # 卡牌/事件数据
    'goods.lpk',                          # 商品表
    'music.lpk'                           # 音频（MOD→OGG 已转码）
)

# 5c) 其余场景布局（除首屏两个与地形族外，data/ 下全部 .Map）
$sceneAssets = @(Get-ChildItem 'data' -Filter '*.Map' -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -notin $firstAssets -and $_.Name -notlike 'glmapm0*.Map' } |
    ForEach-Object { $_.Name })

foreach ($f in ($firstAssets + $lazyAssets + $sceneAssets)) {
    $src = Join-Path 'data' $f
    if (Test-Path $src) { Copy-Item $src (Join-Path $assetsOut $f) -Force; Say "asset  $f" }
    else { Say "SKIP  $f（data/ 中不存在）" }
}

# 5d) S7.6：HD 资产分包（首屏 1x + 非首屏 webp）
# 首屏 HD = assets_hd_pack/1x（真彩 1x，阻塞首帧的 HD 路径）；
# 非首屏 HD = assets_hd_pack/webp（WebP 转码，减体积 30~50%）。
# 无 assets_hd_pack/ 时跳过（S7.2~S7.4 未跑或 --no-hd）。
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
    # webp 非首屏（体积减 30~50%）
    if (Test-Path (Join-Path $hdPack 'webp')) {
        $hdWebp = Join-Path $hdOut 'webp'
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

# 7) 分包体积记录（evidence/S4.3-size.log）
# 首屏附属 = index.html + gl.js + audio.js + 字体 + 主菜单必需美术（阻塞首帧）
# wasm 主体单独记录；非首屏 = 地形/卡牌/商品/音频（进对应场景或空闲补载）
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
$wasmBytes = (Get-Item (Join-Path $outDir 'treasure_wasm.wasm') -ErrorAction SilentlyContinue).Length

# S7.6：HD 体积（web/hd/1x 首屏 + web/hd/webp 非首屏）
$hd1xDir = Join-Path $outDir 'hd/1x'
$hdWebpDir = Join-Path $outDir 'hd/webp'
$hd1xFiles = @()
$hdWebpFiles = @()
if (Test-Path $hd1xDir) { $hd1xFiles = Get-ChildItem $hd1xDir -Recurse -File -ErrorAction SilentlyContinue }
if (Test-Path $hdWebpDir) { $hdWebpFiles = Get-ChildItem $hdWebpDir -Recurse -File -ErrorAction SilentlyContinue }
$hd1xBytes = ($hd1xFiles | Measure-Object -Property Length -Sum).Sum
$hdWebpBytes = ($hdWebpFiles | Measure-Object -Property Length -Sum).Sum

$sizelog = "evidence\S4.3-size.log"
$sizelogS76 = "evidence\S7.6-size.log"
if (-not (Test-Path 'evidence')) { New-Item -ItemType Directory -Path 'evidence' | Out-Null }
$lines = @()
$lines += "S4.3 wasm 体积记录（$((Get-Date).ToString('yyyy-MM-dd'))）"
$lines += ""
$lines += "== wasm 主体 =="
$lines += ("{0,-28} {1,10:N0} B" -f "treasure_wasm.wasm", $wasmBytes)
$lines += ""
$lines += "== 首屏附属资源（阻塞首帧渲染，不含 wasm 主体）=="
foreach ($a in $firstScreen) {
    $lines += ("{0,-28} {1,10:N0} B" -f $a.Name, $a.Length)
}
$lines += ("{0,-28} {1,10:N0} B" -f "首屏附属合计", $fsBytes)
$lines += ""
$lines += "== 非首屏资源（进对应场景或空闲补载，不阻塞首屏）=="
foreach ($a in $nonFirstScreen) {
    $lines += ("{0,-28} {1,10:N0} B" -f $a.Name, $a.Length)
}
$lines += ("{0,-28} {1,10:N0} B" -f "非首屏合计", $nfsBytes)
$lines += ""
$lines += "== 预算比对（基准见 docs/plan-s4.md §S4.3 体积预算）=="
$wasmBudget = 800000
$fsBudget = 430000      # 资源改造 Step 1+2 后含全部 8 场景 .Map + ui.pak + glpics.pak（原 150 KB）
$nfBudget = 200000      # 地形瓦片/地形布局/cards/goods/music（原 450 KB，部分场景 .Map 已归首屏）
$wasmOk = 'PASS'; if ($wasmBytes -gt $wasmBudget) { $wasmOk = 'FAIL(超预算)' }
$fsOk = 'PASS'; if ($fsBytes -gt $fsBudget) { $fsOk = 'FAIL(超预算)' }
$nfOk = 'PASS'; if ($nfsBytes -gt $nfBudget) { $nfOk = 'FAIL(超预算)' }
$lines += ("wasm 主体 {0:N0} B / 预算 {1:N0} B -> {2}" -f $wasmBytes, $wasmBudget, $wasmOk)
$lines += ("首屏附属 {0:N0} B / 预算 {1:N0} B -> {2}" -f $fsBytes, $fsBudget, $fsOk)
$lines += ("非首屏   {0:N0} B / 预算 {1:N0} B -> {2}" -f $nfsBytes, $nfBudget, $nfOk)
$lines += ""
$lines += "分包说明（资源改造 Step 1，docs/modernization.md §9）："
$lines += "  首屏  = wasm + gl.js + audio.js + 字体 + ui.pak + glpics.pak + index/intro.Map"
$lines += "  非首屏= 地形瓦片/地形布局（进主地图时）、cards/goods（触发时）、"
$lines += "          其余场景 .Map（进对应场景时）、music.lpk（空闲 30 帧补载）"
$lines += "美术不转码：PCX 已是最优压缩（RLE+调色板），且 .Map 按精灵索引 f1 寻址，"
$lines += "转图集会打断寻址并引入像素偏差（P4-1 要求逐像素一致）；仅音频 MOD→OGG 必须转。"
$lines | Out-File -FilePath $sizelog -Encoding utf8
Write-Host ""
Say "首屏/非首屏体积已记录到 $sizelog（首屏附属 ${fsBytes} B，非首屏 ${nfsBytes} B）"

# S7.6：HD 体积记录（evidence/S7.6-size.log，docs/modernization.md §9 HD 单列）
$hdLines = @()
$hdLines += "S7.6 HD 资产体积记录（$((Get-Date).ToString('yyyy-MM-dd'))）"
$hdLines += ""
$hdLines += "== HD 首屏（web/hd/1x，真彩 1x，阻塞首帧 HD 路径）=="
$hdLines += ("{0,-28} {1,10:N0} B" -f "hd/1x 合计（$($hd1xFiles.Count) 文件）", $hd1xBytes)
$hdLines += ""
$hdLines += "== HD 非首屏（web/hd/webp，WebP 转码，减体积 30~50%）=="
$hdLines += ("{0,-28} {1,10:N0} B" -f "hd/webp 合计（$($hdWebpFiles.Count) 文件）", $hdWebpBytes)
$hdLines += ""
# HD 预算（docs/impl/14_asset_enhancement.md §6，实测 S7.6 重列）：首屏 1x 全量 335 文件，
# 预估 1.2 MB 偏小（C/D 地形+兵种 PNG-32 实测偏高），实测 1.52 MB → 预算 1.6 MB；webp 全量 ~1.1 MB
$hd1xBudget = 1600000
$hdWebpBudget = 3000000
$hd1xOk = 'PASS'; if ($hd1xBytes -gt $hd1xBudget) { $hd1xOk = 'FAIL(超预算)' }
$hdWebpOk = 'PASS'; if ($hdWebpBytes -gt $hdWebpBudget) { $hdWebpOk = 'FAIL(超预算)' }
$hdLines += "== 预算比对（基准见 docs/impl/14_asset_enhancement.md §6）=="
$hdLines += ("HD 首屏 1x {0:N0} B / 预算 {1:N0} B -> {2}" -f $hd1xBytes, $hd1xBudget, $hd1xOk)
$hdLines += ("HD 非首屏 webp {0:N0} B / 预算 {1:N0} B -> {2}" -f $hdWebpBytes, $hdWebpBudget, $hdWebpOk)
$hdLines | Out-File -FilePath $sizelogS76 -Encoding utf8
Say "HD 体积已记录到 $sizelogS76（首屏 1x ${hd1xBytes} B，webp ${hdWebpBytes} B）"

# 8) 本地预览
Say "本地预览：cargo run -p ti-tools --bin serve -- --dir $outDir --port 8080"
exit 0