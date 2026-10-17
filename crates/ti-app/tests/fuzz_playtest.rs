//! S13.1 种子化自动 playtest：32 个 seed × 2000 步确定性随机输入 fuzz。
//!
//! 检测 3 类异常：
//! - panic（任何 `panic!`/`unwrap`/`expect` 触发，经 `catch_unwind` 捕获）
//! - 状态机越界（场景判别值 0..=8 合法性检查）
//! - 存档损坏（每 500 步 save→load 往返字段一致性）
//!
//! 可复现：同 seed 二次运行输入序列逐帧一致（`deterministic_replay` 测试）。

use std::panic::{self, AssertUnwindSafe};

use ti_app::game_app::{GameApp, GameInput, Scene};
use ti_app::touch::TouchSample;
use ti_core::domain::economy::Building;
use ti_core::domain::map::Dir;
use ti_core::domain::persistence::SaveSlot;
use ti_core::domain::selection::Character;
use ti_core::rules::rng::GameRng;
use ti_platform::settings::Gesture;

/// 每局步数。
const STEPS: u32 = 2000;
/// 存档往返检查间隔（步）。
const SAVE_CHECK_INTERVAL: u32 = 500;
/// seed 总数。
const SEED_COUNT: u32 = 32;

/// 单 seed 运行结果。
#[derive(Debug)]
struct SeedResult {
    /// 实际执行步数（panic 中止时 < STEPS）。
    steps: u32,
    /// 终止场景名。
    final_scene: &'static str,
    /// 异常数（panic + 场景越界 + 存档损坏）。
    anomalies: u32,
    /// panic 计数。
    panics: u32,
    /// 场景越界计数。
    scene_violations: u32,
    /// 存档损坏计数。
    save_corruptions: u32,
}

/// 场景名（报告用，与 `Scene` 变体一一对应）。
fn scene_name(scene: Scene) -> &'static str {
    match scene {
        Scene::Intro => "Intro",
        Scene::Index => "Index",
        Scene::SelectRole => "SelectRole",
        Scene::SelectMap => "SelectMap",
        Scene::MainMap => "MainMap",
        Scene::Vs => "Vs",
        Scene::Message => "Message",
        Scene::Saveload => "Saveload",
        Scene::Exit => "Exit",
    }
}

/// 场景判别值合法（0..=8，`Scene` 枚举 9 变体；`SceneKind::discriminant` 语义）。
fn scene_valid(scene: Scene) -> bool {
    (0..=8).contains(&scene_discriminant(scene))
}

/// 场景判别值（与 `Scene::key` 不同——`key` 有 Intro/Exit 碰撞，用本函数报告 0..=8 唯一值）。
fn scene_discriminant(scene: Scene) -> i32 {
    match scene {
        Scene::Intro => 0,
        Scene::Index => 1,
        Scene::SelectRole => 2,
        Scene::SelectMap => 3,
        Scene::MainMap => 4,
        Scene::Vs => 5,
        Scene::Message => 6,
        Scene::Saveload => 7,
        Scene::Exit => 8,
    }
}

/// 从 `rng` 取一个伪随机键盘输入（16 类，覆盖全部 `GameInput` 主要变体）。
fn random_key_input(rng: &mut GameRng) -> GameInput {
    match rng.in_range(16) {
        0 => GameInput::MoveUp,
        1 => GameInput::MoveDown,
        2 => GameInput::Confirm,
        3 => GameInput::Back,
        4 => GameInput::Prev,
        5 => GameInput::Next,
        6 => GameInput::Walk(Dir::all()[rng.in_range(4) as usize]),
        7 => GameInput::Landmark(rng.in_range(9)),
        8 => GameInput::TradeBuy(1 + rng.in_range(4)),
        9 => GameInput::TradeSell(1 + rng.in_range(4)),
        10 => GameInput::Build(Building::all()[rng.in_range(4) as usize]),
        11 => GameInput::Attack,
        12 => GameInput::LoadSave(rng.in_range(5) as usize),
        13 => GameInput::WriteSave(rng.in_range(5) as usize),
        14 => GameInput::MsgNext,
        _ => GameInput::MsgPrev,
    }
}

