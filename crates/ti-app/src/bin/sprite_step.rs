//! Legacy `.Map` control decode inspector (S15.4：decode-only，无像素写帧 / 不出 PNG)。
//!
//! 架构重构（`docs/process/08_render_arch.md`）后，运行时不再有 legacy 逐像素写帧层，
//! 原「累积 blit 到 176×220 帧 → 4x PNG」的可视化诊断随之失效。本 bin 退化为
//! **纯 legacy 数据检视**：解析 `.Map` 控件记录 + 解码目标记录及之前所有记录的 PCX 精灵
//! （`SpriteAtlas::sprite`，保留诊断解码），逐条报告几何与解码状态（z-order 文件倒序，
//! record 0 先画=最底层）。**不写像素、不出 PNG**——仅作 `legacy/` 逆向参照物的静态检视。
//!
//! 用法：
//!   cargo run -p ti-app --features native-shell --bin sprite-step -- <scene> <record#>

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let scene_name = args.get(1).map_or("index", String::as_str);
    let Some(target_s) = args.get(2) else {
        eprintln!("用法: sprite-step <scene> <record#>");
        return;
    };
    let Ok(target) = target_s.parse() else {
        eprintln!("record# must be a number");
        return;
    };

    let pkg = match std::fs::read("legacy/data/ui.pak") {
        Ok(b) => b,
        Err(e) => {
            eprintln!("ui.pak 读取失败: {e}");
            return;
        }
    };
    let mut atlas = match ti_app::sprite_atlas::SpriteAtlas::new(&pkg) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("atlas 构建失败: {e}");
            return;
        }
    };
    let map_path = format!("legacy/data/{scene_name}.Map");
    let map_bytes = match std::fs::read(&map_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{map_path} 读取失败: {e}");
            return;
        }
    };
    let scene = match ti_formats::mapfile::MapScene::parse(scene_name, &map_bytes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("scene 解析失败: {e}");
            return;
        }
    };
    let controls = match ti_formats::mapfile::MapControl::parse_words(scene_name, &scene.words) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("controls 解析失败: {e}");
            return;
        }
    };
    if target >= controls.len() {
        eprintln!("记录号 {target} 超出范围（共 {0} 条）", controls.len());
        return;
    }

    // 检视 0..=target 的记录（z-order 文件倒序：record 0 先画=最底层）。
    // decode-only：只报告每条的 f1 原点 + 解码后精灵宽高，不写像素。
    let mut decoded: Vec<(usize, u32)> = Vec::new();
    for (i, ctrl) in controls.iter().enumerate() {
        if i > target {
            break;
        }
        if ctrl.f1 == 0 {
            continue;
        }
        match atlas.sprite(ctrl.f1) {
            Some(sprite) => {
                println!(
                    "  [{i}] f1={} origin({x},{y}) sprite {w}x{h} frame{f} f4={fl}",
                    ctrl.f1,
                    x = ctrl.f2,
                    y = ctrl.f3,
                    w = sprite.width,
                    h = sprite.height,
                    f = sprite.frames,
                    fl = ctrl.f4,
                );
                decoded.push((i, ctrl.f1));
            }
            None => println!(
                "  [{i}] f1={} origin({},{}) => no-sprite（解码失败/超映射表）",
                ctrl.f1, ctrl.f2, ctrl.f3
            ),
        }
    }

    let ctrl = &controls[target];
    let sprite_info = match atlas.sprite(ctrl.f1) {
        Some(s) => format!("{}x{}", s.width, s.height),
        None => "none".to_owned(),
    };
    println!(
        "record {target}: f1={} origin({},{}) sprite_size={sprite_info} => {} 条已解码检视（decode-only，无像素写帧）",
        ctrl.f1,
        ctrl.f2,
        ctrl.f3,
        decoded.len()
    );
}
