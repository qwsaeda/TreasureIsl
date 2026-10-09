//! 精灵图集：`ui.pak` 的 120 个 PCX 贴图 + `.Map` 控件记录 → `Framebuf` blit。
//!
//! 寻址链（`docs/design/ui_controls.md` §2.3，F-311 实测）：
//! - `.Map` 控件记录 `f1`（精灵索引 1..120）
//! - → `ui.pak` 的 filenum INI（`f1=N` → `"NN.pcx"`，零填充 2 位）
//! - → `ui.pak` 条目 `name_key("NN.pcx")` → PCX 字节
//! - → `pcx::Image::decode` → 像素 + 调色板
//!
//! **多帧动画（S6.4）**：filenum INI 的 `num` 字段是帧数。实测 120 个精灵中
//! 4 个 `num=5`（14/31/32/37），它们的 PCX 高度 ≈ 5 × 40 —— 帧**纵向堆叠**在
//! 同一张图里。`num=1`（116 个）为静态图。本模块按 `frame_h = height / num`
//! 切帧，并提供 [`SpriteAtlas::advance_animation`] 按 tick 推进帧号。
//!
//! 本模块封装这 4 步，提供 [`SpriteAtlas::blit_scene`] 把某个 `.Map` 场景
//! 的全部控件记录按 `(f2, f3)` 坐标 blit 到帧缓冲（RGB565）。

use std::collections::HashMap;

use ti_formats::lucency::Package;
use ti_formats::mapfile::{MapControl, MapHeader, MapScene};
use ti_formats::pcx::Image;
use ti_platform::gfx::Framebuf;

/// 每帧时长（毫秒）—— 5 帧动画约 10 fps（`docs/gamebook.md` §4c）。
pub const FRAME_MS: u32 = 100;

/// 已解码的精灵（PCX → 像素缓冲 + 256×RGB 调色板）。
#[derive(Debug, Clone)]
pub struct DecodedSprite {
    /// 单帧宽（PCX 图宽）。
    pub width: u32,
    /// 单帧高（`PCX 图高 / num`）。
    pub height: u32,
    /// 帧数（INI `num`，=1 为静态图）。
    pub frames: u32,
    /// 像素索引，`len = width × height × frames`（行主序，帧间堆叠）。
    pub pixels: Vec<u8>,
    /// 256 色调色板（`[i]` = RGB 三元组）。
    pub palette: [[u8; 3]; 256],
}

impl DecodedSprite {
    /// 取第 `frame` 帧的像素行偏移（帧号自动环绕）。
    #[must_use]
    pub fn frame_offset(&self, frame: u32) -> usize {
        if self.frames == 0 {
            return 0;
        }
        let f = frame % self.frames;
        let stride = self.width as usize * self.height as usize;
        f as usize * stride
    }
}

/// 精灵图集：`ui.pak` 容器 + INI 映射 + 解码缓存。
#[derive(Debug, Clone)]
pub struct SpriteAtlas {
    /// `ui.pak` 的完整字节（`read_entry` 需要）。
    pkg_buf: Vec<u8>,
    /// `ui.pak` 已解析索引。
    pkg: Package,
    /// `f1 → (pcx 文件名, 帧数)` 映射（解析 filenum INI）。
    f1_to_name: HashMap<u32, (String, u32)>,
    /// 解码缓存：`f1 → DecodedSprite`。
    cache: HashMap<u32, DecodedSprite>,
    /// 累积经过的毫秒数（由 [`Self::advance_animation`] 推进）。
    elapsed_ms: u32,
}

