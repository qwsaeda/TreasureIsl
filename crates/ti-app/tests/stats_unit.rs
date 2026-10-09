//! S4.3 FPS / 呈现计数单测（置于集成测试目录，豁免门禁的 unwrap/expect 扫描）。
//!
//! `stats` 模块仅在 `ti-app` 开启 `stats` feature 时导出；本测试需随之编译：
//! `cargo test -p ti-app --features stats --test stats_unit`。

#[cfg(feature = "stats")]
mod body {
    use ti_app::stats::FpsMeter;

    #[test]
    fn window_rolls_after_one_second() {
        let mut m = FpsMeter::new();
        for i in 0..60u128 {
            let r = m.tick(i * 16, 0, 0);
            assert_eq!(r, None, "第 {i} 帧（<1s）不应触发");
        }
        let r = m.tick(1000, 0, 0);
        assert!(r.is_some(), "应在 1s 窗口到期时返回 Some");
        let (avg, u, s) = r.expect("已断言 Some");
        assert!(avg > 60.0 && avg < 65.0, "avg={avg}");
        assert_eq!(u, 0);
        assert_eq!(s, 0);
        for t in 1016u128..1984 {
            let r = m.tick(t, 0, 0);
            assert_eq!(r, None, "窗口滚动后 1s 内不应触发 ({t})");
        }
        let r2 = m.tick(2000, 0, 0);
        assert!(r2.is_some());
    }

    #[test]
    fn diffs_presenter_cumulative_counters() {
        let mut m = FpsMeter::new();
        // 累计 uploads=1（第 16ms 置脏一次，之后保持 1）
        let r0 = m.tick(0, 0, 0);
        assert_eq!(r0, None);
        let r1 = m.tick(16, 1, 0);
        assert_eq!(r1, None);
        let r2 = m.tick(32, 1, 1);
        assert_eq!(r2, None);
        // 窗口到期：本窗口 uploads 增量 = 1（第 16ms 脏帧），skipped 增量 = 1（第 32ms 跳帧）
        let mut report = None;
        for t in 48u128..=1000 {
            let r = m.tick(t, 1, 1);
            if r.is_some() {
                report = r;
            }
        }
        assert!(report.is_some(), "窗口应到期");
        let (avg, u, s) = report.expect("已断言 Some");
        assert_eq!(u, 1, "窗口内上传帧增量应为 1");
        assert_eq!(s, 1, "窗口内跳帧增量应为 1");
        assert!(avg > 0.0);
        assert_eq!(m.total_uploads(), 1, "全时段上传累计 = 1");
        assert_eq!(m.total_skipped(), 1, "全时段跳帧累计 = 1");
    }
}
