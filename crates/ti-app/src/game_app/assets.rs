//! 资产注入：图集 / 地形包 / HD 贴图包（渲染资源的加载与注入）。
//!
//! 拆分自 `game_app.rs`（S20.0b）：资产装载与**玩法状态**无关，独立成模块。

use super::GameApp;
use crate::sprite_atlas::SpriteAtlas;

impl GameApp {
    /// 注入精灵图集（`ui.pak` 字节 → `SpriteAtlas`）。失败保持 `None`（渲染退回占位）。
    pub fn set_atlas(&mut self, pkg_bytes: &[u8]) {
        if let Ok(atlas) = SpriteAtlas::new(pkg_bytes) {
            self.atlas = Some(atlas);
        }
    }
    /// 注入已解析的 `.Map` 场景（`name → MapScene`，`render` 时 blit 精灵用）。
    ///
    /// 解析失败的场景跳过（不阻塞其他场景渲染）。
    pub fn add_scene(&mut self, name: &'static str, bytes: &[u8]) {
        if let Ok(scene) = ti_formats::mapfile::MapScene::parse(name, bytes) {
            self.scene_cache.insert(name, scene);
        }
    }
    /// 注入地形（瓦片 pak + 地形 Map），MainMap 场景据此 blit 真实地形（S6.3）。
    ///
    /// 解析失败保持 None（主地图退回占位网格，不黑屏）。
    pub fn set_terrain(&mut self, pak_bytes: &[u8], map_bytes: &[u8], map_name: &str) {
        if let Ok(atlas) = crate::terrain_atlas::TerrainAtlas::new(pak_bytes) {
            self.terrain_atlas = Some(atlas);
        }
        if let Ok(t) = ti_formats::terrain::TerrainMap::parse(map_name, map_bytes) {
            self.terrain_map = Some(t);
        }
    }
    /// S7.5（feature `hd-8bit`）：HD 渲染路径——复用既有 RGB565 [`Self::render`] 派发，
    /// 再把 RGB565 帧缓冲扩展成 RGBA8（`sync_rgba_from_rgb565`），让 HD 帧就绪。
    ///
    /// S15.4：legacy 逐像素 HD PNG 精灵叠加（`blit_scene_rgba`）已移除；HD 帧 = 纯代码
    /// 绘制 UI 的 RGBA8 扩色版（`08_render_arch.md` 契约 2）。
    ///
    /// 调用方随后调 [`Self::present_hd`] 上屏。
    #[cfg(feature = "hd-8bit")]
    pub fn render_hd(&mut self) {
        self.render();
        self.shell.sync_rgba_from_rgb565();
    }
    /// S7.5（feature `hd-8bit`）：RGBA8 HD 帧缓冲上屏（同 `present` 上传策略，真彩无 565 量化）。
    #[cfg(feature = "hd-8bit")]
    pub fn present_hd(&mut self) {
        self.shell.present_rgba();
    }
    /// S7.5（feature `hd-8bit`）：HD RGBA8 缓存条目数（诊断用，未注入时 0）。
    #[cfg(feature = "hd-8bit")]
    #[must_use]
    pub fn atlas_hd_cache_size(&self) -> usize {
        self.atlas.as_ref().map_or(0, SpriteAtlas::hd_cache_size)
    }
    /// S7.5（feature `hd-8bit`）：注入 HD PNG（`f1 → 多帧 RGBA`），未注入时跳过（PCX 回退）。
    #[cfg(feature = "hd-8bit")]
    pub fn load_hd_pngs(&mut self, f1_to_rgba: impl IntoIterator<Item = (u32, Vec<Vec<u8>>)>) {
        if let Some(atlas) = self.atlas.as_mut() {
            atlas.load_hd_pngs(f1_to_rgba);
        }
    }
    /// S7.5（feature `hd-8bit`）：从 `assets_hd_pack/1x/` 目录注入 HD PNG。
    ///
    /// 文件命名：静态精灵 `NN_hd.png`（1 帧）；多帧 `NN_hd_f0.png`~`f4.png`。
    /// 按帧号排序组成 `f1 → 帧列表`，喂 [`Self::load_hd_pngs`]。目录不存在或
    /// 无可解码 PNG 时返回 `false`（PCX 回退，不 panic）。
    /// 仅 native 宿主可用（依赖 `std::fs` 目录枚举）；wasm32 目标不编译。
    #[cfg(all(feature = "hd-8bit", not(target_arch = "wasm32")))]
    pub fn load_hd_pack_from_dir(&mut self, dir: &std::path::Path) -> bool {
        // S10.2：递归 DFS 收集子目录 PNG（assets_hd_pack/1x/{ui,glmap_m01,glmap_m02,
        // glpics,cards}/...）。不引入 walkdir（std 手动递归）。
        fn collect_pngs_recursive(
            root: &std::path::Path,
            png_map: &mut std::collections::BTreeMap<u32, Vec<(u32, Vec<u8>)>>,
        ) {
            let Ok(entries) = std::fs::read_dir(root) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_pngs_recursive(&path, png_map);
                    continue;
                }
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if !std::path::Path::new(name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
                {
                    continue;
                }
                let stem = name.strip_suffix(".png").unwrap_or(name);
                // 静态："NN_hd" → f1=NN, frame=0；多帧："NN_hd_fK" → f1=NN, frame=K
                let (f1_str, frame_str) = match stem.rsplit_once("_f") {
                    Some((a, b)) => (a.strip_suffix("_hd").unwrap_or(a), b),
                    None => (stem.strip_suffix("_hd").unwrap_or(stem), "0"),
                };
                let Ok(f1) = f1_str.parse::<u32>() else {
                    continue;
                };
                let frame_no: u32 = frame_str.parse().unwrap_or(0);
                let Ok(bytes) = std::fs::read(&path) else {
                    continue;
                };
                if let Ok(img) = ti_platform::hd_png::decode_png_rgba(&bytes) {
                    if img.rgba.iter().any(|&b| b > 0) {
                        png_map.entry(f1).or_default().push((frame_no, bytes));
                    }
                }
            }
        }
        let mut png_map: std::collections::BTreeMap<u32, Vec<(u32, Vec<u8>)>> =
            std::collections::BTreeMap::new();
        collect_pngs_recursive(dir, &mut png_map);
        if png_map.is_empty() {
            return false;
        }
        let png_bytes: Vec<(u32, Vec<Vec<u8>>)> = png_map
            .into_iter()
            .map(|(f1, mut frames)| {
                frames.sort_by_key(|(n, _)| *n);
                (f1, frames.into_iter().map(|(_, b)| b).collect())
            })
            .collect();
        self.load_hd_pngs(png_bytes.clone());
        // P2-8：同时喂 terrain_atlas（地形 HD 接线，`assets_hd_pack/1x/` 同档）
        #[cfg(feature = "hd-8bit")]
        if let Some(ta) = self.terrain_atlas.as_mut() {
            let terrain_png: Vec<(u32, Vec<u8>)> = png_bytes
                .iter()
                .filter_map(|(f1, frames)| frames.first().map(|b| (*f1, b.clone())))
                .collect();
            ta.load_hd_pngs(terrain_png);
        }
        true
    }
    /// N7（EXP-048/060）：manifest 驱动的 HD PNG 注入核心逻辑（同步，宿主传 fetch 闭包）。
    ///
    /// 与 native 的 [`Self::load_hd_pack_from_dir`] 同源：注入数 = manifest 全量
    /// （`static_f1` + `multiframe_f1`），不硬编码 f1 数组。
    /// `static_f1` → `NN_hd.png`（1 帧）；`multiframe_f1` → `NN_hd_f0..f4.png`（5 帧）。
    /// 返回实际注入的 `(f1, 帧列表)`（与 `terrain_atlas` 同档喂入）；调用方按
    /// `static_f1 + multiframe_f1` 断言全量（EXP-048/060）。
    #[cfg(feature = "hd-8bit")]
    pub fn load_hd_pngs_from_manifest(
        &mut self,
        static_f1: &[u32],
        multiframe_f1: &[u32],
        fetch: &mut dyn FnMut(&str) -> Option<Vec<u8>>,
    ) -> Vec<(u32, Vec<Vec<u8>>)> {
        let mut mapping: Vec<(u32, Vec<Vec<u8>>)> = Vec::new();
        for f1 in static_f1.iter().chain(multiframe_f1.iter()) {
            let f1 = *f1;
            let mut frames: Vec<Vec<u8>> = Vec::new();
            if multiframe_f1.contains(&f1) {
                for k in 0..5u32 {
                    let name = format!("assets_hd_pack/1x/ui/{f1}_hd_f{k}.png");
                    if let Some(bytes) = fetch(&name) {
                        if let Ok(img) = ti_platform::hd_png::decode_png_rgba(&bytes) {
                            if img.rgba.iter().any(|&b| b > 0) {
                                frames.push(bytes);
                            }
                        }
                    }
                }
            } else {
                let name = format!("assets_hd_pack/1x/ui/{f1}_hd.png");
                if let Some(bytes) = fetch(&name) {
                    if let Ok(img) = ti_platform::hd_png::decode_png_rgba(&bytes) {
                        if img.rgba.iter().any(|&b| b > 0) {
                            frames.push(bytes);
                        }
                    }
                }
            }
            if !frames.is_empty() {
                mapping.push((f1, frames));
            }
        }
        if !mapping.is_empty() {
            self.load_hd_pngs(mapping.clone());
            if let Some(ta) = self.terrain_atlas.as_mut() {
                let terrain_png: Vec<(u32, Vec<u8>)> = mapping
                    .iter()
                    .filter_map(|(f1, frames)| frames.first().map(|b| (*f1, b.clone())))
                    .collect();
                ta.load_hd_pngs(terrain_png);
            }
        }
        mapping
    }
}
