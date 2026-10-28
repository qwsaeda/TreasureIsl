//! S28.4 席位多选测试（`docs/business/territory.md` §1；`rules::selection::seat_toggle/seat_pool`）。
//!
//! 判据：`cargo test -p ti-core --test selection_seats` 全绿。

use ti_core::domain::selection::Character;
use ti_core::rules::selection::{seat_pool, seat_toggle};
use ti_core::rules::territory::SeatPool;

/// §1 不变量：最后一名玩家席不可取消（`players ≥ 1`）。
#[test]
fn seat_selection_keeps_at_least_one_player() {
    let mut seats = vec![Character::Panda];
    let sfx = seat_toggle(&mut seats, Character::Panda, 5);
    assert_eq!(sfx, ti_core::domain::flow::Sfx::Invalid, "唯一玩家席取消 = 无效");
    assert_eq!(seats, vec![Character::Panda], "状态不变");
}

/// §1：toggle 加入/移除 + 角色 index 升序 + 去重。
#[test]
fn seat_selection_toggle_and_order() {
    let mut seats: Vec<Character> = Vec::new();
    assert!(seats.is_empty(), "初始空");
    for ch in [Character::Drake, Character::Panda, Character::Katharina, Character::Kidd] {
        let sfx = seat_toggle(&mut seats, ch, 5);
        assert_eq!(sfx, ti_core::domain::flow::Sfx::Press);
    }
    assert_eq!(
        seats,
        vec![Character::Panda, Character::Kidd, Character::Drake, Character::Katharina],
        "index 升序"
    );
    // 重复 toggle = 无变化（幂等语义：已选 ⇒ 尝试移除；4 席 > 1 可移除）
    let sfx = seat_toggle(&mut seats, Character::Panda, 5);
    assert_eq!(sfx, ti_core::domain::flow::Sfx::Press);
    assert!(!seats.contains(&Character::Panda), "移除成功");
    // 越界角色 = 无效（Katharina index 4 ≥ count 3）
    let sfx = seat_toggle(&mut seats, Character::Katharina, 3);
    assert_eq!(sfx, ti_core::domain::flow::Sfx::Invalid, "index ≥ count = 无效");
    assert_eq!(seats.len(), 3);
}

/// §1：玩家选 ≥1 ⇒ `players + ai = 5`；未选中角色自动 AI 席。
#[test]
fn seat_pool_from_selection() {
    let seats = vec![Character::Panda];
    let pool = seat_pool(&seats);
    assert_eq!(pool.player_count(), 1, "1 玩家");
    assert_eq!(pool.ai_count(), 4, "4 AI（= 原版「每局 4 个 AI 玩家」）");
    assert_eq!(pool.len(), 5);
    // 玩家 2 ⇒ AI 3
    let seats2 = vec![Character::Panda, Character::Kidd];
    let pool2 = seat_pool(&seats2);
    assert_eq!(pool2.player_count(), 2);
    assert_eq!(pool2.ai_count(), 3);
    // 玩家 5 ⇒ 无 AI
    let pool5 = seat_pool(Character::all().as_slice());
    assert_eq!(pool5.player_count(), 5);
    assert_eq!(pool5.ai_count(), 0);
    // 席位顺序 = 角色 index 升序（回合顺序）
    let order: Vec<&str> = pool2.iter().map(|s| s.character).collect();
    assert_eq!(order, ["panda", "tichy", "kidd", "drake", "katharina"]);
}

/// 席位池与 `SeatPool::build` 直连（同输入同输出；不变量一致）。
#[test]
fn seat_pool_matches_direct_build() {
    let seats = vec![Character::Tichy, Character::Drake];
    let pool = seat_pool(&seats);
    let direct = SeatPool::build(&[1, 3]);
    assert_eq!(
        pool.player_seats(),
        direct.player_seats(),
        "seat_pool ≡ SeatPool::build(index 映射)"
    );
}
