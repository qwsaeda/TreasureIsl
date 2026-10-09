# S3.1 主循环 / 场景管理 — 逆向规格

数据源：`ref/pseudocode/*.c`（IDA Hex-Rays）、`ref/functions.csv`、`TreasureIsl.exe` 直接读 `.data` 段（file offset = RVA，ImageBase 0x10000 且各段 file offset 与 RVA 一致，已用已知 vtable 内容校验）。
结论标注：`证据` = 指到具体伪代码行；`推测` = 需进一步验证（标注待验证方法）。

---

## 1. WinMain 流程图

```
start (0x69FB0)
  └─ cinit(); WinMain(...); exit(v8);                     [00069FB0_start.c:8-10]

WinMain (0x45458)                                         [00045458_WinMain.c]
  1. dword_81218 = &unk_7D218; dword_811A8 = 132; ...     行14-17  (全局初始化常量)
  2. sub_456F8(&dword_811A8, 0, hInstance, nShowCmd)      行18    (框架初始化，含 3×InitializeCriticalSection + 遍历已注册类链表)
  3. sub_45914(&v8)                                       行19    (创建应用对象，见 §3)
  4. sub_3D430(v11)                                       行20    (创建全局单例 off_811A0，见 §5)
  5. v10[7] = &off_6FD88; v8 = off_6FD60                  行21-22 (vtable 指针写入，见 §3)
  6. 两段 sub_12F04+sub_1318C（宽字符串拷贝）              行23-26  名称 "TreasureIsland" / 第二个变体 → v9 / v10
  7. if (sub_62A14(&v8))  v6 = sub_62B84(&v8)             行28-29  (窗口/全屏设置 → 主消息泵循环)
  8. sub_457B0(&dword_811A8)                              行30    (框架清理：UnregisterClassW×N + 3×DeleteCriticalSection + HeapDestroy)
  9. sub_3D49C(v11)                                       行31    (off_811A0 单例析构)
 10. sub_45C68(&v8)                                       行32    (app 对象析构 → sub_45A10)
 11. return v6;
```

`sub_456F8`（框架基类初始化）：写 `+4/+8/+12 = hInstance`，`+16 = a2(0)`，`+20..+24 = 0`，3×`InitializeCriticalSection`（`+28/+48/+68`），`+92=0`、`+108=1`、`+116..+128=0`；随后若 `*(+16)!=0` 遍历对象链表（`v5[8]` = vtable[+32] 调用，步长按 `*(+0)==88` 判 7/9）——**推测**为「遍历已注册窗口类并对每个调用某初始化回调」。证据：`000456F8_sub_456F8.c:11-38`。

`sub_457B0`（框架基类清理）：遍历 `+96/+100`（类名数组/计数）逐个 `UnregisterClassW`；`sub_45580` free；遍历 `+16` 对象链表（`v4[4]` = 对象指针，调用 `+8` 虚函数与 `v5=v4[8]` 虚函数）；3×`DeleteCriticalSection`；遍历 `+128` 链表逐个 `operator delete`；若 `+24 && +108` 则 `HeapDestroy`（`+120` 数组或 `+24`）。证据：`000457B0_sub_457B0.c:16-61`。

## 2. 消息泵（sub_62B84, 0x62B84）

```
sub_62B84(_BYTE* a1)                    a1 = app 对象（*a1 = vtable, a1+40 = 退出标志）
  while (1) {
    while (PeekMessageW(&pMsg, 0,0,0, PM_REMOVE)) {
      if (pMsg.message == WM_QUIT(0x12)) return pMsg.lParam;
      DispatchMessageW(&pMsg);
    }
    if (a1[40]) break;                                   // 退出标志置位 → 跳出帧循环
    (*(a1->vtable+16))(a1);                               // 帧函数调用（vtable[+16]，见 §3）
  }
  if (GetMessageW(&pMsg,0,0,0)) DispatchMessageW(&pMsg);   // 阻塞取最后一条消息
  return pMsg.lParam;
```
证据：`00062B84_sub_62B84.c:8-24`。

**进入条件**：WinMain 只在 `sub_62A14(&v8) != 0` 时调用 `sub_62B84`（`00045458_WinMain.c:28`）。

