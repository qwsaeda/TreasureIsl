//! Element-level render verification (EXP-076 §7, docs/process/07_render_pipeline.md):
//! per-sprite blit, printing each control's "landing rect + clip verdict + pixel contribution"
//! for before/after comparison.
//!
//! Usage (long-lived diagnostic bin, declared in Cargo.toml as element-dump):
//!   cargo run -p ti-app --features native-shell --bin element-dump -- index
//!
//! Output:
//!   one line per control = record# | f1 | origin(f2,f3) | sprite w x h | clip_rect result | verdict | pixel contribution
//!   out-of-frame (f2>=176 or f3>=220) -> SKIP (post-fix behavior);
//!   partially out -> CLIP to new size; fully in-frame -> OK.
//!
//! Complements `render_snapshot` (gate item 19, full-frame diff):
//! full-frame diff locks "no drift at all"; element_dump locates "which element drifted".

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let scene_name = args.get(1).map(|s| s.as_str()).unwrap_or("index");
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

    use ti_app::render_pipeline::{FB_H, FB_W, clip_rect};

    println!(
        "=== element-level render check: {scene_name} scene, {0} controls ===",
        controls.len()
    );
    println!("FB size = {FB_W}x{FB_H}");
    println!("rec | f1  | origin(x,y)  | spr w x h | clip rect(x,y,w,h)     | verdict | pixels");
    println!("---|----|--------------|-----------|-----------------------|---------|-------");
    let mut skipped = 0usize;
    let mut clipped = 0usize;
    let mut ok = 0usize;
    for (i, ctrl) in controls.iter().enumerate() {
        if ctrl.f1 == 0 {
            continue;
        }
        let Some(sprite) = atlas.sprite(ctrl.f1) else {
            println!(
                "{i:>2}  | {f1:<4} | ({x:<3},{y:<3})    | -         | -                       | no-sprite | -",
                f1 = ctrl.f1,
                x = ctrl.f2,
                y = ctrl.f3,
            );
            continue;
        };
        let sw = sprite.width as usize;
        let sh = sprite.height as usize;
        match clip_rect(ctrl.f2 as usize, ctrl.f3 as usize, sw, sh) {
            Some(clip) => {
                let mut fb = ti_platform::gfx::Framebuf::new();
                fb.clear(0);
                ti_app::sprite_atlas::SpriteAtlas::blit_sprite(
                    &sprite, ctrl.f2, ctrl.f3, &mut fb, 0,
                );
                let contrib = fb.as_slice().iter().filter(|&&p| p != 0).count();
                let kind = if clip.x == ctrl.f2 as usize && clip.y == ctrl.f3 as usize {
                    "OK"
                } else {
                    "CLIP"
                };
                if kind == "CLIP" {
                    clipped += 1;
                } else {
                    ok += 1;
                }
                println!(
                    "{i:>2}  | {f1:<4} | ({x:<3},{y:<3})    | {sw:<4} x{sh:<3} | ({cx},{cy},{cw},{ch})   | {kind:<4} | {contrib}",
                    f1 = ctrl.f1,
                    x = ctrl.f2,
                    y = ctrl.f3,
                    cx = clip.x,
                    cy = clip.y,
                    cw = clip.w,
                    ch = clip.h,
                );
            }
            None => {
                skipped += 1;
                println!(
                    "{i:>2}  | {f1:<4} | ({x:<3},{y:<3})    | {sw:<4} x{sh:<3} | -                       | SKIP   | 0 (out-of-frame)",
                    f1 = ctrl.f1,
                    x = ctrl.f2,
                    y = ctrl.f3,
                );
            }
        }
    }
    println!("---");
    println!("summary: OK={ok} CLIP={clipped} SKIP(out-of-frame)={skipped}");
    println!(
        "out-of-frame early-exit check: SKIP count > 0 with f2/f3 >= 176/220 (post-fix; old version pulled these to col x=175)"
    );
}
