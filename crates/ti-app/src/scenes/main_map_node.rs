//! S21.7 赛道节点信息签：kind → 标签键 / 确定性落格产出文案 + 签条式面板绘制
//! （预览选中态；只读不移动 —— 规格 `08_main_map_v4_spec.md` §4「点格语义」）。
//!
//! 文案口径：标签 = `scene.main_map.node.<kind>`（`strings.json` 扁平键）；
//! 第二行 = **确定性**落格产出（城市/宝藏/遭遇读 `balance.board` 定值；事件/卡牌/BOSS
//! = 随机或未定 ⇒ 不臆测数值，只显标签）。
use ti_core::data::{Balance, balance};
use ti_core::rules::board::SpaceKind;

use crate::i18n;
use crate::scenes::SceneHost;
use crate::scenes::chrome;

/// S21.7 节点信息签文案键（`scene.main_map.node.<kind>`，`strings.json` 扁平键）。
#[must_use]
pub fn node_label_key(kind: SpaceKind) -> &'static str {
    use SpaceKind as K;
    match kind {
        K::Start => "scene.main_map.node.start",
        K::Plain => "scene.main_map.node.plain",
        K::Port => "scene.main_map.node.port",
        K::City => "scene.main_map.node.city",
        K::Event => "scene.main_map.node.event",
        K::Card => "scene.main_map.node.card",
        K::Battle => "scene.main_map.node.battle",
        K::Territory => "scene.main_map.node.territory",
        K::Treasure => "scene.main_map.node.treasure",
        K::Boss => "scene.main_map.node.boss",
    }
}

/// S21.7 信息签第一行 = 种类标签（i18n 解析；缺键 = 回退键名，e2e 用此断言非回退）。
#[must_use]
pub fn track_node_label(kind: SpaceKind) -> String {
    i18n::sget(node_label_key(kind)).to_string()
}

/// S21.7 信息签第二行 = **确定性**落格产出（城市/宝藏/遭遇读 `balance.board` 定值；
/// 事件/卡牌/BOSS = 随机或未定 ⇒ 不臆测数值，只显标签；只读预览口径，spec §4）。
#[must_use]
pub fn track_node_effect(kind: SpaceKind, b: &Balance) -> Option<String> {
    match kind {
        SpaceKind::City => {
            Some(i18n::sgetf("scene.main_map.toast.city_fmt", &[&b.board.city_gold]))
        }
        SpaceKind::Treasure => {
            Some(i18n::sgetf("scene.main_map.toast.treasure_fmt", &[&b.board.treasure_gold]))
        }
        SpaceKind::Battle => {
            Some(i18n::sgetf("scene.main_map.toast.battle_fmt", &[&b.board.battle_damage]))
        }
        _ => None,
    }
}

/// S21.7 节点信息签（`brass_dark` 签条式面板 + 黄铜边，锚定节点上方；越上沿则翻到下方。
/// 宽度按字自适应，最小 240；缺 balance = 不画（crash-safe））。
pub(crate) fn draw_node_card(
    host: &mut dyn SceneHost,
    cx: i32,
    node_top: i32,
    node_bottom: i32,
    kind: SpaceKind,
    theme: &ti_render::theme::Theme,
) {
    let b = match balance() {
        Ok(b) => b,
        Err(_e) => {
            #[cfg(feature = "stats")]
            eprintln!(
                "[stats] draw_node_card: balance.json load failed (card skipped, crash-safe)"
            );
            return;
        }
    };
    let label = track_node_label(kind);
    let effect = track_node_effect(kind, b);
    let mut w = host.shell().hint_text_width_px(&label);
    if let Some(e) = &effect {
        w = w.max(host.shell().hint_text_width_px(e));
    }
    let pad = 24;
    let panel_w = (w + pad * 2).max(240);
    let panel_h = i32::from(effect.is_some()) * 24 + 48;
    // 面板中心 = 节点中心（spec §4「锚定在被点节点上方」），越屏边则钳回屏内
    let px = (cx - panel_w / 2).clamp(8, 1080 - 8 - panel_w);
    let py = if node_top - panel_h - 8 < 248 { node_bottom + 8 } else { node_top - panel_h - 8 };
    let panel = theme.by_name("brass_dark").unwrap_or(chrome::COLOR_GOLD_WARM);
    let brass = theme.by_name("brass").unwrap_or(chrome::COLOR_GOLD_WARM);
    host.fill_rect(px, py, panel_w, panel_h, panel);
    host.fill_rect(px, py, panel_w, 2, brass);
    host.fill_rect(px, py + panel_h - 2, panel_w, 2, brass);
    host.fill_rect(px, py, 2, panel_h, brass);
    host.fill_rect(px + panel_w - 2, py, 2, panel_h, brass);
    let tx = px + (panel_w - w) / 2;
    host.draw_hint_text(&label, tx, py + 14, chrome::COLOR_TEXT_ON_PARCH);
    if let Some(e) = &effect {
        host.draw_hint_text(e, tx, py + 44, chrome::COLOR_GOLD_WARM);
    }
}