`sub_62A14`（窗口/全屏初始化）：
```
if (IsWindow(*(a1+48)) || (*(a1+36) && sub_62C24(a1)))  return 0;   // 已有窗口 / 能 FindWindowW(类名) 并 SetForegroundWindow → 已在运行，不重启
dword_7FBB0 = 3; dword_7FBCC = 0; dword_7FBD4 = *(a1+16);
if (!sub_62C54(a1+44, 0, &unk_7D208, *(a1+4), 0x10000000, 0,0,0)) return 0;   // 窗口创建（sub_62D68，推测 CreateWindowExW 封装）
SHFullScreen(hwnd, 0x2A);      // 移除全屏任务栏等
GXOpenInput();
MoveWindow(hwnd, 0,0, 240,320, false);
ShowWindow(hwnd, 5 /*SW_SHOW*/);
UpdateWindow(hwnd);
return (*(vtable+20))(a1);      // 虚调用 vtable[+20]（首个"启动"回调）
```
证据：`00062A14_sub_62A14.c:6-18`；`sub_62C24` = `FindWindowW(*(a1+16)) + SetForegroundWindow(win|1)`（`00062C24_sub_62C24.c:8-12`）；`sub_62C54` 调 `sub_62F54(&dword_811A8)` 后调 `sub_62D68`（`00062C54_sub_62C54.c:11-14`，`sub_62D68` 语义 **推测** 为 CreateWindowExW 封装，参数 `(a1+44)=hinst?, ...`）。

`a1+40`（退出标志）写入方：`sub_3129C`（`0003129C_sub_3129C.c:18`，`a1[40]=1`，仅在 `a1[50] && a1[41]` 递减到 0 时触发，**推测** 为帧级计数/退栈完成检测器，被 `sub_2101C` 在每帧循环里调用（`0002101C_sub_2101C.c:4245`）。`sub_5AFEC` 也写 `a1[40]`（`0005AFEC_sub_5AFEC.c:15`，语义为 `*(a1+36)**(a1+3)>>6`，**推测** 为渲染视口/裁剪计算，与退出标志同偏移是 IDA 变量复用还是真实冲突——**推测，待验证**：需对照 sub_2101C/sub_29110 的调用上下文判断两者是否作用于同一结构体）。

## 3. 帧函数（vtable+16）解析 — **本模块核心结论**

### 3.1 静态 vtable 解析（直接读 exe `.data`，file off = RVA）

| RVA | 首 6 dword | 判定 |
|---|---|---|
| `off_6FDC8` | `0x17924, 0x179A0, 0x17A18, ...`（后续 `0x17A...` 递增，均为有效 ARM 代码地址） | **vtable**：`+0=sub_17924`（list 尾指针取 `last`，`00017924_sub_17924.c`）、`+8=sub_179A0`、`+16=sub_17A18` |
| `off_6FD60` | `0x16E0C, 0x16ED8, 0x16F20, ...`（0x16E0C 处字节 `91 1F 00 EB ...` 是合法 ARM 序言） | **vtable**：`+16 = 0x16F20` |
| `off_6FD88` | `0x17000, 0x173C4, 0x17410, ...`（0x17000 处字节 `07 10 A0 E1 ...` 合法） | **vtable**：`+16 = 0x17410` |
| `off_6FDA0` | `0x17504, 0x17550, 0x17644, ...` | **vtable**（`sub_45914` 中 `*a1 = off_6FDA0` 直接用值） |
| `off_6FD94` | `0x17410, ...`（与 off_6FD88 重叠 4 字节后错位） | 疑为 `off_6FD88+0xC` 的别名/相邻 vtable，`sub_45914` 中 `a1[11]=&off_6FD94`（`00045914_sub_45914.c:11`） |

### 3.2 `vtable+16` 的解析结果

消息泵 `sub_62B84:19` 调用 `(*(vtable+16))(a1)`。`a1` 是 app 对象，其 vtable 在 `sub_4595C`（ctor）/`sub_45A10`（dtor）中被设为 `off_6FDC8`（`0004595C_sub_4595C.c:13`、`00045A10_sub_45A10.c:8`）。

