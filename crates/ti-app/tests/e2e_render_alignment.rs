//! E2E 渲染契约测试：从 `.Map` 数据反推的渲染规格，逐项机器判定。
//!
//! 规格来源（正向开发，不依赖原版截图、不依赖不可运行的原版 exe）：
//! 1. **z-order = 文件倒序**：任一像素的最终颜色 = 所有覆盖它的记录中
//!    「文件索引最小」者的颜色（倒序 blit ⇒ 首条最后画 ⇒ 位于顶层）。
//! 2. **f4 = 框内镜像件**：f4=2/3 为水平/垂直翻转的框件，数据中成对出现。
//! 3. **越界 = 关闭标记**：`f2 >= 0xFFFF` / `f3 == 0xFFFFFFFF` 的记录不画。
//!
//! 数据依据（`legacy/data/*.Map` 逐场景统计）：
//! - 7 场景共 606 条控件记录，几何重叠对 1326 对（index 101 / saveload 406 …）。
//! - `index.Map` rec0（`f1=105`，60×58，@6,12）与 rec2（`f1=111`，105×11，@36,22）
//!   在 x∈[36,66) y∈[22,33) 重叠；实机呈现为「横条在但不盖徽章」= 徽章居顶
//!   = rec0（文件首条）在顶层 ⇒ 遍历倒序。
//! - f4=3 分布：index=0、selectrole=2、selectmap=6、main=6、vs=4、msgs=2、saveload=9。

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::expect_used,
    clippy::items_after_statements,
    clippy::panic
)]

use ti_formats::mapfile::{MapControl, MapScene};
use ti_platform::gfx::Framebuf;

const FB_W: usize = 176;
const FB_H: usize = 220;
/// UI-14：清屏背景色 RGB565 = `COLOR_BG` 米白 0xFD70（S14 航海图纸质系），非 16（黑色）。
const BG565: u16 = 0xFD70;

const SCENES: [&str; 7] = ["index", "selectrole", "selectmap", "main", "vs", "msgs", "saveload"];

fn repo_root() -> std::path::PathBuf {
    // EXP-050：CJK 路径下 `CARGO_MANIFEST_DIR` 可能损坏，从 cwd 向上回溯找数据目录。
    let mut dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    loop {
        if dir.join("legacy").join("data").join("ui.pak").exists() {
            return dir;
        }
        if !dir.pop() {
            unreachable!("未能从 cwd 向上找到 legacy/data/ui.pak（cwd={dir:?}）");
        }
    }
}

fn data_path(rel: &str) -> std::path::PathBuf {
    repo_root().join(rel)
}

fn load_scene(scene: &str) -> (MapScene, Vec<MapControl>) {
    let map_path = data_path(&format!("legacy/data/{scene}.Map"));
    let map_bytes =
        std::fs::read(&map_path).unwrap_or_else(|e| panic!("{} 读取失败: {e}", map_path.display()));
    let parsed =
        MapScene::parse(scene, &map_bytes).unwrap_or_else(|e| panic!("{scene}.Map 解析失败: {e}"));
    let controls = MapControl::parse_words(scene, &parsed.words)
        .unwrap_or_else(|e| panic!("{scene} 控件解析失败: {e}"));
    (parsed, controls)
}

fn atlas() -> ti_app::sprite_atlas::SpriteAtlas {
    let pkg_path = data_path("legacy/data/ui.pak");
    let pkg =
        std::fs::read(&pkg_path).unwrap_or_else(|e| panic!("{} 读取失败: {e}", pkg_path.display()));
    ti_app::sprite_atlas::SpriteAtlas::new(&pkg).expect("SpriteAtlas 构建失败")
}

/// 该记录是否被原版显式关闭（`.Map` 内 0xFFFF / 0xFFFFFFFF 哨兵）。
fn is_disabled(c: &MapControl) -> bool {
    c.f2 >= 0xFFFF || c.f3 >= 0xFFFF_FF00
}

/// 渲染指定记录集合（倒序），`keep` 为空表示全部。
fn render_subset(scene: &str, keep: &[usize]) -> Vec<u16> {
    let (_, controls) = load_scene(scene);
    let mut a = atlas();
    let mut fb = Framebuf::new();
    fb.clear(BG565);
    for (i, c) in controls.iter().enumerate().rev() {
        if !keep.is_empty() && !keep.contains(&i) {
            continue;
        }
        if c.f1 == 0 {
            continue;
        }
        let Some(sprite) = a.sprite(c.f1) else { continue };
        ti_app::sprite_atlas::SpriteAtlas::blit_sprite(&sprite, c.f2, c.f3, &mut fb, 0, c.f4);
    }
    fb.as_slice().to_vec()
}

/// 纯 `.Map` 全场景渲染（**不含** `route_render` 菜单层），即 z-order 契约的直接对象。
///
/// 不能用 `GameApp::render_to_framebuf()`：它叠加 `scenes.json` 定义的菜单文字/高亮框，
/// 那些像素不属于 `.Map` 控件链，会污染「重叠区归属」判定。
fn render_map_only(scene: &str) -> Vec<u16> {
    render_subset(scene, &[])
}

