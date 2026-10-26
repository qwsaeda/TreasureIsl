//! S21.4 岛屿重绘（`main_map` v4 世界画布 z5 层）：`terrain.json` 单源驱动的
//! 岛底色 / 墨线海岸 / 礁石浅滩环 / 陆内确定性纹理（[`main_map`] 场景调 `draw_board`）。
//!
//! 可见块口径：等距格底 = `2tw × 2th` 矩形，但**下半被后画的邻块覆盖** ⇒ 每格的可见
//! 面 = 顶半 `2tw × th`；装饰（墨线/浅滩环/纹理）一律画在可见块上，坐标与
//! `chrome::iso_cell_fit` 同源（EXP-067 正逆成对）。
//!
//! 可见块的 4 边邻居（共享整边）：上 = `(r-1,c)`/`(r,c-1)`、下 = `(r+1,c)`/`(r,c+1)`、
//! 左 = `(r+1,c-1)`、右 = `(r-1,c+1)`（半砖错位布局）。
use ti_core::data;
use ti_core::data::SceneLayout;

use crate::scenes::SceneHost;
use crate::scenes::chrome;

/// S18.9：等距棋盘绘制 —— **按地形取色**（`assets/data/terrain.json` 数据 → theme 令牌）。
///
/// 地形表缺失 / 越界 = 回退 v2 单色（不 panic、不空图）。陆地按 (r+c) 奇偶用图例 `alt` 交错。
/// S21.4 岛屿重绘：只画陆/礁/宝格（海洋格不画 → 巨型蓝菱形消失）+ 墨线海岸描边 +
/// 礁石浅滩环 + 陆内确定性纹理（`terrain.json` 为唯一数据源；`legend.edge` = 边饰色：
/// reef→`sea_shallow` 浅滩环 / land·treasure→`land_sand_light` 陆内纹理）。
///
/// 可见块口径：等距格底 = `2tw × 2th` 矩形，但**下半被后画的邻块覆盖** ⇒ 每格的可见
/// 面 = 顶半 `2tw × th`；装饰（墨线/浅滩环/纹理）一律画在可见块上，坐标与
/// `chrome::iso_cell_fit` 同源（EXP-067 正逆成对）。
///
/// 可见块的 4 边邻居（共享整边）：上 = `(r-1,c)`/`(r,c-1)`、下 = `(r+1,c)`/`(r,c+1)`、
/// 左 = `(r+1,c-1)`、右 = `(r-1,c+1)`（半砖错位布局）。
const ISLAND_INK_TOKEN: &str = "wood_warm";
const ISLAND_INK_PX: i32 = 1;
const ISLAND_REEF_SHALLOWS_PX: i32 = 3;
const ISLAND_TEXTURE_LEN: i32 = 4;
const ISLAND_TEXTURE_INSET: i32 = 3;
/// 海洋格码（`terrain.json` legend `S`）；不画 = 岛屿剪影。
const ISLAND_SEA_CODE: char = 'S';
/// 礁石格码（legend `R`）。
const ISLAND_REEF_CODE: char = 'R';

/// 确定性格哈希（FNV-1a 变体；**禁 RNG** → 陆内纹理在快照/像素对拍中可复现）。
fn island_cell_hash(r: i32, c: i32) -> u32 {
    let mut h: u32 = 2_166_136_261;
    for b in r.to_le_bytes().into_iter().chain(c.to_le_bytes()) {
        h ^= u32::from(b);
        h = h.wrapping_mul(16_777_619);
    }
    h
}

/// 画岛屿（`terrain.json` 驱动）：底色 → 墨线海岸 → 礁石浅滩环 → 陆内纹理。
/// S21.4 岛屿绘制上下文（地形查询 + 主题色 + 墨线色）。
struct IslandCtx<'a> {
    /// 地形表（缺 = 全海 ⇒ 不画）。
    terrain: Option<&'a data::TerrainDb>,
    /// 当前地图 id（缺 = 全海 ⇒ 不画）。
    map_id: Option<&'a str>,
    /// 主题（色令牌）。
    theme: &'a ti_render::theme::Theme,
    /// 墨线海岸色（`wood_warm` 暖墨棕；缺令牌 = 回退面板底色）。
    ink: u16,
}

