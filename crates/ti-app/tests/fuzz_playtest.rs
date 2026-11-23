//! S13.1 种子化自动 playtest：32 个 seed × 2000 步确定性随机输入 fuzz。
//!
//! S74.1 测试体系重做（三处假判据 → 真实判据）：
//! - **panic**：`catch_unwind` + `downcast_ref::<&str>/<String>` 提取 payload，
//!   失败输出 seed / `last_step` / `last_scene` / panic 消息（`SeedResult.detail`）；
//!   `extract_panic_message` 单测用 `panic_any` 验证自定义负载提取。
//! - **状态不变式**：[`assert_state_invariants`]（契约 C3）逐步检查 3 条**真实可违反**的
//!   语义不变式（金币上限 / 每席赛道位置合法域 / 解锁链非空），替换旧 `scene_valid()` 恒真式。
//! - **存档往返**：走生产路径 `GameInput::WriteSave`（`SaveloadScene::on_input` 实建槽）
//!   → `save_slots()` 读回 → `restore_save` 实恢复 → 二次写回比对稳定字段；
//!   替换旧「手搓 slot 自读自写」假往返。夹具数值来自 `assets/data/balance.json`。
//!
//! 可复现：同 seed 二次运行输入序列逐帧一致（`deterministic_replay` 测试）。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::any::Any;
use std::cell::Cell;
use std::panic::{self, AssertUnwindSafe};

use ti_app::game_app::{GameApp, GameInput, HitData, Scene};
use ti_core::domain::economy::Building;
use ti_core::domain::persistence::SaveSlot;
use ti_core::domain::selection::{Character, MapSlot};
use ti_core::rules::progression::CLEARED_BIT;
use ti_core::rules::rng::GameRng;

/// 每局步数。
const STEPS: u32 = 2000;
/// 存档往返检查间隔（步）。
const SAVE_CHECK_INTERVAL: u32 = 500;
/// seed 总数。
const SEED_COUNT: u32 = 32;
/// 生产写档版本号（`scenes/saveload.rs::WRITE_VERSION` = v3，带 `boat_level`）。
const REAL_SAVE_VERSION: u32 = 3;
/// 往返检查使用的槽位（写档/读档均在 0 号槽）。
const SAVE_SLOT: usize = 0;