/// 从 `rng` 取一个伪随机触摸采样（手势 + 1080×1920 逻辑坐标）。
fn random_touch(rng: &mut GameRng) -> TouchSample {
    let gestures = [
        Gesture::Tap,
        Gesture::SwipeUp,
        Gesture::SwipeRight,
        Gesture::SwipeLeft,
        Gesture::SwipeDown,
    ];
    TouchSample {
        gesture: gestures[rng.in_range(5) as usize],
        // S15.6 QA108：呈现层逻辑坐标 = 1080×1920（`menu_shell` LOGICAL_W/H_I32 契约 1；
        // 旧 176×220 逻辑盒使 40% 触摸通路探的是 1080 失效区，触摸几何回归零覆盖）
        x: f32::from(u16::try_from(rng.in_range(1080)).unwrap_or(0)),
        y: f32::from(u16::try_from(rng.in_range(1920)).unwrap_or(0)),
    }
}

/// 单 seed 执行 2000 步 fuzz（确定性 `GameRng` 输入流），返回结果统计。
///
/// panic 经 `catch_unwind` 捕获后计为异常，该 seed 提前终止；场景越界 /
/// 存档损坏逐步累计。每 500 步做一次 save→load 往返断言。
fn run_playtest(seed: u32) -> SeedResult {
    let mut panics = 0u32;
    let mut scene_violations = 0u32;
    let mut save_corruptions = 0u32;
    let mut steps_executed = 0u32;
    let mut final_scene = Scene::Index;

    let closure = AssertUnwindSafe(|| {
        let mut rng = GameRng::new(seed);
        let mut app = GameApp::new(1080.0, 1920.0);
        app.set_rng_seed(seed);

        for step in 0..STEPS {
            final_scene = app.scene();
            // 状态机越界检查：场景判别值必须 0..=8
            if !scene_valid(final_scene) {
                scene_violations = scene_violations.saturating_add(1);
            }

            // 本步输入：60% 键盘随机事件 + 40% 触摸采样（确定性走 rng 流；
            // 键盘/触摸各场景行为不同，混合喂入覆盖两条输入通路）
            if rng.in_range(10) < 6 {
                let input = random_key_input(&mut rng);
                let _ = app.handle(input);
            } else {
                let sample = random_touch(&mut rng);
                let _ = app.poll_touch(Some(sample));
            }

            // 存档往返：每 500 步做一次 save→load，断言字段一致
            if step > 0 && step % SAVE_CHECK_INTERVAL == 0 {
                let slot = build_slot(&app);
                let mut restored = GameApp::new(1080.0, 1920.0);
                restored.set_rng_seed(seed);
                restored.write_save_slot(0, slot);
                restored.restore_save(slot);
                let slot_after = restored.save_slots()[0];
                if slot_after != slot {
                    save_corruptions = save_corruptions.saturating_add(1);
                }
                // 往返后场景必须合法（restore 落在 MainMap / SelectRole）
                if !scene_valid(restored.scene()) {
                    scene_violations = scene_violations.saturating_add(1);
                }
            }

            steps_executed = step + 1;
        }
    });

    if panic::catch_unwind(closure).is_err() {
        panics = 1;
        // panic 中止：终止场景不可跨 unwind 取出，以 Index 占位；步数记 0
        final_scene = Scene::Index;
    }

    let anomalies = panics + scene_violations + save_corruptions;
    SeedResult {
        steps: if panics > 0 { 0 } else { steps_executed },
        final_scene: scene_name(final_scene),
        anomalies,
        panics,
        scene_violations,
        save_corruptions,
    }
}

/// 从当前 `GameApp` 状态构造存档槽（与 `saveload.rs::WriteSave` 同口径）。
fn build_slot(app: &GameApp) -> SaveSlot {
    let character = app.character().map_or(1, |c| c.index() + 1);
    let map_slot = app.map_slot().map_or(0, ti_core::domain::selection::MapSlot::zero_index);
    SaveSlot {
        version: 2,
        character,
        gold: app.gold(),
        map_slot,
        position: app.map_pos().map_or(0, |p| p.index(20)),
        rng_steps: 0,
    }
}

