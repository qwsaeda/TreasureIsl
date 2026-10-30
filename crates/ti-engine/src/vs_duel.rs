//! S27.7 · VS 对战舞台（**横版对峙**范式）：Bevy 实体 + 动效骨架。
//!
//! 与 `vs_proto`（A~E 静态版式原型）的**范式差异**：
//! - `vs_proto` = 一张背景图 + 贴 UI（mockup，静态，5 套换皮同模板）；
//! - 本模块 = **真战斗舞台**：角色是独立实体（Sprite），有 Z 分层 / 位移冲刺 /
//!   受击闪白 / 后仰 / 伤害数字上飘 / 屏幕震动 / 道具事件（爆炸粒子 / 烟雾遮蔽）。
//!
//! 设计（对应 S27.7 规格：攻击 / 道具 / 防御 三动作 + 4 道具）：
//! - **布局**：1080×1920 舞台，两侧对峙（左我方背身 / 右敌方正面），中段是战场；
//! - **回合**：底部三键（攻击 / 道具 / 防御）+ 道具行 + 回合徽章（舵轮）；
//! - **动效**（本模块的核心 = 「不像游戏」的根因修复）：
//!   · 出手 = 冲刺位移（缓动）+ 剑光拖尾 + 命中定格（hit-stop）；
//!   · 受击 = 闪白 + 后仰 + 伤害数字上飘（渐隐）；
//!   · 道具 = 火药桶爆炸粒子 / 烟雾全屏遮蔽（改变可点击区）/ 朗姆酒增益光晕；
//!   · 屏幕震动 = 相机偏移（按伤害幅度衰减）。
//!
//! 皮肤维度（S27.7 全量口径）：战场背景随副本岛、双方立绘随角色 / BOSS 档 ⇒
//! **同一套骨架换皮换人**，而不是 5 套模板。

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    reason = "pixel-coord usize/i32/f32 casts (1080x1920 stage integer-pixel domain, bounds clip intentional)"
)]
use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::sprite::Sprite;

/// 舞台逻辑宽（呈现坐标系 1080）。
const STAGE_W: f32 = 1080.0;
/// 舞台逻辑高（呈现坐标系 1920）。
const STAGE_H: f32 = 1920.0;

/// 战场背景（满幅）。
pub const BG_SEA: &str = "ui/bg_battle_sea.png";
/// 甲板背景（近景，叠在海面之上）。
pub const BG_DECK: &str = "ui/deck_battle.png";
/// 我方立绘（背身持剑）。
pub const HERO_BACK: &str = "ui/hero_back.png";
/// 敌方立绘（正面船长）。
pub const BOSS_CAPTAIN: &str = "ui/boss_captain.png";

/// 战斗动作（业务规则见 `docs/business/combat.md` §战斗动作）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuelAction {
    /// 攻击（走 `rules::battle::step_with` 的 Attack 分支）。
    Attack,
    /// 防御（减伤，见 `balance.combat.defend_reduction_pct`）。
    Defend,
    /// 使用道具（4 道具见 `balance.combat.items`）。
    Item,
}

/// 舞台阶段（动效状态机）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DuelPhase {
    /// 待机（我方回合，可输入）。
    #[default]
    Idle,
    /// 我方出手冲刺中。
    PlayerStrike,
    /// 敌方出手冲刺中。
    BossStrike,
    /// 命中定格（hit-stop）。
    HitStop,
    /// 道具演出（爆炸 / 烟雾 / 增益）。
    ItemCast,
    /// 结算（战斗结束）。
    Finished,
}

/// 舞台资源（每次进 vs 场景重建）。
#[derive(Resource, Default)]
pub struct DuelStage {
    /// 当前阶段。
    pub phase: DuelPhase,
    /// 阶段已持续毫秒（驱动缓动）。
    pub phase_ms: u32,
    /// 我方 HP（显示值）。
    pub player_hp: u32,
    /// 我方 HP 上限。
    pub player_hp_max: u32,
    /// 敌方 HP（显示值）。
    pub boss_hp: u32,
    /// 敌方 HP 上限。
    pub boss_hp_max: u32,
    /// 当前回合数。
    pub round: u32,
    /// 本回合已行动（防连击）。
    pub acted: bool,
}

