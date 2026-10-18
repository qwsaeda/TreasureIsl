//! Element-level legacy `.Map` control inspector (S15.4：decode-only，无像素写帧)。
//!
//! 架构重构（`docs/process/08_render_arch.md`）后，运行时不再有 legacy 逐像素写帧层。
//! 本诊断 bin 退化为**纯 legacy 数据检视**：解析 `.Map` 控件记录 + 解码 PCX 精灵
//! （`SpriteAtlas::sprite`，保留诊断解码），逐条报告「记录号 | f1 | 原点(f2,f3) | 精灵宽高 |
//! 是否在 176×220 legacy 帧内 | §4 版面区归属（legacy 176 坐标）」。**不写任何像素、
//! 不做 1080 映射、不出 PNG**——仅作为逆向参照物（`legacy/`）的静态检视工具。
//!
//! Usage:
//!   cargo run -p ti-app --features native-shell --bin element-dump -- <scene>
//!
//! 版面区清单是 legacy 176×220 坐标（`.Map` 逆向参照口径），本 bin 独立维护一份。

/// §4 版面区清单（legacy 176×220 坐标）：场景名 → ((x0,y0,x1,y1), 区名)。
type LayoutZone = ((u32, u32, u32, u32), &'static str);

fn layout_zones(scene: &str) -> &'static [LayoutZone] {
    match scene {
        "index" => &[
            ((0, 0, 176, 40), "标题条"),
            ((16, 48, 160, 160), "菜单列表区"),
            ((4, 208, 172, 220), "状态行"),
        ],
        "selectrole" | "select_role" => {
            &[((0, 0, 176, 40), "标题条"), ((16, 48, 160, 160), "角色列表区")]
        }
        "selectmap" | "select_map" => {
            &[((0, 0, 176, 40), "标题条"), ((16, 48, 160, 160), "地图列表区")]
        }
        "main" | "main_map" => &[
            ((0, 0, 176, 24), "标题条"),
            ((0, 32, 176, 144), "瓦片区"),
            ((4, 152, 172, 176), "HUD 区"),
        ],
        "vs" => &[
            ((0, 0, 176, 24), "标题条"),
            ((16, 32, 160, 40), "玩家血条区"),
            ((16, 56, 160, 64), "BOSS 血条区"),
        ],
        "msgs" | "message" => &[
            ((0, 0, 176, 20), "标题条"),
            ((16, 32, 160, 184), "正文区"),
            ((4, 208, 172, 220), "提示行"),
        ],
        "saveload" => &[((0, 0, 176, 20), "标题条"), ((16, 48, 160, 160), "槽位列表区")],
        _ => &[],
    }
}

/// legacy 176×220 帧尺寸（仅本诊断 bin 用于「是否出帧」判定，非运行时坐标）。
const LEGACY_W: u32 = 176;
const LEGACY_H: u32 = 220;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let scene_name = args.get(1).map_or("index", String::as_str);
    let map_path = format!("legacy/data/{scene_name}.Map");

    let Ok(pkg) = std::fs::read("legacy/data/ui.pak") else {
        eprintln!("ui.pak read failed");
        std::process::exit(1);
    };
    let mut atlas = match ti_app::sprite_atlas::SpriteAtlas::new(&pkg) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("atlas build failed: {e}");
            std::process::exit(1);
        }
    };
    let Ok(map_bytes) = std::fs::read(&map_path) else {
        eprintln!("{map_path} read failed");
        std::process::exit(1);
    };
    let scene = match ti_formats::mapfile::MapScene::parse(scene_name, &map_bytes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("scene parse failed: {e}");
            std::process::exit(1);
        }
    };
    let controls = match ti_formats::mapfile::MapControl::parse_words(scene_name, &scene.words) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("controls parse failed: {e}");
            std::process::exit(1);
        }
    };

    let zones = layout_zones(scene_name);

    println!(
        "=== element-level legacy .Map inspector (decode-only): {scene_name}, {} controls ===",
        controls.len()
    );
    println!(
        "legacy frame = {LEGACY_W}x{LEGACY_H} (reverse-engineering reference, not a runtime coord system)"
    );
    println!("rec | f1  | origin(x,y)  | spr w x h | in-frame | decode | zone");
    println!("---|----|--------------|-----------|----------|--------|-------");
    let mut no_sprite = 0usize;
    let mut out_of_frame = 0usize;
    for (i, ctrl) in controls.iter().enumerate() {
        if ctrl.f1 == 0 {
            continue;
        }
        let Some(sprite) = atlas.sprite(ctrl.f1) else {
            no_sprite += 1;
            println!(
                "{i:>2} | {f1:<4} | ({x:<3},{y:<3})    | -         | {iof:<8} | no-sprite | -",
                f1 = ctrl.f1,
                x = ctrl.f2,
                y = ctrl.f3,
                iof = if ctrl.f2 < LEGACY_W && ctrl.f3 < LEGACY_H { "in" } else { "OUT" },
            );
            continue;
        };
        let in_frame = ctrl.f2 < LEGACY_W && ctrl.f3 < LEGACY_H;
        if !in_frame {
            out_of_frame += 1;
        }
        let zone = zones
            .iter()
            .find(|zone| {
                let (zx0, zy0, zx1, zy1) = zone.0;
                ctrl.f2 >= zx0 && ctrl.f2 < zx1 && ctrl.f3 >= zy0 && ctrl.f3 < zy1
            })
            .map_or("其他", |zone| zone.1);
        println!(
            "{i:>2} | {f1:<4} | ({x:<3},{y:<3})    | {w:<4} x{h:<3} | {iof:<8} | ok       | {zone}",
            f1 = ctrl.f1,
            x = ctrl.f2,
            y = ctrl.f3,
            w = sprite.width,
            h = sprite.height,
            iof = if in_frame { "in" } else { "OUT" },
        );
    }
    println!("---");
    println!(
        "summary: no-sprite={no_sprite} out-of-legacy-frame={out_of_frame} (of {} non-empty controls)",
        controls.iter().filter(|c| c.f1 != 0).count()
    );
}