impl SpriteAtlas {
    /// 从 `ui.pak` 字节构建图集（解析索引 + 找 INI 映射）。
    ///
    /// # Errors
    ///
    /// `ui.pak` 魔数/索引解析失败，或未找到 filenum INI 条目。
    pub fn new(pkg_bytes: &[u8]) -> Result<Self, String> {
        let pkg = Package::parse("ui.pak", pkg_bytes)
            .map_err(|e| format!("ui.pak 解析失败: {e}"))?;

        let mut f1_to_name: HashMap<u32, (String, u32)> = HashMap::new();
        let mut found = false;
        for (i, _) in pkg.entries.iter().enumerate() {
            let Ok(content) = pkg.read_entry(pkg_bytes, i) else {
                continue;
            };
            if content.len() < 4 || content[0] != 0xFF || content[1] != 0xFE {
                continue;
            }
            let mut units: Vec<u16> = Vec::with_capacity(content.len() / 2);
            for chunk in content.chunks_exact(2) {
                units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
            }
            let Ok(text) = String::from_utf16(&units) else {
                continue;
            };
            if Self::parse_filenum_ini(&text, &mut f1_to_name).is_ok() {
                found = true;
                break;
            }
        }

        if !found || f1_to_name.is_empty() {
            return Err("ui.pak 中未找到 filenum INI（f1→pcx 映射表）".to_owned());
        }

        Ok(Self {
            pkg_buf: pkg_bytes.to_vec(),
            pkg,
            f1_to_name,
            cache: HashMap::new(),
            elapsed_ms: 0,
        })
    }

    /// 解析 filenum INI → `f1 → (文件名, 帧数)`。
    ///
    /// `num` 是该精灵的帧数（S6.4 多帧动画）；缺省按 1 帧（静态图）处理。
    fn parse_filenum_ini(
        text: &str,
        out: &mut HashMap<u32, (String, u32)>,
    ) -> Result<(), String> {
        let mut section: Option<u32> = None;
        let mut pending_fn: Option<String> = None;
        let mut pending_num: u32 = 1;

        for raw in text.lines() {
            let line = raw.trim().trim_start_matches('\u{feff}');
            if line.is_empty() {
                continue;
            }
            if let Some(inner) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                // 进入新段前冲刷上一段（filename/num 都在同一段内连续出现）
                if let Some(id) = section.take() {
                    if let Some(name) = pending_fn.take() {
                        out.insert(id, (name, pending_num));
                    }
                }
                pending_num = 1;
                if inner != "filenum" {
                    section = inner.parse::<u32>().ok();
                }
                continue;
            }
            if let Some(rest) = line.strip_prefix("filename=") {
                pending_fn = Some(rest.trim().to_owned());
                continue;
            }
            if let Some(rest) = line.strip_prefix("num=") {
                pending_num = rest.trim().parse::<u32>().unwrap_or(1).max(1);
                continue;
            }
            // 其他键（如 attrib）不参与映射
        }

        // EOF 冲刷：最后一段
        if let Some(id) = section.take() {
            if let Some(name) = pending_fn.take() {
                out.insert(id, (name, pending_num));
            }
        }