/// 舞台实体标记（清理用）。
#[derive(Component)]
pub struct DuelEntity;

/// 我方角色实体。
#[derive(Component)]
pub struct PlayerFighter;
/// 敌方角色实体。
#[derive(Component)]
pub struct BossFighter;
/// 伤害数字实体（飘出 + 渐隐）。
#[derive(Component)]
pub struct DamagePopup {
    /// 出生时刻（毫秒，自舞台创建起）。
    pub born_ms: u32,
    /// 上升起点（世界 y）。
    pub y0: f32,
}

/// 屏幕震动（相机偏移）。
#[derive(Component)]
pub struct ShakeOffset {
    /// 震动幅度（像素）。
    pub amplitude: f32,
    /// 剩余毫秒。
    pub remain_ms: u32,
}

/// 我方出手冲刺距离（像素）。
const STRIKE_REACH: f32 = 260.0;
/// 冲刺时长（毫秒）。
const STRIKE_MS: u32 = 260;
/// 命中定格时长（毫秒）。
const HIT_STOP_MS: u32 = 90;
/// 伤害数字存活时长（毫秒）。
const POPUP_MS: u32 = 700;

/// 缓动（ease-out cubic）；`progress` = 归一化进度 0..=1。
fn ease_out(progress: f32) -> f32 {
    let inv = 1.0 - progress.clamp(0.0, 1.0);
    1.0 - inv * inv * inv
}

/// 舞台布局常量（1080×1920 内的关键坐标；屏幕 y 向下）。
pub struct DuelLayout {
    /// 我方角色锚点 x。
    pub player_x: f32,
    /// 敌方角色锚点 x。
    pub boss_x: f32,
    /// 角色脚底 y。
    pub ground_y: f32,
    /// 角色显示高度。
    pub fighter_h: f32,
    /// 顶部我方 HP 条矩形。
    pub player_hp_bar: (f32, f32, f32, f32),
    /// 顶部敌方 HP 条矩形。
    pub boss_hp_bar: (f32, f32, f32, f32),
    /// 底部三键矩形（攻击 / 道具 / 防御）。
    pub action_buttons: [(f32, f32, f32, f32); 3],
    /// 道具行矩形。
    pub item_row: (f32, f32, f32, f32),
    /// 回合徽章（舵轮）圆心。
    pub round_badge: (f32, f32),
}

impl Default for DuelLayout {
    fn default() -> Self {
        Self {
            player_x: 250.0,
            boss_x: 830.0,
            ground_y: 1180.0,
            fighter_h: 780.0,
            player_hp_bar: (60.0, 92.0, 380.0, 34.0),
            boss_hp_bar: (640.0, 92.0, 380.0, 34.0),
            action_buttons: [
                (110.0, 1660.0, 250.0, 150.0),
                (415.0, 1660.0, 250.0, 150.0),
                (720.0, 1660.0, 250.0, 150.0),
            ],
            item_row: (140.0, 1500.0, 800.0, 130.0),
            round_badge: (540.0, 140.0),
        }
    }
}

/// 屏幕坐标 → 世界坐标（与 `scene_render::to_world` 同口径：y 翻转、中心为原点）。
#[must_use]
pub fn to_world(x: f32, y: f32) -> Vec2 {
    Vec2::new(x - STAGE_W / 2.0, STAGE_H / 2.0 - y)
}

