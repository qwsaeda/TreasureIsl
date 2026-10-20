//! S18.5：轻量确定性粒子（金币光 / 战斗命中火花）。
//!
//! 设计口径（DoD：粒子单测可复现）：
//! - **确定性**：独立 LCG（FX 专用，业务 RNG 单源 `rules::rng::GameRng` 不动——FX 不进业务判定）；
//!   同 seed + 同 spawn/advance 序列 = 同画面（视觉验收可复现）；
//! - **渲染面**：CPU Framebuf 无 alpha 通道 → 淡出 = RGB565 亮度 4 档量化（[`FADE_LEVELS`]），
//!   方形小粒（1–3px）；z 层 = [`crate::z_layers::Z_FX`]（调用序 = 层序）；
//! - **池上限** [`POOL_CAP`]，超限逐最旧（FIFO 语义：Vec 顺序即出生序）。
//!
//! 触发点（`game_app::handle`）：战斗回合命中 → [`ParticleFx::spawn_hit_spark`]；
//! 交易金币变动 → [`ParticleFx::spawn_coin_glint`]。快照路径（`render_to_framebuf`）
//! 先 `clear()`：像素基线不含 FX（EXP-079 稳态快照口径不变）。

/// 粒子池上限（超限逐最旧）。
pub const POOL_CAP: usize = 64;

/// 淡出亮度档数（0 = 熄灭，3 = 全亮；帧无 alpha 通道 → 亮度量化）。
pub const FADE_LEVELS: u32 = 4;

/// 金币金光基色（RGB565：金）。
pub const COIN_GOLD_565: u16 = 0xF7BE;

/// 命中火花基色（RGB565：橙红）。
pub const HIT_SPARK_565: u16 = 0xFC0A;

/// 粒子特效种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FxKind {
    /// 金币光（交易金币变动）。
    CoinGlint,
    /// 战斗命中火花。
    HitSpark,
}

/// 单粒子（确定性演化：位置 = 出生点 + 速度×age，无随机位移增量）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle {
    /// 世界 px（1080×1920 逻辑坐标）。
    pub x: f32,
    /// 世界 px。
    pub y: f32,
    /// 速度 px/ms。
    pub vx: f32,
    /// 速度 px/ms。
    pub vy: f32,
    /// 出生至现在（ms）。
    pub age_ms: u32,
    /// 总寿命（ms）；`age_ms >= ttl_ms` 即死。
    pub ttl_ms: u32,
    /// RGB565 基色（淡出时按 [`FADE_LEVELS`] 亮度量化）。
    pub color: u16,
}

/// RGB565 亮度量化：`level` ∈ 0..=[`FADE_LEVELS`-1]（0 熄灭，3 全亮；均匀分档）。
#[must_use]
pub fn dim565(color: u16, level: u32) -> u16 {
    let lv = level.min(FADE_LEVELS - 1);
    let div = FADE_LEVELS - 1; // 3
    let v = u32::from(color);
    let r = ((v >> 11) & 0x1F) * lv / div;
    let g = ((v >> 5) & 0x3F) * lv / div;
    let b = (v & 0x1F) * lv / div;
    u16::try_from((r << 11) | (g << 5) | b).unwrap_or(u16::MAX)
}

/// 特效模拟状态（纯：同 seed + 同操作序列 = 同画面）。
#[derive(Debug, Clone)]
pub struct ParticleFx {
    /// FX 专用 LCG 状态（不进业务 RNG）。
    lcg: u64,
    /// 粒子池（Vec 顺序 = 出生序，超限逐最旧）。
    pool: Vec<Particle>,
}

impl ParticleFx {
    /// 确定性构造（`seed` 固定 = 跨运行可复现；`GameApp` 用常量 seed）。
    #[must_use]
    pub fn deterministic(seed: u64) -> Self {
        Self { lcg: seed.max(1), pool: Vec::new() }
    }

    /// 清空（快照路径口径）。
    pub fn clear(&mut self) {
        self.pool.clear();
    }

    /// 存活粒子（只读；`None` 语义 = 空池）。
    #[must_use]
    pub fn active(&self) -> &[Particle] {
        &self.pool
    }