- **静态结论**：`off_6FDC8 + 16 = 0x17A18`，即 `sub_17A18`。
- **但**：`sub_17A18` 在 `ref/pseudocode` 中未直接反编译出独立 .c 文件（`functions.csv` 中 `0x17A18` 无独立行，**推测** 它是 `off_6FDC8` 这个「多态接口表」的一个 **数据槽**而非真实函数——即 `off_6FDC8` 更可能是「对象池/事件表」的静态初始表，运行时被替换成**具体场景类的 vtable**（与 §4 的场景加载逻辑吻合：`sub_43B8C` 把新 vtable `off_6F518` 写入 `*a1`，见 §4.2））。
- **推测（待验证）**：`off_6FDC8`/`off_6FD60`/`off_6FD88`/`off_6FDA0` 是 **LucencyApp / 框架基类的静态初始 vtable**（构造期），运行时 app 对象 `*a1` 被场景/窗口类 ctor（如 `sub_43B8C` 设 `off_6F518`）**覆盖**为场景对象自己的 vtable，消息泵每帧调用的「vtable+16」实际落在**当前场景类**的虚函数上，而非固定的 `sub_17A18`。验证方法：在 `sub_2101C`/`sub_29110`（§4 的场景更新函数）里找 `*(vptr+16)` 模式的虚调用点，或 QEMU-CE/frida 下断点观察运行期 `*(app+0)` 的取值随场景切换而变化。

`sub_45914`（`00045914_sub_45914.c`）进一步设置 app 对象的两个 vtable 字段：`*a1 = off_6FDA0`（`a1[0]`）、`a1[11] = &off_6FD94`（`a1[11]`）、`a1[18]=0`、`a1[19]=DefWindowProcW`。结合 `sub_455C4`（§3.3），**推测** `a1[0]=off_6FDA0` 是「窗口包装对象」的 vtable（`sub_455C4` 里 `*(_DWORD*)(a1+28)` 保存/恢复的 `v9 = *a1` 即此 vtable，`v10=*v9` 后调 `v10(a1)` = 虚调用 `+0`；失败后 `v15 = *(vtable+8)` = `+8` 虚函数清理 hwnd），而 `a1[11]=&off_6FD94` 是**事件/子对象**的二级 vtable。证据：`000455C4_sub_455C4.c:17-41`。

`WinMain` 里 `v10[7] = &off_6FD88`（`00045458_WinMain.c:21`）：**推测** 是把 `off_6FD88`（另一张 vtable，`+16=0x17410`）挂到栈对象 `v10` 的第 7 个槽位，作为 app 对象内某个「子系统/场景容器」的初始 vtable；`v8 = off_6FD60`（行22）则是 `sub_62B84` 直接操作的「app 对象」的 vtable 初值（`off_6FD60+16 = 0x16F20`）。**推测** `off_6FD60`/`off_6FD88`/`off_6FDA0`/`off_6FDC8` 是同一 LucencyApp 基类体系下 4 个不同派生类（基类/窗口类/场景容器类/事件类）的静态 vtable，`sub_4595C`（ctor）统一写入 `off_6FDC8`，`sub_45914` 再按字段覆盖 `off_6FDA0`/`off_6FD94`。

### 3.3 窗口消息转发（sub_455C4, 0x455C4）

```
sub_455C4(app, msg, wparam, lparam):
  v8 = *(a1+28);                      // 暂存当前"事件处理对象"指针
  *a1+28 = 栈局部 v17[];             // 换成局部 15-dword 结构
  v17[0..3] = hwnd/msg/wparam/lparam; v17[4..5]=0;
  v11 = (*(a1->vtable[0]))(a1);      // 虚调用 vtable[+0]，语义推测为 OnMessage/handler，返回 0 表示"未处理"
  *a1+28 = v8;
  if (v11 == 0) {
    if (msg != WM_DESTROY(2))
      return CallWindowProcW(*(a1+32), hwnd, msg, wparam, lparam);   // 回退 DefWindowProcW
    // WM_DESTROY 路径：
    ... GetWindowLongW(hwnd, -4) 保存/比较；必要时 SetWindowLongW(hwnd, -4, *(a1+32))
    v15 = *(a1->vtable[8]); v15(a1, hwnd);   // 虚调用 vtable[+8]（推测 OnDestroy/清理回调）
    *(a1+4) = 0;
  }
```
证据：`000455C4_sub_455C4.c:16-43`。`sub_45914` 设 `a1[19]=DefWindowProcW`（`00045914_sub_45914.c:8`），**推测** 即 `a1+32` 槽位（dword 偏移 19×4=0x4C=+76？——**推测，IDA 变量名与实际偏移需再核对，`a1[19]` 与 `a1+32` 是否同槽不确定**）。

## 4. 场景加载与切换