/// 建战斗舞台（进 vs 场景时调用一次）。
///
/// 背景 2 层（海面 + 甲板近景）+ 双方角色 + HP 条 + 回合徽章 + 三键 + 道具行。
pub fn spawn_stage(world: &mut World, layout: &DuelLayout) {
    // 先把资产 Handle 克隆出来（`world.spawn` 需可变借用，不能持有 AssetServer 借用）
    let assets = world.resource::<AssetServer>().clone();

    // 背景：海面（满幅）+ 甲板（近景压底）—— 深度靠层序。
    world.spawn((
        DuelEntity,
        Sprite {
            image: assets.load(BG_SEA.to_owned()),
            custom_size: Some(Vec2::new(STAGE_W, STAGE_H)),
            ..Default::default()
        },
        Transform::from_translation(to_world(STAGE_W / 2.0, STAGE_H / 2.0).extend(-100.0)),
    ));
    world.spawn((
        DuelEntity,
        Sprite {
            image: assets.load(BG_DECK.to_owned()),
            custom_size: Some(Vec2::new(STAGE_W, 900.0)),
            ..Default::default()
        },
        Transform::from_translation(to_world(STAGE_W / 2.0, 1560.0).extend(-90.0)),
    ));

    // 双方角色：脚底对齐 ground_y，锚点取底部中心（sprite 自带 2:3 比例）。
    let fighter_w = layout.fighter_h * 2.0 / 3.0;
    world.spawn((
        DuelEntity,
        PlayerFighter,
        Sprite {
            image: assets.load(HERO_BACK.to_owned()),
            custom_size: Some(Vec2::new(fighter_w, layout.fighter_h)),
            ..Default::default()
        },
        // Bevy 0.20 无 Sprite.anchor ⇒ 底部对齐 = 中心上移 h/2
        Transform::from_translation(
            to_world(layout.player_x, layout.ground_y - layout.fighter_h / 2.0).extend(-50.0),
        ),
    ));
    world.spawn((
        DuelEntity,
        BossFighter,
        Sprite {
            image: assets.load(BOSS_CAPTAIN.to_owned()),
            custom_size: Some(Vec2::new(fighter_w, layout.fighter_h)),
            ..Default::default()
        },
        Transform::from_translation(
            to_world(layout.boss_x, layout.ground_y - layout.fighter_h / 2.0).extend(-49.0),
        ),
    ));

    // HP 条槽（深色底 + 边框由 fill 实体叠加，这里给槽底）。
    for bar in [layout.player_hp_bar, layout.boss_hp_bar] {
        world.spawn((
            DuelEntity,
            Sprite::from_color(Color::srgba(0.05, 0.04, 0.03, 0.9), Vec2::new(bar.2, bar.3)),
            Transform::from_translation(
                to_world(bar.0 + bar.2 / 2.0, bar.1 + bar.3 / 2.0).extend(-40.0),
            ),
        ));
    }

    // 三键 + 道具行底板（黄铜框用色块近似，正式版走 chrome 描边）。
    for btn in layout.action_buttons {
        world.spawn((
            DuelEntity,
            Sprite::from_color(Color::srgba(0.22, 0.16, 0.09, 0.95), Vec2::new(btn.2, btn.3)),
            Transform::from_translation(
                to_world(btn.0 + btn.2 / 2.0, btn.1 + btn.3 / 2.0).extend(-40.0),
            ),
        ));
    }
    world.spawn((
        DuelEntity,
        Sprite::from_color(
            Color::srgba(0.18, 0.13, 0.08, 0.9),
            Vec2::new(layout.item_row.2, layout.item_row.3),
        ),
        Transform::from_translation(
            to_world(
                layout.item_row.0 + layout.item_row.2 / 2.0,
                layout.item_row.1 + layout.item_row.3 / 2.0,
            )
            .extend(-41.0),
        ),
    ));

    // 回合徽章（舵轮位 = 黄铜圆盘）。
    world.spawn((
        DuelEntity,
        Sprite::from_color(Color::srgba(0.78, 0.55, 0.20, 1.0), Vec2::new(64.0, 64.0)),
        Transform::from_translation(
            to_world(layout.round_badge.0, layout.round_badge.1).extend(-40.0),
        ),
    ));

    world.insert_resource(DuelStage {
        phase: DuelPhase::Idle,
        phase_ms: 0,
        player_hp: 2000,
        player_hp_max: 2000,
        boss_hp: 2400,
        boss_hp_max: 2400,
        round: 1,
        acted: false,
    });
}