impl IslandCtx<'_> {
    /// 地形码（缺表/缺图/越界 = 海）。
    fn code_at(&self, r: i32, c: i32) -> char {
        let (Some(t), Some(mid)) = (self.terrain, self.map_id) else {
            return ISLAND_SEA_CODE;
        };
        let (Ok(rr), Ok(cc)) = (u32::try_from(r), u32::try_from(c)) else {
            return ISLAND_SEA_CODE;
        };
        t.code_at(mid, rr, cc).unwrap_or(ISLAND_SEA_CODE)
    }

    /// 是否海洋格（不画 → 岛屿剪影）。
    fn is_sea(&self, r: i32, c: i32) -> bool {
        self.code_at(r, c) == ISLAND_SEA_CODE
    }

    /// 图例（地形类别 → 令牌）。
    fn legend(&self, code: char) -> Option<&data::TerrainLegend> {
        self.terrain.and_then(|t| t.legend_of(code))
    }

    /// 图例令牌色（缺令牌 = 面板底色回退）。
    fn token_color(&self, token: &str) -> u16 {
        self.theme.by_name(token).unwrap_or(crate::menu_shell::COLOR_MENU_BG)
    }
}

/// 画岛屿（`terrain.json` 驱动）：底色 → 墨线海岸 → 礁石浅滩环 → 陆内纹理。
pub(crate) fn draw_board(
    host: &mut dyn SceneHost,
    layout: &data::SceneLayout,
    gw: u32,
    gw_i: i32,
    tw: i32,
    th: i32,
) {
    if gw_i <= 0 || tw <= 0 || th <= 0 {
        return;
    }
    // S26.2 岛屿图片化：`island_m0X`（`scripts/ui_islands.py` 按同 `board_fit` 几何生成）
    // 1:1 贴入 `world_canvas`（节点牌/航线/船仍 = 代码层，命中判定不破）；
    // 资产缺失 / 键缺失 / 地图 id 无对应资产 = 回退代码绘制（不 panic、不空图）。
    if let Some(mid) = host.map_slot().map(crate::game_app::map_slot_id) {
        if let Some(path) = chrome::island_asset(mid) {
            if chrome::asset_present(path) {
                if let Some(c) = layout.extra.get(chrome::keys::WORLD_CANVAS) {
                    host.blit_image(path, c.x, c.y, c.w, c.h);
                    return;
                }
            }
        }
    }
    let theme = host.theme().clone();
    let ctx = IslandCtx {
        terrain: data::terrain().ok(),
        map_id: host.map_slot().map(crate::game_app::map_slot_id),
        theme: &theme,
        ink: theme.by_name(ISLAND_INK_TOKEN).unwrap_or(crate::menu_shell::COLOR_MENU_BG),
    };
    draw_island_bases(host, layout, gw, gw_i, tw, th, &ctx);
    draw_island_decor(host, layout, gw, gw_i, tw, th, &ctx);
}

/// pass 1：底色（陆/礁/宝格；海洋格跳过 → 岛屿剪影）。
///
/// v4 岛屿 = **暖砂剪影**（S21.3 §0）：`legend.alt` 的 (r+c) 奇偶棋盘变体退役
/// （v3「现代棋盘感」残留），陆面统一 `token` 色 + 确定性纹理；`alt` 字段保留
/// （不删数据，S21.8 存量豁免清零时统一处置）。
fn draw_island_bases(
    host: &mut dyn SceneHost,
    layout: &SceneLayout,
    gw: u32,
    gw_i: i32,
    tw: i32,
    th: i32,
    ctx: &IslandCtx<'_>,
) {
    for r in 0..gw_i {
        for c in 0..gw_i {
            let code = ctx.code_at(r, c);
            if code == ISLAND_SEA_CODE {
                continue;
            }
            let (sx, sy) = chrome::iso_cell_fit(r, c, layout, gw);
            let base = ctx
                .legend(code)
                .map_or_else(|| crate::menu_shell::COLOR_MENU_BG, |l| ctx.token_color(&l.token));
            host.fill_rect(sx, sy, tw * 2, th * 2, base);
        }
    }
}