/// 单 seed 运行结果。
#[derive(Debug)]
struct SeedResult {
    /// 实际执行步数（panic 中止时 < STEPS）。
    steps: u32,
    /// 终止场景名。
    final_scene: &'static str,
    /// 异常数（panic + 不变式违背 + 存档损坏）。
    anomalies: u32,
    /// panic 计数。
    panics: u32,
    /// 不变式违背计数（替换旧「场景越界」假判据）。
    invariant_violations: u32,
    /// 存档损坏计数。
    save_corruptions: u32,
    /// 首个失败的定位信息（panic 消息 / 不变式违背 / 存档不一致），供 eprintln 输出。
    detail: Option<String>,
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

/// 场景判别值（与 `Scene::key` 不同——`key` 有 Intro/Exit 碰撞，用本函数报告 0..=8 唯一值）。
/// 仅用于 `deterministic_replay` 记录逐帧场景序列（该用途合法，非越界判定）。
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

/// 真实语义不变式（契约 C3）：只读 `GameApp` 公开字段/访问器，返回全部违背项。
///
/// 每条都不依赖「枚举穷举恒真」——它们绑定业务数值（`assets/data/balance.json`）
/// 与会话内部一致性，代码埋 bug 时**能真正失败**：
/// 1. 金币上限：`gold() <= balance.economy.gold_max`（0 = 不限）；数值单源 = balance.json。
/// 2. 赛道位置合法域：当前 `board` 与 5 个 `seat_sessions` 的 `position_index < track.len()`。
/// 3. 解锁链非空：`unlocked_maps` 至少含一张图（`ProgressionState::initial_unlocked` 口径）。
fn assert_state_invariants(app: &GameApp) -> Result<(), String> {
    let mut errs = Vec::new();

    // I1 金币上限（economy.gold_max；0 = 不限制）。
    if let Ok(b) = ti_core::data::balance() {
        let cap = b.economy.gold_max;
        if cap > 0 && app.gold() > cap {
            errs.push(format!("gold-cap: gold()={} cap={cap}", app.gold()));
        }
    }

    // I2 赛道位置合法域（board.json 格数；BoardSession::new/advance 须保持 index < len）。
    if let Some(board) = &app.board {
        let len = board.track().len();
        let pos = usize::try_from(board.position_index()).unwrap_or(usize::MAX);
        if len > 0 && pos >= len {
            errs.push(format!("board-pos: current pos={pos} >= track.len={len}"));
        }
    }
    for (seat, sess) in app.seat_sessions.iter().enumerate() {
        if let Some(board) = sess {
            let len = board.track().len();
            let pos = usize::try_from(board.position_index()).unwrap_or(usize::MAX);
            if len > 0 && pos >= len {
                errs.push(format!("seat-board-pos: seat[{seat}] pos={pos} >= track.len={len}"));
            }
        }
    }

    // I3 解锁链非空（开档仅 m01；清空 = 跨图推进状态被破坏）。
    if app.unlocked_maps.is_empty() {
        errs.push("unlocked-maps: empty (开档应至少含一张图)".to_string());
    }

    if errs.is_empty() { Ok(()) } else { Err(errs.join("; ")) }
}

/// 从 panic payload 提取消息（`&str` / `String` 两种标准负载；其他负载留标记）。
fn extract_panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_string()
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
        6 => GameInput::Spin,
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

/// 从 `rng` 取一个伪随机命中（S29.2 B1：`poll_hit` 消费 [`HitData`]；Bevy 拾取层产出的
/// 归一命中 = 目标键 `key` + 元素 `index` + 四向 `swipe` 码）。覆盖 `key 0..=7`（0 = 无元素 /
/// 空白，4..7 = 主地图 HUD）与 `swipe 0..=4`。`in_range` 返回 `u32`；`index`/`swipe` 的小值域
/// 截断/回绕属测试取模意图，pedantic cast 豁免。
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
fn random_hit(rng: &mut GameRng) -> HitData {
    HitData {
        key: rng.in_range(8),
        index: rng.in_range(11) as i32 - 1,
        swipe: rng.in_range(5) as u8,
    }
}

/// 真实存档往返：经生产写档路径建槽 → 读回 → 恢复 → 二次写回比对。
///
/// 返回 `Ok(())` 表示写/读/再写链路字段一致；`Err(msg)` 给出首个不一致字段（定位用）。
fn real_save_roundtrip(app: &GameApp, seed: u32, step: u32) -> Result<(), String> {
    // 记录源 app 的可观测字段（写档前快照）。
    let source_gold = app.gold();
    let source_character = app.character().map_or(1, |c| c.index() + 1);
    let source_map = app.map_slot().map_or(0, MapSlot::zero_index);
    let source_boat = app.boat_level();

    // ① 生产写档路径：临时切到 Saveload 场景，由 SaveloadScene::on_input 用全部真实
    //    访问器（rng_steps/unlocked_mask/save_v4_ext/boat_level...）构造槽并写 0 号位；
    //    WriteSave 不切场景（返回 false），写完即还原原场景，不扰动 fuzz 状态机。
    let mut writer = GameApp::new(1080.0, 1920.0);
    // 用「镜像 app」写档：把源 app 的公开字段灌入 writer 后走真实 on_input，避免改源 app.scene。
    // （writer 只承担构造槽的职责；字段与源 app 逐项对齐，见下方校验。）
    mirror_into_writer(&mut writer, app, seed);
    writer.scene = Scene::Saveload;
    writer.handle(GameInput::WriteSave(SAVE_SLOT));
    let written = writer.save_slots()[SAVE_SLOT].clone();

    // ② 写档保真：真实槽须反映源 app 的可观测状态（旧硬编码夹具 version=2/rng_steps=0 在此露馅）。
    if written.version != REAL_SAVE_VERSION
        || written.gold != source_gold
        || written.character != source_character
        || written.map_slot != source_map
        || written.boat_level != source_boat
    {
        return Err(format!(
            "seed={seed} step={step} write-fidelity written={{v:{},gold:{},char:{},map:{},boat:{}}} source={{v:{REAL_SAVE_VERSION},gold:{source_gold},char:{source_character},map:{source_map},boat:{source_boat}}}",
            written.version, written.gold, written.character, written.map_slot, written.boat_level
        ));
    }

    // ③ 生产读档路径：新壳 restore_save 真实恢复（不手搓 slot）。
    let mut restored = GameApp::new(1080.0, 1920.0);
    restored.set_rng_seed(seed);
    restored.restore_save(written.clone());
    if restored.boat_level() != written.boat_level.max(1)
        || restored.is_cleared() != (written.unlocked_mask & CLEARED_BIT != 0)
    {
        return Err(format!(
            "seed={seed} step={step} load-fidelity restored={{boat:{},cleared:{}}} written={{boat:{},mask:{}}}",
            restored.boat_level(),
            restored.is_cleared(),
            written.boat_level,
            written.unlocked_mask
        ));
    }

    // ④ 再写回：读档后的壳经同一生产路径写档，稳定字段须与原槽一致（完整往返闭环）。
    restored.scene = Scene::Saveload;
    restored.handle(GameInput::WriteSave(SAVE_SLOT));
    let rewritten = restored.save_slots()[SAVE_SLOT].clone();
    if rewritten.character != written.character
        || rewritten.map_slot != written.map_slot
        || rewritten.rng_steps != written.rng_steps
        || rewritten.unlocked_mask != written.unlocked_mask
        || rewritten.boat_level != written.boat_level
    {
        return Err(format!(
            "seed={seed} step={step} roundtrip written={{char:{},map:{},steps:{},mask:{},boat:{}}} rewritten={{char:{},map:{},steps:{},mask:{},boat:{}}}",
            written.character,
            written.map_slot,
            written.rng_steps,
            written.unlocked_mask,
            written.boat_level,
            rewritten.character,
            rewritten.map_slot,
            rewritten.rng_steps,
            rewritten.unlocked_mask,
            rewritten.boat_level
        ));
    }

    Ok(())
}

/// 把源 app 的可观测字段灌入新壳（仅为让生产写档路径看到与源一致的视图）。
fn mirror_into_writer(writer: &mut GameApp, source: &GameApp, seed: u32) {
    writer.set_rng_seed(seed);
    writer.character = source.character();
    writer.map_slot = source.map_slot();
    writer.map_pos = source.map_pos();
    writer.boat = source.boat.clone();
    writer.cleared = source.is_cleared();
    writer.unlocked_maps.clone_from(&source.unlocked_maps);
    writer.gold = source.gold();
    writer.seat_gold = source.seat_gold;
    writer.turn_cycle = source.turn_cycle;
    writer.selection = source.selection;
    writer.rng = source.rng.clone();
    writer.saveload_cursor = source.saveload_cursor;
}

/// 单 seed 执行 2000 步 fuzz（确定性 `GameRng` 输入流），返回结果统计。
///
/// panic 经 `catch_unwind` 捕获并提取 payload；不变式违背 / 存档损坏逐步累计，
/// 首个失败写入 `detail`（含 seed + step + 场景 + 原因）。每 500 步做一次真实存档往返。
fn run_playtest(seed: u32) -> SeedResult {
    let mut panics = 0u32;
    let mut invariant_violations = 0u32;
    let mut save_corruptions = 0u32;
    let mut steps_executed = 0u32;
    let mut final_scene = Scene::Index;
    let mut detail: Option<String> = None;

    // panic 后闭包内局部变量不可跨 unwind 取出 ⇒ 用 Cell 把最后一步/场景外泄。
    let last_step = Cell::new(0u32);
    let last_scene = Cell::new(Scene::Index);

    let closure = AssertUnwindSafe(|| {
        let mut rng = GameRng::new(seed);
        let mut app = GameApp::new(1080.0, 1920.0);
        app.set_rng_seed(seed);

        for step in 0..STEPS {
            final_scene = app.scene();
            last_step.set(step);
            last_scene.set(final_scene);

            // 真实语义不变式（逐步累计，替换旧 scene_valid 恒真式）。
            if let Err(msg) = assert_state_invariants(&app) {
                invariant_violations = invariant_violations.saturating_add(1);
                if detail.is_none() {
                    detail = Some(format!(
                        "seed={seed} step={step} scene={} invariant: {msg}",
                        scene_name(final_scene)
                    ));
                }
            }

            // 本步输入：60% 键盘随机事件 + 40% 触摸采样（确定性走 rng 流；
            // 键盘/触摸各场景行为不同，混合喂入覆盖两条输入通路）
            if rng.in_range(10) < 6 {
                let input = random_key_input(&mut rng);
                let _ = app.handle(input);
            } else {
                let hit = random_hit(&mut rng);
                let _ = app.poll_hit(Some(hit));
            }

            // 真实存档往返：每 500 步走生产写档→读档→再写回链路。
            if step > 0 && step % SAVE_CHECK_INTERVAL == 0 {
                if let Err(msg) = real_save_roundtrip(&app, seed, step) {
                    save_corruptions = save_corruptions.saturating_add(1);
                    if detail.is_none() {
                        detail = Some(msg);
                    }
                }
                // 恢复壳也必须满足不变式（restore_save 不得留下半初始化态）。
                let mut probe = GameApp::new(1080.0, 1920.0);
                probe.set_rng_seed(seed);
                let slot = app.save_slots()[SAVE_SLOT].clone();
                probe.restore_save(slot);
                if let Err(msg) = assert_state_invariants(&probe) {
                    invariant_violations = invariant_violations.saturating_add(1);
                    if detail.is_none() {
                        detail = Some(format!("seed={seed} step={step} restored-invariant: {msg}"));
                    }
                }
            }

            steps_executed = step + 1;
        }
    });

    match panic::catch_unwind(closure) {
        Ok(()) => {}
        Err(payload) => {
            panics = 1;
            let msg = extract_panic_message(payload.as_ref());
            detail = Some(format!(
                "seed={seed} panic last_step={} last_scene={}: {msg}",
                steps_executed,
                scene_name(last_scene.get())
            ));
            final_scene = Scene::Index;
        }
    }

    let anomalies = panics + invariant_violations + save_corruptions;
    SeedResult {
        steps: if panics > 0 { 0 } else { steps_executed },
        final_scene: scene_name(final_scene),
        anomalies,
        panics,
        invariant_violations,
        save_corruptions,
        detail,
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
        if let Some(d) = &r.detail {
            eprintln!("seed={seed} detail: {d}");
        }
        if r.panics > 0 {
            eprintln!("seed={seed} panic detected at/before fuzz loop");
        }
        if r.invariant_violations > 0 {
            eprintln!("seed={seed} invariant violations={}", r.invariant_violations);
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
            let hit = random_hit(&mut rng);
            let _ = app.poll_hit(Some(hit));
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

/// panic payload 提取：`panic_any` 注入 `String` 负载，须原样还原（定位信息不丢）。
/// tests/ 允许主动 panic 构造负例（库代码 0 panic 红线不适用）。
#[test]
#[allow(clippy::panic)]
fn panic_payload_extraction() {
    let payload = format!("injected-panic-{}", 42);
    let caught = panic::catch_unwind(AssertUnwindSafe(|| panic::panic_any(payload.clone())));
    match caught {
        Err(p) => assert_eq!(extract_panic_message(p.as_ref()), payload),
        Ok(()) => panic!("panic_any 应被 catch_unwind 捕获"),
    }
    let caught_str = panic::catch_unwind(AssertUnwindSafe(|| panic::panic_any("static-boom")));
    match caught_str {
        Err(p) => assert_eq!(extract_panic_message(p.as_ref()), "static-boom"),
        Ok(()) => panic!("panic_any 应被 catch_unwind 捕获"),
    }
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
        unlocked_mask: 0,
        boat_level: 1,

        v4: None,
    };
    app.restore_save(bad);

    assert_eq!(
        app.character, char_before,
        "越界 character 不得污染 character 字段（FIND-204 半初始化态回归）"
    );
    assert_eq!(app.scene(), Scene::SelectRole, "角色+地图均缺失 → scene = SelectRole");
    assert_eq!(app.gold(), 12345, "restore_save 写 gold 字段（设计行为，非 enter_main_map 路径）");

    // 合法 restore_save（Panda + M01）应成功进主地图
    let good = SaveSlot {
        version: 2,
        character: 1,
        gold: 100,
        map_slot: 0,
        position: 0,
        rng_steps: 0,
        unlocked_mask: 0,
        boat_level: 1,

        v4: None,
    };
    app.restore_save(good);
    assert_eq!(app.scene(), Scene::MainMap, "合法 restore_save 应进主地图");
    assert_eq!(app.character(), Some(Character::Panda));
}