### 4.1 场景「装载器」函数族（每个对应一个 `.Map` 场景文件，均在 exe 里以宽字符串常量出现）

| 函数 | 场景字符串 | 资源容器/文件 | 证据 |
|---|---|---|---|
| `sub_42574` | `aIntroMap` = `"intro.Map"` | `data/ui.pak`（`aUiPak`）+ `intro.Map` | `00042574_sub_42574.c:31,54,58` |
| `sub_3ECDC` | `aSelectroleMap` = `"selectrole.Map"` | `ui.pak` + `selectrole.Map` | `0003ECDC_sub_3ECDC.c:36` |
| `sub_402D8` | `aSelectmapMap` = `"selectmap.Map"`；再读 `aGlmapm01Map`/`aGlmapm02Map`/`aGlmapm03Map`（`glmapm01.Map` 等，`CreateFileW` 直读 `data/*.Map` 判断存在性）| `ui.pak` + `selectmap.Map` + 3 个地形 `.Map` 文件直读探测 | `000402D8_sub_402D8.c:48,72,89,103` |
| `sub_40D80` | `aIndexMap` = `"index.Map"` | `ui.pak` + `index.Map` + `aGlpicsPak`（`glpics.pak`）+ `aDataGlRec`（`data/gl.rec` 魔数校验 `0x01315A47`） | `00040D80_sub_40D80.c:41,64,71-84` |
| `sub_2077C` | `aDataMainMap` = `"main.Map"`（主地图/探索场景） | — | `0002077C_sub_2077C.c:175` |

共同模式（以 `sub_40D80` 为例，`00040D80_sub_40D80.c:26-67`）：
```
sub_40D80(ctx, pUi, pMap):
  ctx[1]=pUi; ctx[2]=pMap;
  宽字符串包 ui.pak 名 → sub_37EDC(buf, ui, "ui.pak")      // 从 ui.pak 索引取 PCX/控件表
  宽字符串包 "index.Map" 名 → sub_37EDC(buf2, ui, "index.Map")
  scene = new 0x3FC;
  sub_43B8C(scene, pMap);                       // 场景对象 ctor，设 *scene = off_6F518（§4.2）
  sub_43D54(scene, *(pUi+16));                 // 关联 UI 句柄
  (*(scene->vtable[+4]))(scene, ...);          // 虚调用 +4：装载资源/场景记录
  进一步按场景类型加载额外 pak（index: glpics.pak；selectmap: 逐个探测 glmapm0x.Map 存在性并 sub_406DC 写入 ctx+13/16/19 三个槽位 = 已解锁地图清单）
```

### 4.2 场景对象 vtable：`sub_43B8C`（0x43B8C）

```
sub_43B8C(scene_mem, pMap):
  sub_44978(scene_mem+244, &flag);
  scene_mem[1] = pMap;
  *scene_mem = &off_6F518;          // ← 场景对象 vtable = off_6F518
  scene_mem[253]=1; [123..254] = 0; 返回 scene_mem
```
证据：`00043B8C_sub_43B8C.c:8-22`。`off_6F518` 在 exe 中实际字节是 ASCII 文本 `"reverse circuit deman"`（`0x656572...`，**非** vtable 指针）——说明 `*a1 = &off_6F518` 在此是「取地址」存入字符串常量，**推测** 是 IDA 反编译歧义：`off_6F518` 实际应是被 `operator new` 分配的某类对象的 vtable 首址，但反汇编把 `&off_6F518` 误识别成 ASCII 段起始；**待验证**：需对照 `sub_44978`/`off_6F518` 所在 `.data` 真实布局（可能 `&off_6F518` 实际落在 `.text` 的另一处，`0x6F518` 只是 IDA 命名巧合）。

场景对象虚函数调用点（`sub_29110` 内，见 §4.3）：
- `(**(scene_ptr) + 4)(scene, ...)` — 装载（`sub_43B8C` 之后紧跟调用）
- `(**(scene_ptr) + 8)(scene)` — 销毁/卸载（每个 case 收尾，如 `00029110_sub_29110.c:823`）
- `(**(scene_ptr) + 16)(scene, a2, a3, a4)` — **帧更新/事件处理**（`00029110_sub_29110.c:766`、`2624`；`0002101C_sub_2101C.c` 内大量 `(**(vptr)+16)` 调用，**推测** 即场景的 `Update/OnFrame` 虚函数，与 §3 消息泵里 `a1->vtable+16` 语义对应——即消息泵的「vtable+16 帧函数」实际转发到**当前活跃场景对象**的 `+16` 虚函数）

