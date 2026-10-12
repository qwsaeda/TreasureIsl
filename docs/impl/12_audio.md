# 12 — 音频系统实现规格

> 单一事实源：本文件是 `ti-platform::audio` + `ti-app::audio_prep` 的实现规格。
> 关联文档：`docs/design/constants.md` §12（音效资源表）。

---

## 1. 音频资源清单

### 1.1 音效（WAV）

| 文件 | 触发场景 | 事件码 | 说明 |
|---|---|---|---|
| `music\press.wav` | 点击有效菜单项 | 13 | 按键音 |
| `music\invalid.wav` | 无效操作 | 37 | 错误音 |
| `music\victory.wav` | 战斗胜利 | — | 胜利音 |

### 1.2 BGM（OGG）

| 文件 | 触发场景 | 循环 | 说明 |
|---|---|---|---|
| `music\glmusic.mod` → `glmusic.ogg` | 主菜单/主地图 | 是 | 循环播放 |

### 1.3 资源映射

```rust
pub enum Sfx {
    Press,      // press.wav
    Invalid,    // invalid.wav
    Victory,    // victory.wav
}

pub enum Bgm {
    MainTheme,  // glmusic.ogg
}

pub struct AudioResourceMap {
    sfx: HashMap<Sfx, Vec<u8>>,
    bgm: HashMap<Bgm, Vec<u8>>,
}
```

---

## 2. 音频通道

### 2.1 通道架构

| 通道类型 | 数量 | 用途 | 并发限制 |
|---|---|---|---|
| BGM 通道 | 1 | 背景音乐 | 1（独占） |
| 音效通道 | 8 | 音效播放 | ≤8 并发 |

### 2.2 通道状态

> 实际 `ti-platform::audio`（`audio.rs:51`）为 struct 非 enum：
> `ChannelState { active: bool, volume: u8 (0..=255), position: usize, looping: bool }`。
> 本节的 `AudioChannel` + `ChannelState` enum 为设计占位。
> ⚠️ 以下 enum 为**设计占位**（`@deprecated`），真实实现是 `struct ChannelState { active, volume, position, looping }`（`audio.rs:51`），勿照抄。

```rust
pub struct AudioChannel {
    id: u8,
    state: ChannelState,
    volume: f32,        // 0.0 ~ 1.0
    source: Option<AudioSource>,
}

pub enum ChannelState {
    Idle,
    Playing,
    Paused,
    Stopped,
}
```

---

## 3. 音量控制

### 3.1 音量通道

| 音量通道 | 范围 | 默认值 | 存储位置 |
|---|---|---|---|
| BGM 音量 | 0~100% | 60% | `settings.dat` offset 6 |

> 注：settings 侧 0..=100，mixer 侧 0..=255（`audio.rs` `ChannelState.volume: u8`），换算 `0..=100 × 255/100`。
| 音效音量 | 0~100% | 80% | `settings.dat` offset 7 |
| 主音量 | 0~100% | 100% | 运行时计算 |

### 3.2 音量计算

```
effective_volume = channel_volume * master_volume * mute_factor

mute_factor = 0.0 if muted else 1.0
```

### 3.3 静音控制

| 触发 | 行为 |
|---|---|
| 用户切换静音 | `mute_factor = 0.0`，所有通道静音 |
| 浏览器 `visibilitychange` | 后台自动静音 |
| 恢复前台 | 恢复原音量 |

---

## 4. BGM 播放

### 4.1 播放流程

```
play_bgm(bgm):
    if bgm_channel.state == Playing:
        fade_out(bgm_channel, 500ms)     // 淡出当前
    load_source(bgm_channel, bgm)
    set_volume(bgm_channel, bgm_volume * master_volume)
    start_loop(bgm_channel)               // 循环播放
```

### 4.2 淡入淡出

```
fade_out(channel, duration_ms):
    start_vol = channel.volume
    steps = duration_ms / 10
    for i in 0..steps:
        channel.volume = start_vol * (1 - i / steps)
        sleep(10ms)
    stop(channel)

fade_in(channel, duration_ms):
    steps = duration_ms / 10
    for i in 0..steps:
        channel.volume = target_vol * (i / steps)
        sleep(10ms)
```

### 4.3 循环播放

> 注：设计占位；当前 BGM 走 `audio_prep` 预转码 OGG + `Mixer.play`（`crates/ti-platform/src/audio.rs:64`），
> 无独立 `BgmPlayer`/`AudioResourceMap` struct。

```rust
pub struct BgmPlayer {
    channel: AudioChannel,
    loop_enabled: bool,
    fade_duration_ms: u32,
}

impl BgmPlayer {
    pub fn play(&mut self, bgm: Bgm, resources: &AudioResourceMap);
    pub fn stop(&mut self);
    pub fn fade_switch(&mut self, new_bgm: Bgm, resources: &AudioResourceMap);
    pub fn set_volume(&mut self, vol: f32);
}
```

---

## 5. 音效播放

### 5.1 播放流程

```
play_sfx(sfx):
    channel = find_idle_channel()          // 找空闲通道
    if channel == None:
        channel = steal_oldest_channel()   // 抢占最旧通道
    load_source(channel, sfx)
    set_volume(channel, sfx_volume * master_volume)
    play_once(channel)                     // 单次播放
```

### 5.2 通道抢占

```
steal_oldest_channel():
    oldest = channels[0]
    for ch in channels:
        if ch.start_time < oldest.start_time:
            oldest = ch
    stop(oldest)
    return oldest
```

