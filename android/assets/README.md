# Android assets（S5.5 / S2.6，Mac 侧编译 APK 时由 cargo-apk 打进 APK）

本目录是 cargo-apk 标准布局的 `android/assets/`。`treasure_android.rs` 的 `read_asset`
经 `macroquad::file::load_file` 读取 APK assets（与 wasm 的 HTTP fetch 同源机制）。

## 同步（编译 APK 前执行）

```sh
mkdir -p android/assets
cp data/* android/assets/          # 或从已构建的浏览器包同步（推荐）
#   cp web/assets/* android/assets/
```

**Android 与浏览器加载同一套资源**，两者内容必须一致；
`scripts/build_wasm.ps1` 的资产清单（33 个）即为准。

## 清单（33 个，约 483 KB）

- **首屏 11 个**：`GoldLand.Fnt`、`font.dat`、`settings.dat`、`ui.pak`、`glpics.pak`、
  `index.Map`、`intro.Map` + 其余场景 `.Map` 的一部分
- **按需 22 个**：地形瓦片（`glmap_m0*.pak`）、地形布局（`glmapm0*.Map`）、
  `cards.lpk`、`goods.lpk`、其余 18 个场景 `.Map`、`music.lpk`

逐项体积与触发时机见 `../README.md` §2。

## 可选：OGG 预转码

若 ti-app 开 `mod-ogg` feature（默认关），还需把 `build_assets` 离线产物
`assets_out/audio/*.ogg` 复制进 `android/assets/audio/`。默认关闭走 WAV 混音，无需 OGG。

## 版本控制

本目录的实际资源文件（大二进制）**不入库**，只保留本说明。
构建前把 `data/` 或 `web/assets/` 同步进来即可（源资源已在 `data/` 入库）。
