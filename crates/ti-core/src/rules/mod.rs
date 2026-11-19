//! 正式实现模块：规则/状态机（唯一事实源在 `docs/business/` + `assets/data/*.json`）。

// `rules/battle`（旧 HP 回合制 `BattleSession`/`CombatBonus`/`CombatAction`/道具）已删
// （S34.6 D5 兵力模型迁移，现行战斗 = `force::resolve` 一次性结算；B-01 4 道具死内容随清）。
pub mod board;
pub mod boat;
pub mod build;
pub mod cards;
pub mod cards_draw;
pub mod events;
pub mod force;
pub mod main_loop;
pub mod main_map;
pub mod menu;
pub mod merchant;
pub mod messages;
pub mod progression;
pub mod render;
pub mod rng;
pub mod saveload;
pub mod saveload_json;
pub mod selection;
pub mod territory;
pub mod territory_v2;
pub mod trade;
