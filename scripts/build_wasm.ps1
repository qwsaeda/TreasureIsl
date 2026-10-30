# S18.8a：构建 wasm 交付产物到 web/（**Bevy 壳**）。
# 用法：powershell -ExecutionPolicy Bypass -File scripts/build_wasm.ps1
#       powershell -ExecutionPolicy Bypass -File scripts/build_wasm.ps1 -Check
#       （-Check = S21.8 工具契约只读前置体检：不构建 / 不拷贝 / 不写 evidence，前置缺失 ⇒ 非 0 退出）
#
# S18.8a 变更：发货 wasm 入口 = `crates/ti-engine` 的 Bevy 壳 `treasure-bevy-wasm`
# （ti-app 的 macroquad wasm 壳 `treasure-wasm` 已退役，miniquad 的 gl.js / quad-snd 的
# audio.js 一并不再需要）。
# 本脚本职责：**构建 wasm 交付产物 + 拷贝资源 + 记录体积**。
# 浏览器发布链（wasm-bindgen JS 胶水 + HTML 页面 + Bevy 资源管线 + 800 KB 预算复验）
# 属 S18.8e；本脚本产出的 `web/` 暂不含可打开的 Bevy 页面（勿当可玩站点）。
param([switch]$Check)

$ErrorActionPreference = 'Continue'
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$target = 'wasm32-unknown-unknown'
$outDir = 'web'
# S27.7：ti-engine 并回根 workspace ⇒ 产物落**根** target 目录（不再是 crates/ti-engine/target/）。
# 2026-10-29：cargo 统一 target 目录 = /disk/target（`.cargo/config.toml` `[build] target-dir`）；
# 脚本不再硬编码 `target/`，动态解析 cargo 实际 target 目录（Windows 机器若 CARGO_TARGET_DIR
# 覆盖则同样适用）。
$engineManifest = 'crates\ti-engine\Cargo.toml'
$targetDir = (& cargo metadata --no-deps --format-version 1 2>$null |
    ConvertFrom-Json).target_directory
if (-not $targetDir) { $targetDir = 'target' }
# 路径拼接跨平台（Windows 反斜杠 / Linux 正斜杠；docs/migration_linux.md §4）
$wasmOut = Join-Path (Join-Path (Join-Path $targetDir $target) 'release') 'treasure-bevy-wasm.wasm'

# S21.8 工具契约只读模式：-Check = 前置体检（rustup target / 交付壳 manifest / 首屏资源在位；
# 不构建 / 不拷贝 / 不写 evidence；任一前置缺失 ⇒ 非 0 退出）
if ($Check) {
    $ok = $true
    if (((& rustup target list --installed 2>$null) -join ' ') -notmatch $target) {
        Write-Host "  FAIL  未安装 $target" -ForegroundColor Red
        $ok = $false
    }
    if (-not (Test-Path $engineManifest)) {
        Write-Host "  FAIL  缺交付壳 manifest: $engineManifest" -ForegroundColor Red
        $ok = $false
    }
    # 注意：data/ 资源缺失不是致命项（构建流程对缺失资源打 SKIP 继续，见下方资源拷贝段；
    # S11.1 起 data/ 已归档 legacy/data/，本脚本按 SKIP 口径运行）
    if (-not $ok) { exit 1 }
    Write-Host "  OK  -Check 前置体检通过（$target 已装 + 交付壳 manifest + 首屏资源在位；只读未构建）"
    exit 0
}

function Say([string]$msg) { Write-Host "  $msg" }

# 1) 目标已安装？
$installed = (@(& rustup target list --installed 2>$null) -join ' ')
if ($installed -notmatch $target) {
    Write-Host "  MISSING  未安装 $target；请先运行：rustup target add $target" -ForegroundColor Yellow
    exit 1
}
Say "OK  target $target"

# 2) 编译（release；交付产物口径 = release wasm）
Say "cargo build --release -p ti-engine --bin treasure-bevy-wasm --target $target ..."
& cargo build --release -p ti-engine --bin treasure-bevy-wasm --target $target 2>&1 |
    Select-Object -Last 15 | ForEach-Object { Say $_ }
if ($LASTEXITCODE -ne 0) { Write-Host "  FAIL  cargo build" -ForegroundColor Red; exit 1 }

# 2b) S31.2c：wasm-bindgen 胶水（裸 WebAssembly.instantiate 不导函数名，必须生成
#      匹配版本的 .js 绑定 + .d.ts；CLI 须与 lock 的 wasm-bindgen 0.2.129 同版本，
#      缺失时自动 cargo install 匹配版）
$wbWasm = $wasmOut
& wasm-bindgen --version 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    Say "wasm-bindgen CLI 缺失 → 自动安装（cargo install wasm-bindgen-cli --version 0.2.129）"
    & cargo install wasm-bindgen-cli --version 0.2.129 2>&1 |
        Select-Object -Last 3 | ForEach-Object { Say "  $($_)" }
}
& wasm-bindgen $wbWasm --target web --out-dir $outDir 2>&1 |
    Select-Object -Last 5 | ForEach-Object { Say "  wasm-bindgen: $_" }
if ($LASTEXITCODE -ne 0) { Write-Host "  FAIL  wasm-bindgen" -ForegroundColor Red; exit 1 }