### 4.3 场景切换分派：`switch (*(_DWORD*)(a1+716))`

`sub_29110`（0x29110，size 22844，**推测** 为「场景管理器/帧更新总控」）里有两处主 switch（`00029110_sub_29110.c:772` 与 `2628`），均对 `*(a1+716)` 分支，**推测** 是「当前场景类型码 / 场景状态机 phase」：

**第一处（`a1+576` 非空路径，行 762-859）**，`switch (*(a1+716))`：
- `case 0`：若 `MEMORY[0x10008115C]+1020 != 0` → 直接跳 `LABEL_210`；否则 `new 0x474` → `sub_572E4` 构造「**vs**」（战斗/对战场景对象，加载 `aDataUiPak` + `aDataVsMap` = `vs.Map`），装载后 `*(a1+716)=25`，销毁旧 `a1+576`（`+8` 虚函数 + `vtable[0]` 虚调用传 1），把新对象挂回 `a1+576`
- `case 3`：`v123->vtable[8]`（销毁），`v123->vtable[0]` 传 1；`*(a1+716)=0`；`*(a1+576)=0` —— **推测** 退出 vs 场景回主菜单
- `case 4`：读 `MEMORY[0x100081154]/[0x100081170]`（当前玩家/目标位置？），`sub_30998/sub_30978`（**推测** 设置相机/坐标），销毁旧对象；`*(a1+720)=24`；`*(a1+716)=27`

**第二处（行 2628，`a1+588` 非空路径）**，`switch (*(a1+716))`：
- `case 2`：`sub_17C7C(*(a1+4), 0)`（**推测** 播放音效停止）；`*(a1+588)->vtable[+12]`（**推测** 场景卸载虚函数）
- `case 4`：`sub_34A08(*(a1+588))==0` 判定（**推测** 场景完成态检查）
- （其余 case 25/26/27 集中在 `00029110_sub_29110.c:2585-2619`，`case 26`/`case 27` 走 `sub_51D88`/`sub_3402C` 初始化动画/过场，`*(a1+716)` 与 `*(a1+720)` 联动设为 27/24/120/48/0，**推测** 为过场动画 phase 机）

`sub_2101C`（0x2101C，size 31344，**推测** 为**主地图探索/贸易场景的帧更新**）里也有 3 处 `switch`（行 1356 对局部 `v512`、行 1709 对 `sub_31424(*v285)`、行 2561 对 `sub_31424(v43+72)`），均以 `sub_31424`（事件对象取 id，**推测** = 事件枚举/事件 id）做分支，`case 1..8` 各对应不同游戏事件（探索/贸易/事件触发等），并伴随 `sub_2F45C(buf, aDataMsglMap/aDataCardMap/aDataBigcardMap/...)`（场景字符串装载）——**推测** 是「事件 → 场景字符串」的映射分派。

**场景切换图（按资源字符串 + 状态码整理，`推测` 均指「该 switch 分支的触发条件/前置场景」）**：

```
                 WinMain → sub_62B84(消息泵) → 每帧 vtable[+16] →
                 [当前场景对象的 +16 虚函数]
                            |
            ┌───────────────┴────────────────┐
            |  sub_29110（场景管理器帧更新）    |
            |  分支点：*(a1+716) 状态码        |
            ├─ 0/3/4 → 加载 vs.Map（sub_572E4）[case 0]
            ├─ 2/4   → 卸载/完成当前场景        [case 2/4]
            └─ 25/26/27 → 过场动画 phase 机    [case 25/26/27]
            |
            |  场景装载器（每个独立函数，new 0x3FC + sub_43B8C）：
            ├─ sub_42574 → intro.Map + ui.pak
            ├─ sub_3ECDC → selectrole.Map + ui.pak
            ├─ sub_402D8 → selectmap.Map + ui.pak + 探测 glmapm01/02/03.Map 存在性（三槽位 sub_406DC）
            ├─ sub_40D80 → index.Map + ui.pak + glpics.pak + data/gl.rec（魔数校验）
            └─ sub_2077C → main.Map（主地图探索/贸易，内含 3 处 switch(sub_31424(...)) 事件分派）
```

## 5. 全局单例 off_811A0（sub_3D430 / sub_3D49C）

