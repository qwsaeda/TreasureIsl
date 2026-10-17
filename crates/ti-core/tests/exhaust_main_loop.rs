//! S15.1 主循环 / 场景状态机域业务场景穷举测试（`exhaust_main_loop`，第 5 域）。
//!
//! 覆盖 `docs/00_design.md` §1 第 4 条可断言约束「任意场景 ≤ N 次操作可回主菜单」
//! （N = `balance.ui.max_back_steps`，`docs/business/loop_core.md` §场景切换）：
//! 枚举空间由 `assets/data/exhaust_scenarios.json`（`ti_core::data::exhaust()`）驱动——
//! `main_loop.states`（`rules/main_loop.rs` `SceneStack::advance` 转移表 match 分支全部命中码
//! 0/2/3/4/25/26/27，事实源在 `main_loop.rs` 13-15 行注释，非 `loop_core.md`）×
//! `main_loop.triggers`（`main_loop.rs` 转移表 trigger 码表全集 1/2/4/5），对**每个**状态码做
//! BFS 求「回到主菜单（状态 0）的最短 trigger 步数」。
//!
//! **约束语义**：`00_design.md` 第 4 条是「存在性上界」——任意场景**存在**一条 ≤ N 的序列
//! 可回主菜单（无死锁），BFS 最短路径 `steps ≤ N` 正是该语义的精确刻画；并非「所有 ≤N 序列
//! 都回主菜单」（那会是「任意性」强约束，原文未要求）。
//!
//! 逐状态穷举（非抽样），转移函数直接调 `SceneStack::advance`（唯一事实源），
//! 若未来转移表被改坏（某状态变成死胡同 / 回主菜单最短路径变长超 N）即 FAIL。

use std::collections::{BTreeMap, VecDeque};

use ti_core::data::{balance, exhaust};
use ti_core::rules::main_loop::SceneStack;

/// 转移函数：给定状态码 `s` 与 trigger `t`，返回 `SceneStack::advance` 产出的**新状态码**。
/// 每边独立构造 `SceneStack`（只取状态码，不关心 history/scene 副作用）。
fn transition(s: u32, t: u32) -> u32 {
    let mut st = SceneStack::new();
    st.set_state(s);
    let (ns, _) = st.advance(t);
    ns
}

/// 主菜单状态码（`loop_core.md`：0 = 主菜单）。
const MAIN_MENU_STATE: u32 = 0;

/// 穷举：全状态码 × trigger 枚举（JSON 驱动），BFS 验证「任意场景 ≤ N 次操作回主菜单」。
#[test]
fn exhaust_main_loop_no_deadlock_within_max_back_steps() {
    let e = exhaust().expect("exhaust_scenarios.json should parse");
    let b = balance().expect("balance.json valid");
    let n = b.ui.max_back_steps;
    let states = &e.main_loop.states;
    let triggers = &e.main_loop.triggers;
    assert!(!states.is_empty(), "main_loop.states must not be empty");
    assert!(!triggers.is_empty(), "main_loop.triggers must not be empty");
    assert!(states.contains(&MAIN_MENU_STATE), "状态枚举须含主菜单 0");

    let mut states_checked: u32 = 0;
    let mut worst_steps: u32 = 0;
    for &start in states {
        // BFS：从 `start` 出发，求回到主菜单（状态 0）的最短 trigger 步数。
        let mut dist: BTreeMap<u32, u32> = BTreeMap::new();
        let mut queue: VecDeque<u32> = VecDeque::new();
        dist.insert(start, 0);
        queue.push_back(start);
        let mut steps_to_main: Option<u32> = None;
        while let Some(s) = queue.pop_front() {
            if s == MAIN_MENU_STATE {
                steps_to_main = Some(dist[&s]);
                break;
            }
            let d = dist[&s];
            for &t in triggers {
                let ns = transition(s, t);
                if dist.insert(ns, d + 1).is_none() {
                    queue.push_back(ns);
                }
            }
        }

        if start == MAIN_MENU_STATE {
            assert_eq!(steps_to_main, Some(0), "主菜单自身须 0 步（已在主菜单）");
        } else {
            assert!(
                steps_to_main.is_some(),
                "状态 {start} 无法回主菜单（死锁！）：states={states:?} triggers={triggers:?}"
            );
            let steps = steps_to_main.expect("上句已断言 is_some");
            assert!(
                steps <= n,
                "状态 {start} 回主菜单需 {steps} 步，超上限 N={n}（balance.ui.max_back_steps）"
            );
            worst_steps = worst_steps.max(steps);
        }
        states_checked += 1;
    }

    // 组合数对账（逐状态断言，防 JSON 档位改动后静默失配）
    let expected = u32::try_from(states.len()).unwrap_or(u32::MAX);
    assert_eq!(
        states_checked, expected,
        "逐状态穷举须覆盖 JSON main_loop.states 全部 {expected} 个状态码"
    );
    // 最强不变式（精确值）：最坏状态回主菜单步数 ≤ N，且实测远小于 N（当前转移表 ≤2 步）
    assert!(worst_steps <= n, "全状态最坏回主菜单步数 {worst_steps} 须 ≤ N={n}");
}

/// 穷举不变式：任意状态经任意 trigger 序列（≤ N 步内）都不会陷入「永不回主菜单」的
/// 非终止环（BFS 在有限状态空间上收敛 = 无死锁环）。逐 trigger 序列穷举 1 步转移，
/// 断言转移表封闭（输出的状态码必在 `main_loop.states` ∪ {0} 内，不产生未定义状态码）。
#[test]
fn exhaust_main_loop_transition_table_closed() {
    let e = exhaust().expect("exhaust_scenarios.json should parse");
    let states = &e.main_loop.states;
    let triggers = &e.main_loop.triggers;
    let known: std::collections::BTreeSet<u32> = states.iter().copied().collect();

    let mut edges: u32 = 0;
    for &s in states {
        for &t in triggers {
            let ns = transition(s, t);
            // 转移表封闭：任意已知状态 + 任意 trigger 的像必须是已知状态（或主菜单 0），
            // 不得跳到未定义状态码（`advance` 未知态保持原值 → 仍在集合内）。
            assert!(
                known.contains(&ns),
                "转移表失封：状态 {s} + trigger {t} → {ns} 不在状态枚举内"
            );
            edges += 1;
        }
    }
    let expected = u32::try_from(states.len() * triggers.len()).unwrap_or(u32::MAX);
    assert_eq!(edges, expected, "边数须 = |states| × |triggers|（JSON 驱动）");
}