    fn next_u32(&mut self, bound: u32) -> u32 {
        // 64 位 LCG（Numerical Recipes 常数）：FX 够用（非密码学、非业务）。
        self.lcg = self
            .lcg
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.lcg >> 32) as u32) % bound
    }

    /// 金币光：短促小亮片（确定性 6 枚，微漂移，ttl 300–600ms，金色）。
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn spawn_coin_glint(&mut self, x: f32, y: f32) {
        for _ in 0..6 {
            let vx = (self.next_u32(20) as f32 - 10.0) * 0.02;
            let vy = -(self.next_u32(30) as f32) * 0.02 - 0.15;
            let ttl_ms = 300 + self.next_u32(300);
            self.push_particle(Particle { x, y, vx, vy, age_ms: 0, ttl_ms, color: COIN_GOLD_565 });
        }
    }

    /// 命中火花：高速短迹（确定性 8 枚，ttl 120–200ms，橙红）。
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn spawn_hit_spark(&mut self, x: f32, y: f32) {
        for _ in 0..8 {
            let ang = self.next_u32(628) as f32 * 0.01; // 0..2π（粗量化）
            let sp = 0.6 + self.next_u32(4) as f32 * 0.15;
            let ttl_ms = 120 + self.next_u32(80);
            self.push_particle(Particle {
                x,
                y,
                vx: ang.cos() * sp,
                vy: ang.sin() * sp - 0.2,
                age_ms: 0,
                ttl_ms,
                color: HIT_SPARK_565,
            });
        }
    }

    fn push_particle(&mut self, p: Particle) {
        if self.pool.len() >= POOL_CAP {
            // 超限逐最旧（出生序 = Vec 序）。
            self.pool.remove(0);
        }
        self.pool.push(p);
    }

    /// 推进 `dt_ms`（位置积分 + 寿命衰老 + 过期剔除）。
    pub fn advance(&mut self, dt_ms: u32) {
        if dt_ms == 0 {
            return;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        let dt = dt_ms as f32;
        for p in &mut self.pool {
            p.age_ms = p.age_ms.saturating_add(dt_ms);
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
        self.pool.retain(|p| p.age_ms < p.ttl_ms);
    }

    /// 渲染 op（z 层 = [`crate::z_layers::Z_FX`]；调用序 = 层序）。
    /// 返回 (x, y, w, h, 淡出色)；粒尺寸 = 速度越大越大（1–3px）。
    #[must_use]
    pub fn render_rects(&self) -> Vec<(i32, i32, i32, i32, u16)> {
        let mut out = Vec::with_capacity(self.pool.len());
        for p in &self.pool {
            // 寿命进度 0..1 → 亮度档 3 → 0（advance 已剔除 age≥ttl，step ∈ 0..=3）。
            let step = p.age_ms.saturating_mul(FADE_LEVELS) / p.ttl_ms.max(1);
            let level = FADE_LEVELS - 1 - step.min(FADE_LEVELS - 1);
            let speed = p.vx.hypot(p.vy);
            let size = 1 + u32::from(speed > 0.3) + u32::from(speed > 0.8);
            #[allow(clippy::cast_possible_truncation)]
            let (px, py) = (p.x.floor() as i32, p.y.floor() as i32);
            out.push((
                px,
                py,
                i32::try_from(size).unwrap_or(1),
                i32::try_from(size).unwrap_or(1),
                dim565(p.color, level),
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: u64 = 0xC0_FF_EE;

    #[test]
    fn deterministic_same_seed_same_sequence() {
        let mut a = ParticleFx::deterministic(SEED);
        let mut b = ParticleFx::deterministic(SEED);
        a.spawn_coin_glint(500.0, 900.0);
        b.spawn_coin_glint(500.0, 900.0);
        for _ in 0..50 {
            a.advance(16);
            b.advance(16);
        }
        assert_eq!(a.render_rects(), b.render_rects(), "同 seed 同序列须同画面");
    }

    #[test]
    fn different_seed_diverges() {
        let mut a = ParticleFx::deterministic(SEED);
        let mut b = ParticleFx::deterministic(SEED + 1);
        a.spawn_hit_spark(100.0, 100.0);
        b.spawn_hit_spark(100.0, 100.0);
        assert_ne!(a.active()[0].vx, b.active()[0].vx, "异 seed 轨迹须分叉");
    }

    #[test]
    fn ttl_expiry_clears_pool() {
        let mut fx = ParticleFx::deterministic(SEED);
        fx.spawn_hit_spark(0.0, 0.0);
        assert_ne!(fx.active().len(), 0, "命中火花须生成粒子");
        // 最大 ttl = 200ms（hit_spark）→ 推进 201ms 必空。
        fx.advance(201);
        assert!(fx.active().is_empty(), "ttl 耗尽粒子须剔除");
    }

    #[test]
    fn pool_cap_evicts_oldest() {
        let mut fx = ParticleFx::deterministic(SEED);
        for _ in 0..(POOL_CAP + 10) {
            fx.spawn_hit_spark(0.0, 0.0);
        }
        assert_eq!(fx.active().len(), POOL_CAP, "池超上限须逐最旧");
        // 全部同 ttl 起算：存活 = 最新一批（最旧 10 枚已被逐出）。
        fx.advance(1);
        assert_eq!(fx.active().len(), POOL_CAP);
    }

    #[test]
    fn fade_levels_monotone_to_black() {
        let full = dim565(COIN_GOLD_565, FADE_LEVELS - 1);
        assert_eq!(full, COIN_GOLD_565, "满亮档 = 基色");
        let dark = dim565(COIN_GOLD_565, 1);
        let off = dim565(COIN_GOLD_565, 0);
        assert_eq!(off, 0, "熄灭档 = 全黑");
        let lum = |c: u16| -> u32 {
            u32::from((c >> 11) & 0x1F) * 32 + u32::from((c >> 5) & 0x3F) * 16 + u32::from(c & 0x1F)
        };
        assert!(lum(dark) < lum(full), "亮档须单调递减");
        assert_eq!(lum(off), 0);
    }

    #[test]
    fn advance_moves_particles_by_velocity() {
        let mut fx = ParticleFx::deterministic(SEED);
        fx.clear();
        fx.pool.push(Particle {
            x: 0.0,
            y: 0.0,
            vx: 0.5,
            vy: -0.25,
            age_ms: 0,
            ttl_ms: 1000,
            color: COIN_GOLD_565,
        });
        fx.advance(100);
        let p = &fx.active()[0];
        assert!(
            (p.x - 50.0).abs() < 1e-3 && (p.y + 25.0).abs() < 1e-3,
            "位置 = 出生点 + v×dt（{p:?}）"
        );
        assert_eq!(p.age_ms, 100);
    }
}
