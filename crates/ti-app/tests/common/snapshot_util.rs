/// S15.6 QA125：TTF 在位前置断言——快照入口渲染帧前 `shell.ttf_font` 必须 `Some`
/// （基线须为 44px 真字形态）。TTF 缺失（cwd 回溯不到 `assets/fonts/` 或字体文件缺失）
/// → panic 带诊断，**禁止**静默 Regenerated 豆腐基线入库（QA016 类空窗：基线缺失时
/// Regenerated 接受 + 无 diff 判据，缺字体环境会把 12px 豆腐帧当基线写盘）。
pub fn require_ttf_present(app: &ti_app::game_app::GameApp, context: &str) {
    assert!(
        app.shell().ttf_font.is_some(),
        "{context} 快照入口 TTF 缺失（assets/fonts/lxgw-wenkai-reg.ttf）：menu_shell 自动加载 \
         依赖 cwd 向上 8 级回溯找 assets/fonts/，检查测试 cwd / 字体文件在位；禁止豆腐基线入库（S15.6 QA125）"
    );
}