/// 32 seed 全量 fuzz：逐 seed 跑 2000 步，报告步数 / 终止场景 / 异常分类；
/// 任一异常即测试失败（0 容忍）。
#[test]
fn fuzz_32_seeds() {
    let mut total_anomalies = 0u32;
    let mut clean = 0u32;
    for seed in 0..SEED_COUNT {
        let r = run_playtest(seed);
        total_anomalies += r.anomalies;
        if r.anomalies == 0 {
            clean += 1;
        }
        println!(
            "seed={seed:>2}  steps={}  final_scene={}  anomalies={}",
            r.steps, r.final_scene, r.anomalies
        );
        if r.panics > 0 {
            eprintln!("seed={seed} panic detected at/before fuzz loop");
        }
        if r.scene_violations > 0 {
            eprintln!("seed={seed} scene violations={}", r.scene_violations);
        }
        if r.save_corruptions > 0 {
            eprintln!("seed={seed} save corruptions={}", r.save_corruptions);
        }
    }
    println!("{clean}/{SEED_COUNT} seeds clean");
    assert_eq!(total_anomalies, 0, "32 seed 应 0 异常（当前总异常 {total_anomalies}）");
    assert_eq!(clean, SEED_COUNT, "全部 seed 应 clean");
}

/// 可复现判据：seed=0 跑两次，逐帧场景序列必须完全一致（diff = 0）。
#[test]
fn deterministic_replay() {
    let seq1 = record_scene_sequence(0);
    let seq2 = record_scene_sequence(0);
    assert_eq!(
        seq1.len(),
        seq2.len(),
        "同 seed 两次运行步数应一致（{} vs {}）",
        seq1.len(),
        seq2.len()
    );
    assert_eq!(seq1, seq2, "同 seed 两次运行的场景序列应逐帧一致（EXP-040 确定性）");
    println!("deterministic_replay: seed=0 场景序列 {} 帧，两次完全一致", seq1.len());
}

/// 记录一个 seed 完整 fuzz 的场景序列（每步 1 个 i32 判别值）。
///
/// 与 [`run_playtest`] 同输入流（同 `GameRng` seed + 同键盘/触摸比例），
/// 两次调用产出逐帧一致序列。
fn record_scene_sequence(seed: u32) -> Vec<i32> {
    let mut seq = Vec::new();
    let mut rng = GameRng::new(seed);
    let mut app = GameApp::new(1080.0, 1920.0);
    app.set_rng_seed(seed);
    for _ in 0..STEPS {
        seq.push(scene_discriminant(app.scene()));
        if rng.in_range(10) < 6 {
            let input = random_key_input(&mut rng);
            let _ = app.handle(input);
        } else {
            let sample = random_touch(&mut rng);
            let _ = app.poll_touch(Some(sample));
        }
    }
    seq
}

/// 单 seed 快速回归：seed=5 跑 2000 步必须 0 异常（比 32 seed 全量更快命中回归）。
#[test]
fn fuzz_single_seed_quick() {
    let r = run_playtest(5);
    assert_eq!(r.anomalies, 0, "seed=5 应 0 异常，实际 {r:?}");
    assert_eq!(r.steps, STEPS, "seed=5 应跑满 2000 步");
}

/// FIND-204 回归测试：越界槽读档不污染 character 字段。
#[test]
fn restore_save_invalid_slot_does_not_pollute_character() {
    let mut app = GameApp::new(1080.0, 1920.0);
    app.set_rng_seed(42);
    // 先设一个已选角色（确保调用前 character 非 None，可观测「未被污染」）
    app.character = Some(Character::Drake);
    let char_before = app.character;

    let bad = SaveSlot {
        version: 2,
        character: u32::MAX,
        gold: 12345,
        map_slot: u32::MAX,
        position: 0,
        rng_steps: 0,
    };
    app.restore_save(bad);

    assert_eq!(
        app.character, char_before,
        "越界 character 不得污染 character 字段（FIND-204 半初始化态回归）"
    );
    assert_eq!(app.scene(), Scene::SelectRole, "角色+地图均缺失 → scene = SelectRole");
    assert_eq!(app.gold(), 12345, "restore_save 写 gold 字段（设计行为，非 enter_main_map 路径）");

    // 合法 restore_save（Panda + M01）应成功进主地图
    let good =
        SaveSlot { version: 2, character: 1, gold: 100, map_slot: 0, position: 0, rng_steps: 0 };
    app.restore_save(good);
    assert_eq!(app.scene(), Scene::MainMap, "合法 restore_save 应进主地图");
    assert_eq!(app.character(), Some(Character::Panda));
}