`sub_3D430`（0x3D430，ctor）：清零 8 个 u32 槽（`+4..+28`），`+0 = -1`，`+32/+33 = 1`，`sub_1333C(+36)`（宽字符串构造），`+48..+60 = 0`，`off_811A0 = a1`。证据：`0003D430_sub_3D430.c:6-22`。
`sub_3D49C`（0x3D49C，dtor）：`off_811A0 = nullptr`；`sub_2F4F4(+48)`（**推测** = 某子对象析构/free）；`sub_12F48(+36)`（宽字符串析构，`00012F48_sub_12F48.c` = free 封装）。证据：`0003D49C_sub_3D49C.c:6-8`。
**推测**：off_811A0 = 「全局游戏状态/配置单例」（`-1` 初值 + 多个 u32 标志位 + 宽字符串名），被 `sub_4554C`（`0004554C_sub_4554C.c:8`）与 `sub_45C40`/`sub_45C68`（app 对象析构链）引用，**推测** 是「存档/设置」承载体（对应 S3.10 存读档模块）。

## 6. 函数归属表（main_loop 模块）

判定标准：(a) 直接被 `start`/`WinMain`/`sub_62A14`/`sub_62B84` 调用链触及；(b) 参与 vtable 设置/窗口消息转发/帧函数虚调用；(c) 场景装载器（new 场景对象 + 设 `off_6F518` vtable + 调 `+4/+8/+16` 虚函数）；(d) 全局单例 ctor/dtor。完整归属见 `ref/functions_main_loop.csv`（按用户要求，不改动 `ref/functions.csv` 原文件，另建独立 CSV）。

| 函数 | 地址 | 归属理由 |
|---|---|---|
| `start` | 0x69FB0 | 入口，`cinit()+WinMain+exit` |
| `WinMain` | 0x45458 | 顶层流程（§1） |
| `sub_456F8` | 0x456F8 | 框架基类 ctor（3×CritSection + 类链表遍历） |
| `sub_45914` | 0x45914 | app 对象 ctor 封装，设 vtable 字段 `off_6FDA0`/`off_6FD94`/`DefWindowProcW` |
| `sub_4595C` | 0x4595C | app 对象 ctor 主体，`*a1 = off_6FDC8` |
| `sub_45A10` | 0x45A10 | app 对象 dtor 主体，`*a1 = off_6FDC8`（复位） |
| `sub_45C68` | 0x45C68 | app 对象 dtor thunk → `sub_45A10` |
| `sub_45C40` | 0x45C40 | app 对象析构 + 可选 `operator delete`（`a2&1` 判是否堆分配） |
| `sub_4554C` | 0x4554C | 组合对象析构（先 `sub_3D49C(+80)` 单例，再 `sub_45C68`），推测为「框架+app」复合 dtor |
| `sub_455C4` | 0x455C4 | 窗口消息 WndProc（虚调用 `+0` handler，失败回退 `CallWindowProcW`/`DefWindowProcW`，`+8` OnDestroy） |
| `sub_457B0` | 0x457B0 | 框架基类 dtor（UnregisterClass/DeleteCritSection/HeapDestroy） |
| `sub_45580` | 0x45580 | `sub_457B0` 内部 free 辅助（`+96` 槽位） |
| `sub_62A14` | 0x62A14 | 窗口/全屏/GXOpenInput 初始化 + `vtable[+20]` 启动回调 |
| `sub_62C24` | 0x62C24 | `FindWindowW`+`SetForegroundWindow`（判断是否已运行实例） |
| `sub_62C54` | 0x62C54 | 窗口创建封装（调 `sub_62D68`，**推测** CreateWindowExW） |
| `sub_62B84` | 0x62B84 | 主消息泵（PeekMessage 循环 + `a1[40]` 退出标志 + `vtable[+16]` 帧调用） |
| `sub_3D430` | 0x3D430 | 全局单例 ctor（off_811A0） |
| `sub_3D49C` | 0x3D49C | 全局单例 dtor |
| `sub_43B8C` | 0x43B8C | 场景对象 ctor（`*a1 = off_6F518`） |
| `sub_43D54` | 0x43D54 | 场景对象关联 UI 句柄（每个场景装载器紧随 `sub_43B8C` 调用） |
| `sub_37EDC` | 0x37EDC | 场景资源「按字符串名取 pak 记录」通用装载（所有场景装载器共用） |
| `sub_42574` | 0x42574 | intro.Map 场景装载 |
| `sub_3ECDC` | 0x3ECDC | selectrole.Map 场景装载 |
| `sub_402D8` | 0x402D8 | selectmap.Map + glmapm0x 存在性探测装载 |
| `sub_40D80` | 0x40D80 | index.Map + glpics.pak + gl.rec 魔数校验装载 |
| `sub_2077C` | 0x2077C | main.Map（主地图）装载 |
| `sub_29110` | 0x29110 | 场景管理器帧更新/切换总控（两处 `switch(*(a1+716))`） |
| `sub_2101C` | 0x2101C | 主地图探索/贸易帧更新（3 处 `switch(sub_31424(...))` 事件分派，内含 `sub_3129C` 帧计数→`a1[40]` 退出标志） |
| `sub_3129C` | 0x3129C | 帧/退栈计数检测，置 `a1[40]=1`（消息泵退出标志） |
| `sub_12F04` | 0x12F04 | 宽字符串 `strlen`（所有场景装载器前置） |
| `sub_1318C` | 0x1318C | 宽字符串包构造/拷贝（WinMain、sub_4595C 用） |
| `sub_12F48` | 0x12F48 | 宽字符串析构（free 封装，sub_45A10/sub_3D49C 用） |

