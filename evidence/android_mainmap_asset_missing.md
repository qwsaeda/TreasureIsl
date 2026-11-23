# 真问题登记：Android 主地图资产不显示（2026-11-23）

## 现象（真机 redroid 10.0.182.152:5555，装 0.3.0 APK）
- 首页羊皮藏宝图 + 角色立绘：**正常显示**
- 选角页角色立绘 + 列表：**正常显示**
- 选图页地图卡片 + 列表：**正常显示**
- **主地图：只有黄铜环线 + P1/P2 棋子 + 深蓝/棕色块，岛屿贴图/灯塔/宝箱/羊皮底全缺**

## 已排除（实测）
- APK 内资产齐全：`assets/ui/v6/isle_land.png`(2.7MB)/island.png(4.3MB)/tile_* + chart_full/ocean_bg 全在，137 个 ui 资产。
- `asset_present()` 在 Android 恒 true（不阻塞 blit），不是它拦的。
- 剥前缀 reader（StripAssetsReader）已注册，首页/选角/选图图都正常 = AssetServer 路径没问题。
- **桌面 native `--shot` 当前源码主地图资产完整**（岛屿+灯塔+宝箱+羊皮底）——源码本身没问题。
- 并行 agent 在我编译后又改了源码（指纹 2e49→7866 不一致），但桌面用当前源码编译仍正常。

## 根因方向（未坐实，需下一步）
平台渲染路径差异：桌面 `--shot` 走 CPU 帧缓冲 `blit_image`（资产在）；Android `chrome_in_bevy_ui=true`
走 GPU 渲染，主地图 v6 的 `host.blit_image`（main_map.rs:2284+ 的 isle_land/lighthouse/treasure）
可能没进 draw_list ⇒ spawn_draw_list 只画 chrome 原语 ⇒ GPU 不画这些 PNG。
即：1080 游戏世界内容（blit_image 画的）在 Bevy GPU 路径下未被提取成 sprite。
这与此前 S29 记录的「chrome_in_bevy_ui=true 时 CPU 路径跳过部分 blit」同类。

## 复现步骤
1. 装 dist/TreasureIsl-0.3.0-aarch64.apk 到 redroid/真机
2. 触摸：新游戏→勾2席→点已勾确认→选图→点 m01
3. 观察主地图：只有环线+棋子，无岛屿/羊皮底

## 下一步建议
查 `host.blit_image` 在 `chrome_in_bevy_ui=true` 时的 GPU 路径：是否进了 draw_list。
若是 → 让 blit_image 在 GPU 路径也 spawn sprite（或在 bevy_ui 层用 ImageNode 画主地图资产）。
注意排除并行 agent 的干扰（建议等其收工后用干净源码重编 APK 复验一次）。

## 状态
未修。登记避免被「设备限制」搪塞——首页/选角/选图都正常，唯独主地图资产缺，是真 bug 不是设备问题。
