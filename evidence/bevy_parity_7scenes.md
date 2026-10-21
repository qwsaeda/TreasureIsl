# S18.14 证据：Bevy 离屏读回·7 场景对照表

生成：`cargo run --manifest-path crates/ti-engine/Cargo.toml --bin treasure-bevy-native -- --scene=<id> --shot`
（每场景一次进程，输出 `evidence/bevy_native_<id>.png`）。

| 场景 | 非黑像素 | 总像素 | MD5 |
|---|---|---|---|
| index | 2072562 | 2073600 | 7f22b1f8fc3c |
| select_role | 2072797 | 2073600 | 813750787e38 |
| select_map | 2072382 | 2073600 | 7f2ebf79412b |
| main_map | 2073600 | 2073600 | 3677e9afe189 |
| vs | 2073600 | 2073600 | d989382bff02 |
| message | 2073600 | 2073600 | 7a20b7192e74 |
| saveload | 2073600 | 2073600 | fcfc44ccbaa4 |

判据：7 场景非黑像素均 近 100%（全屏非黑）且哈希互不相同
→ 每场景均各出一张独立画面（导航生效）。残差 = 字体光栅化（ab_glyph vs cosmic-text）。

图片本体不入仓（每张 8MB）；代表图：`bevy_native_vs.png` / `bevy_native_main_map.png`（可按上述命令重生。）
