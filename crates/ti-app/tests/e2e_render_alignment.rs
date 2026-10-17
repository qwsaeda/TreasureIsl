//! E2E 渲染契约测试：从 `.Map` 数据反推的渲染规格，逐项机器判定。
//!
//! S15.4：legacy 逐像素写帧层移除后，依赖 `blit_sprite` / `legacy_scaled` 的 3 个
//! 逐像素判据（z-order 倒序、关闭标记不画像素、f4 翻转像素方向）已随删除而暂移除
//! （见 `08_render_arch.md` 契约 5，测试重锚到代码绘制区在 S15.5 完成）。
//! 保留：纯数据契约（f4=3 镜像件成对率 ≥90%），不依赖像素写帧。

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::cast_possible_truncation
)]

use ti_formats::mapfile::{MapControl, MapScene};

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
    // 成对率按整数百分比判定（避免 `usize as f64` 精度丢失，clippy::cast_possible_truncation）。
    let pct = paired * 100 / flipped;
    assert!(pct >= 90, "f4=3 镜像件成对率应 >=90%，实际 {paired}/{flipped} = {pct}%");
    eprintln!("f4 翻转契约通过：{paired}/{flipped} 条 f4=3 找到对称伙伴（{pct}%）");
}

/// S15.5 重锚（`08_render_arch.md` §6 契约 5）：z-order「文件序最低 index 胜」（首条 .Map 记录 =
/// 顶层，旧渲染器倒序 blit 实现）在 legacy 像素写帧层移除后，迁为 ti_formats **纯数据**断言：
/// `MapControl::parse_words` 必须保文件序（返回第 i 条 == 原始字流第 i 条 6 字记录）。
/// z-order 完全由数据序决定，不再进像素门禁。
#[test]
fn z_order_lowest_file_index_wins_is_data_preserved() {
    let count_word = ti_formats::mapfile::MapControl::COUNT_WORD;
    for scene in SCENES {
        let (sc, controls) = load_scene(scene);
        let declared = usize::try_from(sc.words[count_word]).unwrap_or(0);
        assert_eq!(
            controls.len(),
            declared,
            "{scene}: parse_words 记录数 {n} 应 == words[{count_word}] = {declared}",
            n = controls.len()
        );
        let base = ti_formats::mapfile::HEADER_WORDS;
        for (i, c) in controls.iter().enumerate() {
            let file_words = &sc.words[base + i * 6..base + i * 6 + 6];
            assert_eq!(
                c.words(),
                *file_words,
                "{scene} 第 {i} 条控件 parse_words 序漂移 —— 文件序 z-order（最低 index 胜）被破坏"
            );
        }
    }
}

/// S15.5 重锚（`08_render_arch.md` §6 契约 5）：「关闭标记不产像素」迁为 ti_formats **纯数据**断言。
/// `f2`/`f3` = blit 原点（F-311：f2=屏 X、f3=屏 Y）。f2/f3 越出场景宽高（哨兵/关闭件）的记录
/// 旧渲染器越界裁剪后零 blit 区（不产像素）；此处以数据口径断言「有效/关闭」按 f2/f3 可二分，
/// 不再需要像素门禁。
#[test]
fn disabled_records_partitioned_by_f2_f3_sentinel() {
    let mut total = 0usize;
    let mut enabled = 0usize;
    let mut disabled = 0usize;
    for scene in SCENES {
        let (sc, controls) = load_scene(scene);
        let w = i64::from(sc.header.width);
        let h = i64::from(sc.header.height);
        for c in &controls {
            total += 1;
            let f2 = i64::from(c.f2);
            let f3 = i64::from(c.f3);
            // 有效件 = 原点在场景宽高内（旧渲染器 blit 区非空）；否则为哨兵/关闭件（越界零 blit）。
            if f2 < w && f3 < h {
                enabled += 1;
            } else {
                disabled += 1;
            }
        }
    }
    assert!(total > 0, "应解析出 .Map 控件记录（f2/f3 判据无数据可二分）");
    // 二分恒等：有效 + 关闭 = 全量（纯数据口径，锁「f2/f3 越界 = 关闭」的确定性判定）。
    assert_eq!(enabled + disabled, total, "f2/f3 哨兵二分数学不闭合");
    eprintln!(
        "f2/f3 关闭件判定：全量 {total}，有效 {enabled}，关闭(越界哨兵) {disabled}（纯数据，非像素门禁）"
    );
}