        if out.is_empty() {
            return Err("filenum INI 未解析出任何 f1 映射".to_owned());
        }
        Ok(())
    }

    /// 取 `f1` 对应的解码精灵（惰性解码 + 缓存）。
    ///
    /// 多帧精灵（`num > 1`）按 `frame_h = 图高 / num` 切帧存入 `pixels`。
    /// `f1` 不在映射表 / 解码失败 → `None`（调用方跳过该控件）。
    #[must_use]
    pub fn sprite(&mut self, f1: u32) -> Option<DecodedSprite> {
        if let Some(s) = self.cache.get(&f1) {
            return Some(s.clone());
        }
        let (name, frames) = self.f1_to_name.get(&f1)?.clone();
        let entry_idx = self.pkg.find(&name)?;
        let pcx_bytes = self.pkg.read_entry(&self.pkg_buf, entry_idx).ok()?;
        let img = Image::decode(&name, &pcx_bytes).ok()?;

        // S6.4：多帧精灵纵向堆叠 -> frame_h = 图高 / 帧数
        let frame_h = if frames > 0 { u32::from(img.height) / frames } else { u32::from(img.height) };
        let frame_h = frame_h.max(1);
        let sprite = DecodedSprite {
            width: u32::from(img.width),
            height: frame_h,
            frames,
            pixels: img.pixels,
            palette: img.palette,
        };
        self.cache.insert(f1, sprite.clone());
        Some(sprite)
    }

    /// 按 tick 推进全局动画帧（S6.4）。返回当前帧号。
    ///
    /// 调用方每帧调一次；`frames == 1` 的精灵不受影响（取模后恒为 0）。
    pub fn advance_animation(&mut self, delta_ms: u32) -> u32 {
        self.elapsed_ms = self.elapsed_ms.wrapping_add(delta_ms);
        self.frame()
    }

    /// 当前动画帧号 = 累积毫秒 / 每帧时长（由 [`Self::advance_animation`] 推进）。
    #[must_use]
    pub fn frame(&self) -> u32 {
        self.elapsed_ms / FRAME_MS
    }

    /// 把 `.Map` 场景的全部控件记录按 `(f2, f3)` blit 到帧缓冲（RGB565）。
    ///
    /// 多帧精灵取当前动画帧 [`Self::frame`]。
    /// 返回成功 blit 的精灵数量。
    pub fn blit_scene(&mut self, scene: &MapScene, fb: &mut Framebuf) -> u32 {
        self.blit_scene_frame(scene, fb, self.frame())
    }

    /// 按指定帧号 blit 场景（测试与定格渲染用）。
    pub fn blit_scene_frame(
        &mut self,
        scene: &MapScene,
        fb: &mut Framebuf,
        frame: u32,
    ) -> u32 {
        let Ok(controls) = MapControl::parse_words("scene.Map", &scene.words) else {
            return 0;
        };
        let mut blitted = 0u32;
        for ctrl in &controls {
            if ctrl.f1 == 0 {
                continue;
            }
            let Some(sprite) = self.sprite(ctrl.f1) else {
                continue;
            };
            Self::blit_sprite(&sprite, ctrl.f2, ctrl.f3, fb, frame);
            blitted += 1;
        }
        blitted
    }

    /// 单个精灵 blit（RGB565，越界裁剪，置脏行带）。`frame` 自动环绕。
    fn blit_sprite(sprite: &DecodedSprite, x: u32, y: u32, fb: &mut Framebuf, frame: u32) {
        const FB_W: usize = 176;
        const FB_H: usize = 220;
        let ox = (x as usize).min(FB_W - 1);
        let oy = (y as usize).min(FB_H - 1);
        let sw = sprite.width as usize;
        let sh = sprite.height as usize;
        let fbase = sprite.frame_offset(frame);
        let fstride = sw * sh;
        let px = fb.as_mut_slice();
        for row in 0..sh {
            let fy = oy + row;
            if fy >= FB_H {
                break;
            }
            for col in 0..sw {
                let fx = ox + col;
                if fx >= FB_W {
                    break;
                }
                let idx = sprite
                    .pixels
                    .get(fbase + row * sw + col)
                    .copied()
                    .unwrap_or(0) as usize;
                let [r, g, b] = sprite.palette[idx];
                let rgb565: u16 =
                    (u16::from(r) >> 3) << 11 | (u16::from(g) >> 2) << 5 | u16::from(b) >> 3;
                let pos = fy * FB_W + fx;
                if let Some(slot) = px.get_mut(pos) {
                    *slot = rgb565;
                }
            }
        }
        let w = sw.min(FB_W.saturating_sub(ox));
        let h = sh.min(FB_H.saturating_sub(oy));
        fb.mark_dirty(
            i32::try_from(ox).unwrap_or(0),
            i32::try_from(oy).unwrap_or(0),
            i32::try_from(w).unwrap_or(0),
            i32::try_from(h).unwrap_or(0),
        );
    }

    /// 映射表大小（f1 条目数）。
    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.f1_to_name.len()
    }

    /// 已解码缓存条目数（性能统计用）。
    #[must_use]
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// 帧数 > 1 的精灵数（动画统计用）。
    #[must_use]
    pub fn animated_count(&self) -> usize {
        self.f1_to_name.values().filter(|(_, n)| *n > 1).count()
    }
}

