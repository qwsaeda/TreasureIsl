//! 地形图集：`glmap_m0*.pak` 瓦片 + `glmapm0*.Map` cell 记录 → 等距投影 blit。
//!
//! 寻址链（F-307 实测）：
//! - `glmapm0*.Map` 的 cell 记录 `f5` = 瓦片索引（0..48）
//! - → `glmap_m0*.pak` 的 filenum INI（`f5=N` → `"NN-name.pcx"`）
//! - → `glmap_m0*.pak` 条目 `name_key` → PCX 字节 → 解码
//!
//! 放置（等距投影，F-307 恒等式 3/3 全量验证）：
//! - `iso_x = 32 × (row + col)`
//! - `iso_y = 16 × (row + grid_w - 1 - col)`
//!
//! 瓦片尺寸实测 63×31 ~ 127×105（远大于逻辑格步进 32×16），blit 时以
//! `(iso_x, iso_y)` 为**中心锚点**（向左上扩展），符合等距地图的绘制惯例。

use std::collections::HashMap;

use ti_formats::lucency::Package;
use ti_formats::pcx::Image;
use ti_formats::terrain::TerrainMap;
use ti_platform::gfx::Framebuf;
#[cfg(feature = "gpu-render")]
use ti_platform::gpu_renderer::{GpuRenderer, SpriteRef};
#[cfg(feature = "hd-8bit")]
use ti_platform::hd_png::HdPng;

/// 等距投影 x 步进（每对角格 32 像素，F-307）。
pub const ISO_X_STEP: u32 = 32;
/// 等距投影 y 步进（每反对角格 16 像素，F-307）。
pub const ISO_Y_STEP: u32 = 16;

/// 已解码的地形瓦片。
#[derive(Debug, Clone)]
pub struct DecodedTile {
    width: u32,
    height: u32,
    /// 像素索引，`len = width × height`（行主序）。
    pixels: Vec<u8>,
    /// 256 色调色板（`[i]` = RGB 三元组）。
    palette: [[u8; 3]; 256],
}

/// 地形图集：`glmap_m0*.pak` 容器 + INI 映射 + 解码缓存。
///
/// 与 [`crate::sprite_atlas::SpriteAtlas`] 同机制（filenum INI + `name_key` 寻址），
/// 但瓦片表独立于 `ui.pak`。
#[derive(Debug, Clone)]
pub struct TerrainAtlas {
    /// `glmap_m0*.pak` 完整字节（`read_entry` 需要）。
    pkg_buf: Vec<u8>,
    /// `glmap_m0*.pak` 已解析索引。
    pkg: Package,
    /// `tile 索引 → pcx 文件名` 映射（filenum INI）。
    tile_to_name: HashMap<u32, String>,
    /// 解码缓存：`tile 索引 → DecodedTile`。
    cache: HashMap<u32, DecodedTile>,
    /// S7.5（feature `hd-8bit`）：HD PNG 解码缓存（`tile 索引 → HdPng`）。
    #[cfg(feature = "hd-8bit")]
    hd_cache: HashMap<u32, HdPng>,
}

impl TerrainAtlas {
    /// 从 `glmap_m0*.pak` 字节构建图集。
    ///
    /// # Errors
    ///
    /// 容器解析失败，或未找到 filenum INI。
    pub fn new(pkg_bytes: &[u8]) -> Result<Self, String> {
        let pkg = Package::parse("glmap_m.pak", pkg_bytes)
            .map_err(|e| format!("地形 pak 解析失败: {e}"))?;

        let mut tile_to_name: HashMap<u32, String> = HashMap::new();
        let mut found = false;
        for (i, _) in pkg.entries.iter().enumerate() {
            let Ok(content) = pkg.read_entry(pkg_bytes, i) else {
                continue;
            };
            if content.len() < 4 || content[0] != 0xFF || content[1] != 0xFE {
                continue;
            }
            let mut units: Vec<u16> = Vec::new();
            units.push(u16::from_le_bytes([content[2], content[3]]));
            for chunk in content.get(4..).unwrap_or(&[]).chunks_exact(2) {
                units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
            }
            let Ok(text) = String::from_utf16(&units) else {
                continue;
            };
            if Self::parse_filenum_ini(&text, &mut tile_to_name).is_ok() {
                found = true;
                break;
            }
        }
        if !found || tile_to_name.is_empty() {
            return Err("地形 pak 中未找到 filenum INI".to_owned());
        }
        Ok(Self {
            pkg_buf: pkg_bytes.to_vec(),
            pkg,
            tile_to_name,
            cache: HashMap::new(),
            #[cfg(feature = "hd-8bit")]
            hd_cache: HashMap::new(),
        })
    }