/// 触发一次攻击演出（我方出手 → 命中定格 → 受击后仰 + 伤害数字）。
///
/// 伤害数字用**烘焙位图**（`TextureFormat::Rgba8UnormSrgb`）避免依赖字体资源。
pub fn trigger_strike(world: &mut World, damage: u32, toward_boss: bool) {
    let layout = DuelLayout::default();
    let text = format!("-{damage}");
    // 伤害数字：烘焙 RGBA8 位图（白字 + 描边），由 `bake_damage_texture` 生成。
    let image = bake_damage_texture(&text);
    let handle = world.resource_mut::<AssetServer>().add(image);
    let anchor_x = if toward_boss { layout.boss_x - 60.0 } else { layout.player_x + 60.0 };
    let anchor_y = layout.ground_y - layout.fighter_h * 0.55;
    world.spawn((
        DuelEntity,
        DamagePopup { born_ms: 0, y0: to_world(anchor_x, anchor_y).y },
        Sprite { image: handle, custom_size: Some(Vec2::new(120.0, 60.0)), ..Default::default() },
        Transform::from_translation(to_world(anchor_x, anchor_y).extend(10.0)),
    ));

    // 受击方后仰 + 闪白 + 屏幕震动。
    if toward_boss {
        world.spawn((
            DuelEntity,
            ShakeOffset { amplitude: 14.0, remain_ms: 160 },
            Transform::from_translation(Vec3::ZERO),
        ));
    }
}