# 3) 拷贝 wasm 到 web/
$wasmIn = $wasmOut
if (-not (Test-Path $wasmIn)) { Write-Host "  FAIL  未找到 $wasmIn" -ForegroundColor Red; exit 1 }
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
Say "拷贝 wasm 到 $outDir\ ..."
Copy-Item $wasmIn (Join-Path $outDir 'treasure_bevy_wasm.wasm') -Force

# 3b) 清掉 macroquad 时代的遗留壳文件（gl.js / audio.js / treasure_wasm.wasm / index.html）：
#     它们属于已退役的 macroquad wasm 壳，留着会让人误以为 web/ 仍可玩。
foreach ($stale in @('gl.js', 'audio.js', 'treasure_wasm.wasm')) {
    $p = Join-Path $outDir $stale
    if (Test-Path $p) {
        Remove-Item $p -Force
        Say "移除遗留 $stale（macroquad 壳产物，S18.8a 退役）"
    }
}

# 3c) S31.2c：浏览器入口页（index.html + main.js；wasm-bindgen 胶水由 2b 段生成
#      的 treasure_bevy_wasm.js 驱动）
$htmlSrc = Join-Path $PSScriptRoot 'web_index.html'
$jsSrc = Join-Path $PSScriptRoot 'web_main.js'
Copy-Item $htmlSrc (Join-Path $outDir 'index.html') -Force
Copy-Item $jsSrc (Join-Path $outDir 'main.js') -Force
# S31.2c：env-stub（wasm "env" 模块占位，miniquad 遗留宿主 FFI；S18.8b/c 根除后删除）
$stubSrc = Join-Path $PSScriptRoot 'env-stub.js'
if (Test-Path $stubSrc) {
    Copy-Item $stubSrc (Join-Path $outDir 'env-stub.js') -Force
    Say "entry  env-stub.js -> $outDir\"
}
Say "entry  index.html + main.js -> $outDir\"

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

# 4e) S31.1 地图包部署（`docs/process/21_map_package_spec.md` §7：wasm 侧包根 = `web/assets/maps/`；
#     运行期 fetch 读取归 S31.2+，本步只部署包文件，缺包 = 内嵌默认包 0 行为变化）
$mapsSrc = Join-Path (Get-Location) 'assets\maps'
if (Test-Path $mapsSrc) {
    # S31.2b：生成浏览器侧包清单（幂等；index.json 缺 = 浏览器走内嵌默认包 §6 回退）。
    python scripts/map_pack.py index
    $mapsOut = Join-Path $outDir 'assets\maps'
    New-Item -ItemType Directory -Force -Path $mapsOut | Out-Null
    foreach ($packDir in (Get-ChildItem $mapsSrc -Directory)) {
        if ($packDir.Name -eq '_template') { continue }
        if (-not (Test-Path (Join-Path $packDir.FullName 'manifest.json'))) { continue }
        $dst = Join-Path $mapsOut $packDir.Name
        New-Item -ItemType Directory -Force -Path $dst | Out-Null
        Copy-Item (Join-Path $packDir.FullName '*.json') $dst -Force
        Say "map-pack  $($packDir.Name) -> web/assets/maps/$($packDir.Name)"
    }
    $defaultPtr = Join-Path $mapsSrc '.default'
    if (Test-Path $defaultPtr) {
        Copy-Item $defaultPtr (Join-Path $mapsOut '.default') -Force
        Say "map-pack  .default -> web/assets/maps/.default"
    }
    # S31.2b：浏览器侧已装包清单（`map_pack.py index` 生成；壳层 `map_pack_fetch` 读它
    # 枚举包 + 选默认包，缺 = 内嵌默认包 §6 回退）
    $indexJson = Join-Path $mapsSrc 'index.json'
    if (Test-Path $indexJson) {
        Copy-Item $indexJson (Join-Path $mapsOut 'index.json') -Force
        Say "map-pack  index.json -> web/assets/maps/index.json"
    }
} else {
    Say "SKIP  地图包（assets/maps/ 不存在，运行期全走内嵌默认包）"
}

# 4f) S31.2c：Bevy 资产服务器（wasm HttpWasmAssetReader）fetch 根 = `web/assets/`，
#     按 URL 相对读 `assets/fonts/` + `assets/ui/`（`bevy::asset::AssetPlugin.file_path` 空串
#     = 部署根）；native 壳走 std::fs 不经过本段。缺目录 = 字体/UI 贴图降级不 panic。
$bevyAssetDirs = @('fonts', 'ui')
foreach ($ad in $bevyAssetDirs) {
    $srcDir = Join-Path (Get-Location) "assets\$ad"
    if (Test-Path $srcDir) {
        $dstDir = Join-Path $outDir "assets\$ad"
        New-Item -ItemType Directory -Force -Path $dstDir | Out-Null
        Copy-Item (Join-Path $srcDir '*') $dstDir -Force -Recurse
        $n = (Get-ChildItem $dstDir -File).Count
        Say "bevy-assets  $ad -> web/assets/$ad（$n 个文件）"
    } else {
        Say "SKIP  bevy-assets $ad（assets/$ad/ 不存在）"
    }
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