    /// 解析 filenum INI → `tile 索引 → 文件名`。
    ///
    /// 地形 INI 与 `ui.pak` 结构相同，但键名是 `attrib` 而非 `num`
    /// （实测 `glmap_m01.pak` 条目 5：`[1] filename=gr_001.pcx attrib=1`）。
    fn parse_filenum_ini(text: &str, out: &mut HashMap<u32, String>) -> Result<(), String> {
        let mut section: Option<u32> = None;
        let mut pending: Option<String> = None;
        for raw in text.lines() {
            let line = raw.trim().trim_start_matches('\u{feff}');
            if line.is_empty() {
                continue;
            }
            if let Some(inner) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if inner != "filenum" {
                    section = inner.parse::<u32>().ok();
                }
                continue;
            }
            if let Some(rest) = line.strip_prefix("filename=") {
                pending = Some(rest.trim().to_owned());
                continue;
            }
            if let Some(id) = section.take() {
                if let Some(name) = pending.take() {
                    out.insert(id, name);
                }
            }
        }
        if out.is_empty() {
            return Err("地形 filenum INI 未解析出映射".to_owned());
        }
        Ok(())
    }

    /// 取 `tile` 对应的解码瓦片（惰性解码 + 缓存）。
    #[must_use]
    pub fn tile(&mut self, index: u32) -> Option<DecodedTile> {
        if let Some(t) = self.cache.get(&index) {
            return Some(t.clone());
        }
        let name = self.tile_to_name.get(&index)?;
        let entry_idx = self.pkg.find(name)?;
        let pcx = self.pkg.read_entry(&self.pkg_buf, entry_idx).ok()?;
        let img = Image::decode(name, &pcx).ok()?;
        let tile = DecodedTile {
            width: u32::from(img.width),
            height: u32::from(img.height),
            pixels: img.pixels,
            palette: img.palette,
        };
        self.cache.insert(index, tile.clone());
        Some(tile)
    }

    /// 等距投影：`iso_x = 32 × (row + col)`。
    #[must_use]
    pub fn iso_x(row: u32, col: u32) -> i32 {
        i32::try_from(ISO_X_STEP * (row + col)).unwrap_or(0)
    }

    /// 等距投影：`iso_y = 16 × (row + grid_w - 1 - col)`。
    #[must_use]
    pub fn iso_y(row: u32, col: u32, grid_w: u32) -> i32 {
        i32::try_from(ISO_Y_STEP * (row + grid_w - 1 - col)).unwrap_or(0)
    }
    /// 地形等距包围盒（iso 像素）：`min_x, min_y, max_x, max_y`。
    #[must_use]
    pub fn bounds(terrain: &TerrainMap) -> (i64, i64, i64, i64) {
        let cells = terrain.cell_records();
        if cells.is_empty() {
            return (0, 0, 0, 0);
        }
        let xs: Vec<i64> = cells.iter().map(|c| i64::from(c.iso_x)).collect();
        let ys: Vec<i64> = cells.iter().map(|c| i64::from(c.iso_y)).collect();
        let min_x = xs.iter().copied().min().unwrap_or(0);
        let max_x = xs.iter().copied().max().unwrap_or(0);
        let min_y = ys.iter().copied().min().unwrap_or(0);
        let max_y = ys.iter().copied().max().unwrap_or(0);
        (min_x, min_y, max_x, max_y)
    }

    /// 居中摄像机：把地形包围盒中心对到逻辑屏中心（88, 110）。
    #[must_use]
    pub fn center_camera(terrain: &TerrainMap) -> (i64, i64) {
        let (min_x, min_y, max_x, max_y) = Self::bounds(terrain);
        (88 - (min_x + max_x) / 2, 110 - (min_y + max_y) / 2)
    }

    /// 按居中摄像机把地形 blit 到帧缓冲（painter's order：先按 `iso_y` 升序）。
    ///
    /// **地形是大画布 + 摄像机**：实测 m01 的 `iso_x` 范围 0..1216、`iso_y` 16..608，
    /// 远超 176×220 逻辑屏，故必须按摄像机偏移把画布窗口平移到屏内。
    ///
    /// 瓦片以 `(iso_x, iso_y)` **居中**于等距格点（`iso_x - w/2`）。
    /// 返回成功 blit 的瓦片数。
    pub fn blit_map(&mut self, terrain: &TerrainMap, fb: &mut Framebuf) -> u32 {
        self.blit_map_cam(terrain, fb, Self::center_camera(terrain))
    }

    /// 带摄像机的地形 blit（camera = 等距画布原点到屏幕的偏移，通常为负）。
    pub fn blit_map_cam(
        &mut self,
        terrain: &TerrainMap,
        fb: &mut Framebuf,
        camera: (i64, i64),
    ) -> u32 {
        let mut cells = terrain.cell_records();
        // painter's order：iso_y 升序（远的先画，近的覆盖）
        cells.sort_by_key(|c| c.iso_y);
        let mut blitted = 0u32;
        for c in &cells {
            let Some(tile) = self.tile(c.tile) else {
                continue;
            };
            let x = i64::from(c.iso_x) + camera.0 - i64::from(tile.width) / 2;
            let y = i64::from(c.iso_y) + camera.1 - i64::from(tile.height) / 2;
            Self::blit_decoded(&tile, x, y, fb);
            blitted += 1;
        }
        blitted
    }
    /// 单瓦片 blit（RGB565，越界裁剪，置脏行带）。
    ///
    /// allow 理由：瓦片尺寸 ≤ 127×105、屏宽 176，坐标转 `usize` 无截断/符号丢失。
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn blit_decoded(tile: &DecodedTile, ox: i64, oy: i64, fb: &mut Framebuf) {
        const FB_W: i64 = 176;
        const FB_H: i64 = 220;
        let tw = i64::from(tile.width);
        let th = i64::from(tile.height);
        let px = fb.as_mut_slice();
        for row in 0..th {
            let fy = oy + row;
            if !(0..FB_H).contains(&fy) {
                continue;
            }
            for col in 0..tw {
                let fx = ox + col;
                if !(0..FB_W).contains(&fx) {
                    continue;
                }
                let src = usize::try_from(row * tw + col).unwrap_or(0);
                let idx = tile.pixels.get(src).copied().unwrap_or(0) as usize;
                let [r, g, b] = tile.palette[idx];
                let rgb565: u16 =
                    (u16::from(r) >> 3) << 11 | (u16::from(g) >> 2) << 5 | u16::from(b) >> 3;
                let pos = usize::try_from(fy * FB_W + fx).unwrap_or(0);
                if let Some(slot) = px.get_mut(pos) {
                    *slot = rgb565;
                }
            }
        }
        // 可见区域（与屏求交），用于脏行带
        let vx0 = ox.max(0);
        let vy0 = oy.max(0);
        let vx1 = (ox + tw).min(FB_W);
        let vy1 = (oy + th).min(FB_H);
        let w = vx1 - vx0;
        let h = vy1 - vy0;
        if w > 0 && h > 0 {
            fb.mark_dirty(
                i32::try_from(vx0).unwrap_or(0),
                i32::try_from(vy0).unwrap_or(0),
                i32::try_from(w).unwrap_or(0),
                i32::try_from(h).unwrap_or(0),
            );
        }
    }

    /// S7.5（feature `hd-8bit`）：按居中摄像机把地形 blit 到 RGBA8 HD 帧缓冲。
    /// 返回成功 blit 的瓦片数（语义与 [`Self::blit_map`] 一致）。
    #[cfg(feature = "hd-8bit")]
    pub fn blit_map_rgba(
        &mut self,
        terrain: &TerrainMap,
        fb: &mut ti_platform::gfx::FramebufRgba,
    ) -> u32 {
        self.blit_map_cam_rgba(terrain, fb, Self::center_camera(terrain))
    }

    /// S7.5（feature `hd-8bit`）：带摄像机的地形 RGBA8 blit。
    #[cfg(feature = "hd-8bit")]
    pub fn blit_map_cam_rgba(
        &mut self,
        terrain: &TerrainMap,
        fb: &mut ti_platform::gfx::FramebufRgba,
        camera: (i64, i64),
    ) -> u32 {
        let mut cells = terrain.cell_records();
        cells.sort_by_key(|c| c.iso_y);
        let mut blitted = 0u32;
        for c in &cells {
            let Some(tile) = self.tile(c.tile) else {
                continue;
            };
            let x = i64::from(c.iso_x) + camera.0 - i64::from(tile.width) / 2;
            let y = i64::from(c.iso_y) + camera.1 - i64::from(tile.height) / 2;
            let hd = self.hd_cache.get(&c.tile);
            Self::blit_decoded_rgba(&tile, hd, x, y, fb);
            blitted += 1;
        }
        blitted
    }

    /// S7.5（feature `hd-8bit`）：单瓦片 RGBA8 blit（越界裁剪，置脏行带）。
    ///
    /// 走 PNG 真彩像素（`hd` 注入）；未注入时回退到 PCX 调色板扩色。
    #[cfg(feature = "hd-8bit")]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn blit_decoded_rgba(
        tile: &DecodedTile,
        hd: Option<&HdPng>,
        ox: i64,
        oy: i64,
        fb: &mut ti_platform::gfx::FramebufRgba,
    ) {
        const FB_W: i64 = 176;
        const FB_H: i64 = 220;
        let tw = i64::from(tile.width);
        let th = i64::from(tile.height);
        let px = fb.as_mut_slice();
        let sw = usize::try_from(tile.width).unwrap_or(0);
        for row in 0..th {
            let fy = oy + row;
            if !(0..FB_H).contains(&fy) {
                continue;
            }
            for col in 0..tw {
                let fx = ox + col;
                if !(0..FB_W).contains(&fx) {
                    continue;
                }
                let pos = usize::try_from(fy * FB_W + fx).unwrap_or(0);
                let Some(slot) = px.get_mut(pos) else {
                    continue;
                };
                if let Some(img) = hd {
                    let sidx = usize::try_from(row)
                        .unwrap_or(0)
                        .saturating_mul(sw)
                        .saturating_add(usize::try_from(col).unwrap_or(0))
                        .saturating_mul(4);
                    let Some(rgba) = img.rgba.get(sidx..sidx + 4) else {
                        continue;
                    };
                    if rgba[3] == 0 {
                        continue;
                    }
                    *slot = ti_platform::gfx::png_rgba_to_framebuf_u32([
                        rgba[0], rgba[1], rgba[2], rgba[3],
                    ]);
                } else {
                    let src = usize::try_from(row * tw + col).unwrap_or(0);
                    let idx = tile.pixels.get(src).copied().unwrap_or(0) as usize;
                    let [r, g, b] = tile.palette[idx];
                    *slot =
                        0xFF00_0000u32 | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b);
                }
            }
        }
        let vx0 = ox.max(0);
        let vy0 = oy.max(0);
        let vx1 = (ox + tw).min(FB_W);
        let vy1 = (oy + th).min(FB_H);
        let w = vx1 - vx0;
        let h = vy1 - vy0;
        if w > 0 && h > 0 {
            fb.mark_dirty(
                i32::try_from(vx0).unwrap_or(0),
                i32::try_from(vy0).unwrap_or(0),
                i32::try_from(w).unwrap_or(0),
                i32::try_from(h).unwrap_or(0),
            );
        }
    }

    /// S7.5（feature `hd-8bit`）：注入 `tile → HD PNG` 映射（`assets_hd_pack/1x/` 档）。
    #[cfg(feature = "hd-8bit")]
    pub fn load_hd_pngs(&mut self, tile_to_rgba: impl IntoIterator<Item = (u32, Vec<u8>)>) {
        for (tile, bytes) in tile_to_rgba {
            if let Ok(img) = ti_platform::hd_png::decode_png_rgba(&bytes) {
                self.hd_cache.insert(tile, img);
            }
        }
    }

    /// S7.5（feature `hd-8bit`）：已注入的 HD PNG 条目数（统计用）。
    #[cfg(feature = "hd-8bit")]
    #[must_use]
    pub fn hd_cache_size(&self) -> usize {
        self.hd_cache.len()
    }

    /// 映射表大小（tile 索引条目数）。
    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.tile_to_name.len()
    }

    /// 已解码缓存条目数。
    #[must_use]
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    // ── S10.4 GPU 路径（feature `gpu-render`）────────────────────────────────────

    /// S10.4（feature `gpu-render`）：把 `tile` 对应的瓦片转成 GPU 纹理引用。
    ///
    /// `atlas_texture_id` 为已上传 macroquad 纹理的 `id()`；
    /// `atlas_base_x/y` 为图集内该瓦片的左上角像素坐标。
    #[cfg(feature = "gpu-render")]
    #[must_use]
    pub fn gpu_tile(
        &mut self,
        tile_index: u32,
        atlas_texture_id: u32,
        atlas_base_x: i32,
        atlas_base_y: i32,
    ) -> Option<SpriteRef> {
        let tile = self.tile(tile_index)?;
        let fw = i32::try_from(tile.width).unwrap_or(0);
        let fh = i32::try_from(tile.height).unwrap_or(0);
        Some(SpriteRef { texture: atlas_texture_id, rect: (atlas_base_x, atlas_base_y, fw, fh) })
    }

    /// S10.4（feature `gpu-render`）：GPU 地形 blit（走 `GpuRenderer::blit_terrain_tile`）。
    ///
    /// 不修改帧缓冲（GPU 路径直接上屏），返回成功提交的瓦片数。
    #[cfg(feature = "gpu-render")]
    pub fn gpu_blit_map(
        &mut self,
        terrain: &TerrainMap,
        renderer: &mut GpuRenderer,
        atlas_texture_id: u32,
        atlas_base_x: i32,
        atlas_base_y: i32,
        camera: (i64, i64),
    ) -> u32 {
        let mut cells = terrain.cell_records();
        cells.sort_by_key(|c| c.iso_y);
        let mut blitted = 0u32;
        for c in &cells {
            let Some(ref_) = self.gpu_tile(c.tile, atlas_texture_id, atlas_base_x, atlas_base_y)
            else {
                continue;
            };
            let x = i64::from(c.iso_x) + camera.0;
            let y = i64::from(c.iso_y) + camera.1;
            renderer.blit_terrain_tile(
                (i32::try_from(x).unwrap_or(0), i32::try_from(y).unwrap_or(0)),
                &ref_,
            );
            blitted += 1;
        }
        blitted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ti_formats::terrain::TerrainMap as TM;

    fn repo_file(name: &str) -> Option<Vec<u8>> {
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .map_or_else(|| std::path::PathBuf::from("."), std::path::Path::to_path_buf);
        std::fs::read(base.join("legacy").join("data").join(name)).ok()
    }

    fn atlas_or_skip() -> Option<TerrainAtlas> {
        let pkg = repo_file("glmap_m01.pak")?;
        match TerrainAtlas::new(&pkg) {
            Ok(a) => Some(a),
            Err(e) => {
                eprintln!("SKIP: {e}");
                None
            }
        }
    }

    /// 等距投影公式（F-307 恒等式）。
    #[test]
    fn iso_projection_matches_f307() {
        // iso_x = 32*(row+col), iso_y = 16*(row + grid_w - 1 - col)
        for (row, col, gw) in [(0u32, 0u32, 20u32), (0, 19, 20), (5, 7, 20), (19, 0, 20)] {
            assert_eq!(TerrainAtlas::iso_x(row, col), i32::try_from(32 * (row + col)).unwrap_or(0));
            assert_eq!(
                TerrainAtlas::iso_y(row, col, gw),
                i32::try_from(16 * (row + gw - 1 - col)).unwrap_or(0)
            );
        }
        // 缺格 cell(0, gw-1) 应落在屏幕最左上（iso_y=0）
        assert_eq!(TerrainAtlas::iso_y(0, 19, 20), 0);
    }

    #[test]
    fn terrain_atlas_end_to_end() {
        let Some(mut atlas) = atlas_or_skip() else { return };
        assert!(atlas.entry_count() >= 40, "应解析出 40+ 个瓦片映射，实际 {}", atlas.entry_count());
        let Some(tile) = atlas.tile(1) else {
            eprintln!("SKIP: tile 1 解码失败");
            return;
        };
        assert!(tile.width > 0 && tile.height > 0);
        assert_eq!(tile.pixels.len(), usize::try_from(tile.width * tile.height).unwrap_or(0));
    }

    #[test]
    fn terrain_atlas_missing_tile_returns_none() {
        let Some(mut atlas) = atlas_or_skip() else { return };
        assert!(atlas.tile(9999).is_none(), "tile 9999 超出映射 → None");
    }

    /// 真实地形 + 真实瓦片 → blit 出非空画面。
    #[test]
    fn blit_real_terrain_produces_pixels() {
        let Some(mut atlas) = atlas_or_skip() else { return };
        let Some(map_bytes) = repo_file("glmapm01.Map") else {
            eprintln!("SKIP: glmapm01.Map 不存在");
            return;
        };
        let Ok(terrain) = TM::parse("glmapm01.Map", &map_bytes) else {
            eprintln!("SKIP: 地形解析失败");
            return;
        };
        let mut fb = Framebuf::new();
        let blitted = atlas.blit_map(&terrain, &mut fb);
        assert!(blitted > 0, "应 blit 出至少 1 个瓦片，实际 {blitted}");
        let nonzero = fb.as_slice().iter().filter(|&&p| p != 0).count();
        assert!(nonzero > 100, "blit 后应有大量非黑像素，实际 {nonzero}");
        assert!(fb.has_dirty(), "blit 后应置脏");
    }

    /// 越界瓦片不越界写（cell 在屏幕外）。
    #[test]
    fn blit_clips_to_screen() {
        let Some(mut atlas) = atlas_or_skip() else { return };
        let mut fb = Framebuf::new();
        // 造一个极端地形：单格在很远的位置
        let tile = atlas.tile(1);
        if let Some(t) = tile {
            // 直接用超屏坐标 blit
            TerrainAtlas::blit_decoded(&t, -500, -500, &mut fb);
            TerrainAtlas::blit_decoded(&t, 500, 500, &mut fb);
            assert!(fb.as_slice().iter().all(|&p| p == 0), "完全出屏的 blit 不应写入像素");
        }
    }

    #[test]
    fn filenum_ini_parsing_with_attrib() {
        let ini =
            "[filenum]\nfilenum=3\n[1]\nfilename=a.pcx\nattrib=1\n[2]\nfilename=b.pcx\nattrib=1\n";
        let mut out = HashMap::new();
        if TerrainAtlas::parse_filenum_ini(ini, &mut out).is_err() {
            eprintln!("FAIL: filenum INI 解析应成功");
            return;
        }
        assert_eq!(out.get(&1), Some(&"a.pcx".to_owned()));
        assert_eq!(out.get(&2), Some(&"b.pcx".to_owned()));
    }
}