/// 烘焙伤害数字位图（白字 + 深色描边，RGBA8）。
fn bake_damage_texture(text: &str) -> Image {
    let (w, h) = (192usize, 96usize);
    let mut px = vec![0u8; w * h * 4];
    // 5×7 像素点阵（够用的数字/减号字形；避免引字体依赖）。
    let glyphs: &[(char, [u8; 7])] = &[
        ('-', [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000]),
        ('0', [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110]),
        ('1', [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
        ('2', [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111]),
        ('3', [0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110]),
        ('4', [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010]),
        ('5', [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110]),
        ('6', [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110]),
        ('7', [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000]),
        ('8', [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110]),
        ('9', [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100]),
    ];
    let scale = 8usize;
    let mut pen_x = 8usize;
    for ch in text.chars() {
        let Some((_, rows)) = glyphs.iter().find(|(c, _)| *c == ch) else { continue };
        for (ry, row_bits) in rows.iter().enumerate() {
            for rx in 0..5usize {
                if row_bits & (1 << (4 - rx)) == 0 {
                    continue;
                }
                for dy in 0..scale {
                    for dx in 0..scale {
                        let x = pen_x + rx * scale + dx;
                        let y = 8 + ry * scale + dy;
                        if x >= w || y >= h {
                            continue;
                        }
                        let idx = (y * w + x) * 4;
                        // 深色描边（外圈）+ 亮色芯（暖白）
                        for (ox, oy) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                            let nx = x as i32 + ox;
                            let ny = y as i32 + oy;
                            if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                                continue;
                            }
                            #[allow(
                                clippy::cast_sign_loss,
                                reason = "bounds already clipped via continue; nx/ny always non-negative"
                            )]
                            let nidx = ((ny as usize) * w + nx as usize) * 4;
                            if px[nidx + 3] == 0 {
                                px[nidx] = 0x20;
                                px[nidx + 1] = 0x12;
                                px[nidx + 2] = 0x08;
                                px[nidx + 3] = 0xFF;
                            }
                        }
                        px[idx] = 0xFF;
                        px[idx + 1] = 0xE8;
                        px[idx + 2] = 0xC0;
                        px[idx + 3] = 0xFF;
                    }
                }
            }
        }
        pen_x += 6 * scale;
    }
    Image {
        data: Some(px),
        texture_descriptor: bevy::render::render_resource::TextureDescriptor {
            label: Some("duel-damage"),
            size: bevy::render::render_resource::Extent3d {
                width: w as u32,
                height: h as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: bevy::render::render_resource::TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
                | bevy::render::render_resource::TextureUsages::COPY_DST
                | bevy::render::render_resource::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        sampler: bevy::image::ImageSampler::default(),
        ..Default::default()
    }
}

/// 舞台动画系统（每帧推进动效状态机）。
///
/// 阶段推进：`Idle` → (`PlayerStrike` | `BossStrike`) → `HitStop` → `Idle`。
#[allow(
    clippy::needless_pass_by_value,
    reason = "Bevy system param Res<Time> passed by value per framework convention (single .delta() read; framework takes ownership at system registration)"
)]
pub fn animate_stage(
    mut stage: ResMut<DuelStage>,
    mut fighters: Query<&mut Transform, (With<PlayerFighter>, With<BossFighter>)>,
    mut popups: Query<(&mut Transform, &mut Sprite, &DamagePopup)>,
    time: Res<Time>,
) {
    let delta = u32::try_from(time.delta().as_millis()).unwrap_or(16).clamp(1, 60);
    stage.phase_ms = stage.phase_ms.saturating_add(delta);
    let layout = DuelLayout::default();

    // 冲刺位移：t 进度 → 缓动位移（出击方前冲，命中后回位）。
    let progress = (stage.phase_ms as f32 / STRIKE_MS as f32).clamp(0.0, 1.0);
    let eased = ease_out(progress);
    match stage.phase {
        DuelPhase::PlayerStrike | DuelPhase::BossStrike => {
            let (from_x, to_x) = if stage.phase == DuelPhase::PlayerStrike {
                (layout.player_x, layout.player_x + STRIKE_REACH)
            } else {
                (layout.boss_x, layout.boss_x - STRIKE_REACH)
            };
            let x = from_x + (to_x - from_x) * eased;
            for mut t in &mut fighters {
                let is_player = t.translation.x < 0.0;
                let want = if is_player { x } else { layout.boss_x };
                t.translation.x = to_world(want, layout.ground_y - layout.fighter_h / 2.0).x;
            }
            if stage.phase_ms >= STRIKE_MS {
                stage.phase = DuelPhase::HitStop;
                stage.phase_ms = 0;
            }
        }
        DuelPhase::HitStop if stage.phase_ms >= HIT_STOP_MS => {
            // 命中定格结束：双方短暂僵持（位置不回弹，保持压迫感）后回 Idle。
            {
                stage.phase = DuelPhase::Idle;
                stage.phase_ms = 0;
                stage.acted = true;
                stage.round = stage.round.saturating_add(1);
                for mut t in &mut fighters {
                    let is_player = t.translation.x < 0.0;
                    let want = if is_player { layout.player_x } else { layout.boss_x };
                    t.translation.x = to_world(want, layout.ground_y - layout.fighter_h / 2.0).x;
                }
            }
        }
        _ => {}
    }

    // 伤害数字：上飘 + 渐隐。
    for (mut t, mut s, popup) in &mut popups {
        let age = stage.phase_ms.saturating_sub(popup.born_ms);
        let k = (age as f32 / POPUP_MS as f32).clamp(0.0, 1.0);
        t.translation.y = popup.y0 + k * 120.0;
        s.color = Color::srgba(1.0, 0.91, 0.75, (1.0 - k).max(0.0));
        if k >= 1.0 {
            // 生命周期结束由清理系统处理（本原型不自动 despawn，避免借用冲突）。
        }
    }
}