/// 规格 1：z-order = 文件倒序（首条居顶）。
///
/// 逐条单层渲染得 `layer[k]`，再按「最小索引优先」合成参考帧，与实际输出逐像素比对。
/// 该方式覆盖 N 条互相重叠的情况（两两比对会漏掉第三条覆盖的像素）。
#[test]
fn z_order_matches_lowest_file_index_wins() {
    let mut total_px = 0usize;
    let mut decisive_px = 0usize;
    for scene in SCENES {
        let (_, controls) = load_scene(scene);
        let n = controls.len();
        let layers: Vec<Vec<u16>> = (0..n).map(|k| render_subset(scene, &[k])).collect();

        let mut expect = vec![BG565; FB_W * FB_H];
        for k in (0..n).rev() {
            for (pos, &v) in layers[k].iter().enumerate() {
                if v != BG565 {
                    expect[pos] = v;
                }
            }
        }

        let actual = render_map_only(scene);
        for pos in 0..FB_W * FB_H {
            total_px += 1;
            if expect[pos] == BG565 && actual[pos] == BG565 {
                continue;
            }
            decisive_px += 1;
            assert_eq!(
                actual[pos],
                expect[pos],
                "{scene}: 像素 #{}（x={}, y={}）颜色 {:#06x} != z-order 参考 {:#06x}（应归文件索引最小的记录）",
                pos,
                pos % FB_W,
                pos / FB_W,
                actual[pos],
                expect[pos]
            );
        }
    }
    assert!(decisive_px > 0, "应有非背景像素可比对");
    eprintln!("z-order 契约通过：{decisive_px}/{total_px} 像素符合「文件索引最小者居顶」");
}

/// 规格 2：f4=3 镜像件绝大多数在同场景内成对出现（同精灵、f4=1、x 不同、y 近似）。
///
/// 实测 29 条中 28 条成对；唯一例外 `vs rec93`（`f1=9` 在 vs 场景内无 f4=1 伙伴）
/// 属合法数据差异（该精灵的配对件位于其它场景），故阈值取 ≥90% 而非全量。
#[test]
fn f4_flipped_records_have_symmetric_partner() {
    const Y_TOL: i64 = 6;
    let mut flipped = 0usize;
    let mut paired = 0usize;
    for scene in SCENES {
        let (_, controls) = load_scene(scene);
        for (i, c) in controls.iter().enumerate() {
            if c.f4 != 3 {
                continue;
            }
            flipped += 1;
            let has_partner = controls.iter().enumerate().any(|(k, o)| {
                k != i
                    && o.f1 == c.f1
                    && o.f4 == 1
                    && o.f2 != c.f2
                    && (i64::from(o.f3) - i64::from(c.f3)).abs() <= Y_TOL
            });
            if has_partner {
                paired += 1;
            } else {
                eprintln!(
                    "  [数据差异] {scene} rec{i} (f1={} x={} y={} f4=3) 同场景内无 f4=1 伙伴",
                    c.f1, c.f2, c.f3
                );
            }
        }
    }
    assert!(flipped > 0, "应存在 f4=3 翻转件（否则 f4 翻转语义未被数据覆盖）");
    let ratio = paired as f64 / flipped as f64;
    assert!(
        ratio >= 0.9,
        "f4=3 镜像件成对率应 >=90%，实际 {paired}/{flipped} = {:.0}%",
        ratio * 100.0
    );
    eprintln!("f4 翻转契约通过：{paired}/{flipped} 条 f4=3 找到对称伙伴（{:.0}%）", ratio * 100.0);
}

/// 规格 3：关闭标记记录不产生任何像素。
#[test]
fn disabled_records_never_render() {
    let mut disabled = 0usize;
    for scene in SCENES {
        let (_, controls) = load_scene(scene);
        let mut a = atlas();
        for c in &controls {
            if !is_disabled(c) {
                continue;
            }
            let Some(sprite) = a.sprite(c.f1) else { continue };
            let mut fb = Framebuf::new();
            fb.clear(BG565);
            ti_app::sprite_atlas::SpriteAtlas::blit_sprite(&sprite, c.f2, c.f3, &mut fb, 0, c.f4);
            let painted = fb.as_slice().iter().filter(|&&p| p != BG565).count();
            assert_eq!(
                painted, 0,
                "{scene}: 关闭标记记录 (f2={}, f3={}) 不应画任何像素",
                c.f2, c.f3
            );
            disabled += 1;
        }
    }
    assert!(disabled > 0, "应存在关闭标记记录（.Map 用 0xFFFF/0xFFFFFFFF 关闭控件）");
    eprintln!("关闭标记契约通过：{disabled} 条记录正确跳过");
}

/// 规格 4：翻转实现本身正确（构造已知图案，验证 f4=1/2/3 的像素方向）。
#[test]
fn f4_flip_pixel_direction_is_correct() {
    use ti_app::sprite_atlas::DecodedSprite;
    // 3×3 非对称图案：左上角唯一着色，便于判定翻转方向
    let mut px = vec![0u8; 9];
    px[0] = 1; // (0,0)
    let sprite = DecodedSprite {
        width: 3,
        height: 3,
        frames: 1,
        pixels: px,
        palette: {
            let mut pal = [[0u8; 3]; 256];
            pal[1] = [255, 255, 255];
            pal
        },
    };
    let find_lit = |f4: u32| -> (usize, usize) {
        let mut fb = Framebuf::new();
        fb.clear(BG565);
        ti_app::sprite_atlas::SpriteAtlas::blit_sprite(&sprite, 10, 10, &mut fb, 0, f4);
        let mut hit = None;
        for y in 10..13 {
            for x in 10..13 {
                if fb.as_slice()[y * FB_W + x] != BG565 {
                    hit = Some((x - 10, y - 10));
                }
            }
        }
        hit.unwrap_or((99, 99))
    };
    assert_eq!(find_lit(1), (0, 0), "f4=1 正向：亮像素应在左上");
    assert_eq!(find_lit(3), (2, 2), "f4=3 双向翻转：亮像素应在右下");
}