### 5.3 Rust 实现

```rust
pub struct SfxPlayer {
    channels: [AudioChannel; 8],
    sfx_volume: f32,
    master_volume: f32,
}

impl SfxPlayer {
    pub fn play(&mut self, sfx: Sfx, resources: &AudioResourceMap) {
        let data = match resources.sfx.get(&sfx) {
            Some(d) => d,
            None => return,
        };
        let channel = self.find_idle_channel()
            .unwrap_or_else(|| self.steal_oldest_channel());
        channel.load(data);
        channel.set_volume(self.sfx_volume * self.master_volume);
        channel.play_once();
    }

    fn find_idle_channel(&self) -> Option<&mut AudioChannel> {
        self.channels.iter_mut().find(|c| c.state == ChannelState::Idle)
    }

    fn steal_oldest_channel(&mut self) -> &mut AudioChannel {
        // 抢占最旧通道
        let idx = self.channels.iter()
            .enumerate()
            .min_by_key(|(_, c)| c.start_time)
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.channels[idx].stop();
        &mut self.channels[idx]
    }
}
```

---

## 6. 浏览器自动播放策略

### 6.1 自动播放限制

| 浏览器 | 策略 |
|---|---|
| Chrome | 需要用户交互后才能播放音频 |
| Safari | 需要用户交互后才能播放音频 |
| Firefox | 需要用户交互后才能播放音频 |

### 6.2 处理方案

```
init_audio():
    if browser && audio_context.state == "suspended":
        show_overlay("点击任意处开始")
        on_first_click:
            audio_context.resume()
            start_bgm()
```

### 6.3 Rust/WASM 实现

```rust
pub struct AutoplayHandler {
    user_gesture_detected: bool,
    pending_bgm: Option<Bgm>,
}

impl AutoplayHandler {
    pub fn on_user_gesture(&mut self) {
        self.user_gesture_detected = true;
        if let Some(bgm) = self.pending_bgm.take() {
            self.start_bgm(bgm);
        }
    }

    pub fn request_bgm(&mut self, bgm: Bgm) {
        if self.user_gesture_detected {
            self.start_bgm(bgm);
        } else {
            self.pending_bgm = Some(bgm);
        }
    }
}
```

---

## 7. 音频资源加载

### 7.1 加载时机

| 资源 | 加载时机 | 缓存 |
|---|---|---|
| `press.wav` | 首屏 | 是 |
| `invalid.wav` | 首屏 | 是 |
| `victory.wav` | 战斗场景 | 按需 |
| `glmusic.ogg` | 主菜单 | 是 |

### 7.2 加载流程

```
load_audio_resources():
    for sfx in [Press, Invalid]:
        data = load_file("music/{sfx}.wav")
        resources.sfx.insert(sfx, data)
    // BGM 按需加载
```

---

## 8. 音频状态机

### 8.1 状态转换

```
┌─────────┐  play()   ┌─────────┐  pause()  ┌─────────┐
│  Idle   │ ────────► │ Playing │ ────────► │ Paused  │
└─────────┘           └─────────┘           └─────────┘
    ▲                     │                     │
    │     stop()          │ stop()             │ resume()
    └─────────────────────┴─────────────────────┘
```

### 8.2 状态转换表

| 当前状态 | 操作 | 目标状态 | 行为 |
|---|---|---|---|
| Idle | `play()` | Playing | 加载并播放 |
| Playing | `pause()` | Paused | 暂停 |
| Playing | `stop()` | Idle | 停止并释放 |
| Paused | `resume()` | Playing | 恢复播放 |
| Paused | `stop()` | Idle | 停止并释放 |
| Idle | `stop()` | Idle | 无操作 |

---

## 9. 平台音频后端

### 9.1 后端选择

| 端 | 后端 | 说明 |
|---|---|---|
| 桌面 native | `rodio` | 跨平台音频 |
| 浏览器 wasm | `Web Audio API` | 浏览器原生 |
| Android | `OpenSL ES` | NDK 音频 |

### 9.2 统一接口

```rust
#[async_trait]
pub trait AudioBackend {
    async fn play_wav(&mut self, data: &[u8], volume: f32) -> Result<(), AudioErr>;
    async fn play_ogg_loop(&mut self, data: &[u8], volume: f32) -> Result<Handle, AudioErr>;
    async fn pause(&mut self, handle: Handle) -> Result<(), AudioErr>;
    async fn resume(&mut self, handle: Handle) -> Result<(), AudioErr>;
    async fn stop(&mut self, handle: Handle) -> Result<(), AudioErr>;
    async fn set_volume(&mut self, handle: Handle, vol: f32) -> Result<(), AudioErr>;
}

#[derive(Debug)]
pub enum AudioErr {
    DecodeError,
    PlaybackError,
    ResourceNotFound,
}
```

---

## 10. 音频调试

### 10.1 调试命令

| 命令 | 功能 |
|---|---|
| `audio_test_sfx(Sfx::Press)` | 播放指定音效 |
| `audio_test_bgm(Bgm::MainTheme)` | 播放指定 BGM |
| `audio_dump_state()` | 打印通道状态 |

### 10.2 日志

```
[AUDIO] channel 0: Playing Press (vol=0.80)
[AUDIO] channel 1: Playing MainTheme (vol=0.60, loop=true)
[AUDIO] mute: enabled
```