/// 把 `MapHeader` 的字段直接暴露（避免构造 `MapScene` 时的字段访问器依赖）。
#[allow(dead_code)]
fn _map_header(_h: &MapHeader) {
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 从仓库根 `data/ui.pak` 读真实包（测试用；无此文件则全链测试 SKIP）。
    fn real_pkg() -> Option<Vec<u8>> {
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .map_or_else(|| std::path::PathBuf::from("."), std::path::Path::to_path_buf);
        std::fs::read(base.join("data").join("ui.pak")).ok()
    }

    /// 取真实包并构建图集；缺失则 `SKIP` 测试。
    fn atlas_or_skip() -> Option<SpriteAtlas> {
        let pkg = real_pkg()?;
        match SpriteAtlas::new(&pkg) {
            Ok(a) => Some(a),
            Err(e) => {
                eprintln!("SKIP: {e}");
                None
            }
        }
    }

    #[test]
    fn atlas_end_to_end_with_real_pkg() {
        let Some(mut atlas) = atlas_or_skip() else { return };
        assert!(
            atlas.entry_count() >= 100,
            "应有 ≥100 个 f1 映射，实际 {}",
            atlas.entry_count()
        );
        let sprite = atlas.sprite(1).expect("f1=1（01.pcx）应能解码");
        assert!(sprite.width > 0 && sprite.height > 0);
        assert_eq!(
            sprite.pixels.len(),
            usize::try_from(sprite.width * sprite.height * sprite.frames).unwrap_or(0)
        );

        let mut fb = Framebuf::new();
        let scene = MapScene {
            header: MapHeader { width: 176, height: 220, flags: 0, reserved: 0 },
            words: vec![176, 220, 0, 0, 1, 1, 1, 10, 10, 1, 0],
        };
        let blitted = atlas.blit_scene(&scene, &mut fb);
        assert_eq!(blitted, 1, "应 blit 1 个精灵");
        let nonzero = fb.as_slice().iter().filter(|&&p| p != 0).count();
        assert!(nonzero > 0, "blit 后应有非黑像素");
        assert!(fb.has_dirty(), "blit 后应置脏行带");
    }

    #[test]
    fn atlas_missing_f1_returns_none() {
        let Some(mut atlas) = atlas_or_skip() else { return };
        assert!(atlas.sprite(9999).is_none(), "f1=9999 超出映射表 → None");
        assert!(atlas.sprite(0).is_none(), "f1=0 无效 → None");
    }

    #[test]
    fn filenum_ini_parsing() {
        let ini =
            "[filenum]\nfilenum=5\n[1]\nfilename=01.pcx\nnum=1\n[2]\nfilename=02.pcx\nnum=5\n";
        let mut out = HashMap::new();
        SpriteAtlas::parse_filenum_ini(ini, &mut out).expect("应解析成功");
        assert_eq!(out.get(&1), Some(&("01.pcx".to_owned(), 1)));
        assert_eq!(out.get(&2), Some(&("02.pcx".to_owned(), 5)), "num=5 应被解析");
    }

    #[test]
    fn filenum_ini_rejects_empty() {
        let mut out = HashMap::new();
        assert!(SpriteAtlas::parse_filenum_ini("", &mut out).is_err(), "空 INI 应失败");
    }

    /// 越界 blit 不越界写。
    #[test]
    fn blit_clips_to_screen() {
        let Some(mut atlas) = atlas_or_skip() else { return };
        let mut fb = Framebuf::new();
        let scene = MapScene {
            header: MapHeader { width: 176, height: 220, flags: 0, reserved: 0 },
            words: vec![176, 220, 0, 0, 2, 1, 1, 200, 200, 1, 0, 1, 1, 200, 200, 1, 0],
        };
        let _ = atlas.blit_scene(&scene, &mut fb);
    }

    // ── S6.4 多帧动画 ──────────────────────────────────────────────

    #[test]
    fn multi_frame_slicing_from_real_pkg() {
        let Some(mut atlas) = atlas_or_skip() else { return };
        assert!(
            atlas.animated_count() >= 4,
            "实测应有 4 个 num=5 的多帧精灵，实际 {}",
            atlas.animated_count()
        );
        // f1=14（14.pcx）实测 34x199 num=5 -> frames * frame_h = 199
        let sprite = atlas.sprite(14).expect("f1=14 应能解码");
        assert_eq!(sprite.frames, 5, "14.pcx 应为 5 帧");
        assert_eq!(
            sprite.frames * sprite.height,
            199,
            "帧高切分应还原图高 199"
        );
        let still = atlas.sprite(1).expect("f1=1 应能解码");
        assert_eq!(still.frames, 1, "静态精灵应为 1 帧");
    }

    #[test]
    fn frame_offset_wraps_around() {
        let Some(mut atlas) = atlas_or_skip() else { return };
        let Some(s) = atlas.sprite(14) else {
            eprintln!("SKIP: f1=14 不可用");
            return;
        };
        let stride = s.width as usize * s.height as usize;
        assert_eq!(s.frame_offset(0), 0);
        assert_eq!(s.frame_offset(1), stride);
        assert_eq!(s.frame_offset(4), 4 * stride);
        assert_eq!(s.frame_offset(5), 0, "帧号应环绕");
        assert_eq!(s.frame_offset(7), 2 * stride, "帧号应环绕");
    }

    #[test]
    fn advance_animation_drives_frame() {
        let Some(mut atlas) = atlas_or_skip() else { return };
        assert_eq!(atlas.frame(), 0, "初始帧号应为 0");
        atlas.advance_animation(FRAME_MS);
        assert_eq!(atlas.frame(), 1);
        atlas.advance_animation(FRAME_MS * 2);
        assert_eq!(atlas.frame(), 3);
        atlas.advance_animation(0);
        assert_eq!(atlas.frame(), 3, "delta=0 不应推进");
    }

    #[test]
    fn animated_blit_differs_by_frame() {
        let Some(mut atlas) = atlas_or_skip() else { return };
        if atlas.sprite(14).is_none() {
            eprintln!("SKIP: f1=14 不可用");
            return;
        }
        let scene = MapScene {
            header: MapHeader { width: 176, height: 220, flags: 0, reserved: 0 },
            words: vec![176, 220, 0, 0, 1, 1, 14, 10, 10, 1, 0],
        };
        let mut fb0 = Framebuf::new();
        let mut fb1 = Framebuf::new();
        assert_eq!(atlas.blit_scene_frame(&scene, &mut fb0, 0), 1);
        assert_eq!(atlas.blit_scene_frame(&scene, &mut fb1, 1), 1);
        assert_ne!(
            fb0.as_slice(),
            fb1.as_slice(),
            "5 帧动画的不同帧应 blit 出不同画面"
        );
    }

    #[test]
    fn static_sprite_same_across_frames() {
        let Some(mut atlas) = atlas_or_skip() else { return };
        let scene = MapScene {
            header: MapHeader { width: 176, height: 220, flags: 0, reserved: 0 },
            words: vec![176, 220, 0, 0, 1, 1, 1, 10, 10, 1, 0],
        };
        let mut fb0 = Framebuf::new();
        let mut fb1 = Framebuf::new();
        atlas.blit_scene_frame(&scene, &mut fb0, 0);
        atlas.blit_scene_frame(&scene, &mut fb1, 3);
        assert_eq!(
            fb0.as_slice(),
            fb1.as_slice(),
            "静态精灵（num=1）在任意帧号下画面应相同"
        );
    }
}