**未纳入本模块、但与之强耦合**（留待 S3.2~S3.5 细分）：各场景具体 gameplay 逻辑（sub_48654/sub_49120/sub_49B94 等子函数，被 `sub_2101C` 大量调用）、`sub_31424`（事件 id 解析，S3.9 卡牌/事件模块）、`sub_572E4`（vs 战斗场景构造，S3.8）、`sub_30FDC`（存读档 IO，S3.10）。

## e) 缺口与推测汇总（待验证）

1. `off_6FDC8` 的 `+16` 槽（`0x17A18`）是**静态初始 vtable** 还是**运行时被场景类 vtable 覆盖**——目前只有「`sub_43B8C` 把 `*scene = off_6F518`」一处直接证据支持「运行时替换」，但 `off_6F518` 字节内容异常（ASCII 文本），**推测** `sub_43B8C` 反编译存在 vtable 地址误识别，需 hexdump `0x6F518` 附近真实内存 + 对照 `sub_44978` 确认。
2. `sub_45914` 中 `a1[19]=DefWindowProcW` 与 `sub_455C4` 中 `*(a1+32)`（WndProc 槽位）是否同一槽位（偏移 19×4=0x4C vs +32）——**推测** IDA 不同变量名指向同一物理偏移，需 ARM 反汇编核对 `sub_45914` 实际写入的偏移。
3. `a1[40]` 同时被 `sub_3129C`（退出标志）与 `sub_5AFEC`（渲染视口计算，`0005AFEC_sub_5AFEC.c:15`）写入——**推测** 是不同结构体复用同一偏移量（不同对象实例），非真实冲突，但需 QEMU-CE 断点确认。
4. `sub_62C54`/`sub_62D68` 是否为 CreateWindowExW 封装（参数形态吻合但不直接调用 `CreateWindowExW`，只调了 `sub_62F54`+`sub_62D68`）——**推测**，需 IDA 里对 `sub_62D68` 跟一手调用栈。
5. `switch (*(a1+716))` 的状态码（0/2/3/4/25/26/27）与「场景类型」的一一对应——目前只有 case 0（加载 vs.Map）与 case 3（回主菜单）有较明确的资源字符串佐证，其余（2/4/25/26/27）语义**推测** 为过场/卸载 phase，需配合 S3.8（战斗）与 S3.11（消息/过场）模块细化。

## 已知问题清单

1. **P3-2 场景图未验证**：本模块 DoD 为「场景切换图与 ref 一致 + diff 测试」，但上表 §7 第 5 条的状态码语义未收敛，P3-2（`coverage scenes → missing = 0`）目前没有机器可判定的实现。
2. `define_scene!` 未落地（`docs/style.md` §5.4 规划中）：20 个 UI 场景的 vtable/事件分发表仍是手写样板，S3.1 之后应收敛为宏注册，否则每加一个场景都要复制粘贴。
3. 全局单例 `off_811A0` 的语义**推测**为存档/设置承载体，与 S3.10（[`save_load.md`](save_load.md)）交叉引用；两侧结论需在 S3.10 落地时合并复核。