/// pass 2：装饰（墨线海岸 / 礁石浅滩环 / 陆内纹理）——画在可见块（顶半）上，
/// 顶层绘制 ⇒ 不被邻块底色覆盖（海岸线连续）。
fn draw_island_decor(
    host: &mut dyn SceneHost,
    layout: &SceneLayout,
    gw: u32,
    gw_i: i32,
    tw: i32,
    th: i32,
    ctx: &IslandCtx<'_>,
) {
    let bw = tw * 2;
    let bh = th;
    for r in 0..gw_i {
        for c in 0..gw_i {
            let code = ctx.code_at(r, c);
            if code == ISLAND_SEA_CODE {
                continue;
            }
            let (sx, sy) = chrome::iso_cell_fit(r, c, layout, gw);
            let edge_color = ctx.legend(code).map(|l| ctx.token_color(&l.edge));
            let up_sea = ctx.is_sea(r - 1, c) || ctx.is_sea(r, c - 1);
            let down_sea = ctx.is_sea(r + 1, c) || ctx.is_sea(r, c + 1);
            let left_sea = ctx.is_sea(r + 1, c - 1);
            let right_sea = ctx.is_sea(r - 1, c + 1);

            // 墨线海岸描边：**所有非海格**的临海边（岛的外轮廓连续；陆↔礁交界不画线）。
            if up_sea {
                host.fill_rect(sx, sy, bw, ISLAND_INK_PX, ctx.ink);
            }
            if down_sea {
                host.fill_rect(sx, sy + bh - ISLAND_INK_PX, bw, ISLAND_INK_PX, ctx.ink);
            }
            if left_sea {
                host.fill_rect(sx, sy, ISLAND_INK_PX, bh, ctx.ink);
            }
            if right_sea {
                host.fill_rect(sx + bw - ISLAND_INK_PX, sy, ISLAND_INK_PX, bh, ctx.ink);
            }

            if code == ISLAND_REEF_CODE {
                // 礁石浅滩环：向可见块外扩的浅滩带（`legend.edge` = sea_shallow）。
                let shallow = edge_color.unwrap_or(ctx.theme.sea_shallow);
                if up_sea {
                    host.fill_rect(
                        sx,
                        sy - ISLAND_REEF_SHALLOWS_PX,
                        bw,
                        ISLAND_REEF_SHALLOWS_PX,
                        shallow,
                    );
                }
                if down_sea {
                    host.fill_rect(sx, sy + bh, bw, ISLAND_REEF_SHALLOWS_PX, shallow);
                }
                if left_sea {
                    host.fill_rect(
                        sx - ISLAND_REEF_SHALLOWS_PX,
                        sy,
                        ISLAND_REEF_SHALLOWS_PX,
                        bh,
                        shallow,
                    );
                }
                if right_sea {
                    host.fill_rect(sx + bw, sy, ISLAND_REEF_SHALLOWS_PX, bh, shallow);
                }
                continue;
            }

            // 陆内确定性纹理：哈希定点的短划（`legend.edge` = land_sand_light 浅砂颗粒）。
            let Some(tex) = edge_color else { continue };
            let span_x = (bw - ISLAND_TEXTURE_INSET * 2 - ISLAND_TEXTURE_LEN).max(1);
            let span_y = (bh - ISLAND_TEXTURE_INSET * 2).max(1);
            for h in [island_cell_hash(r, c), island_cell_hash(c, r)] {
                let ox = ISLAND_TEXTURE_INSET
                    + i32::try_from(h % u32::try_from(span_x).unwrap_or(1)).unwrap_or(0);
                let oy = ISLAND_TEXTURE_INSET
                    + i32::try_from((h >> 8) % u32::try_from(span_y).unwrap_or(1)).unwrap_or(0);
                host.fill_rect(sx + ox, sy + oy, ISLAND_TEXTURE_LEN, 1, tex);
            }
        }
    }
}
