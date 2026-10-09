# 逆向事实库索引（Facts Index）

**收录规则**：只写有脚本/工具证据的结论，标注 `证据`；推测必须写 `推测` 并说明待验证方法。
新增/推翻时更新对应分卷并在 `docs/experience/index.md` 记一笔（不要静默改结论）。

- `F-NNN` **全局唯一、绝不重号**。门禁第 9 项 `doccheck` 校验 ID 唯一性 + 跨文档引用完整性。
- 与经验库的分工：**事实库 = 结论**（带证据），**经验库 = 过程与教训**（`docs/experience/index.md`）。

## 分卷

| 卷 | 内容 | F-ID |
|---|---|---|
| [`exe.md`](exe.md) | 工具环境实测 + PE 头/节表/导入/版本 | F-101 ~ F-108 |
| [`pak.md`](pak.md) | 容器 `LucencyPackage`：头/索引/key/重打包 | F-201 ~ F-210, F-305 |
| [`map.md`](map.md) | `.Map` 两族布局、控件字段语义、事件 id 回填与分派 | F-301/302/306/307/308/311/315/316/317/318 |
| [`font.md`](font.md) | `GoldLand.Fnt` / `font.dat` 布局与位序 | F-303 |
| [`save.md`](save.md) | `settings.dat` / `gl*.sav` / `gl.rec` | F-304 |
| [`tools.md`](tools.md) | IDA / REA MCP 可用性 / RNG 来源 | F-313/314/319/320/321 |

## 已知缺口（按计划补齐）

> 只列**当前仍未闭合**的缺口。已闭合的条目请从本表删除，其结论留在各分卷的事实条目里。

- `.Map` 记录 `f5` 控件类型非 0 档（100/200/300…1400 共 44 种）逐档含义与交互行为 —— 属 S3.3+ 场景逻辑（F-311 注明）。
- 事件描述符对象 `+22` 字节的**回填时机与逐 id 确认** —— 属 S3.3+（F-315/F-317）。
- 地形 `glmapm0*.Map` trailer 的 `f6/f7/f8` 与连线记录 `M` 的字段语义 —— 已验证的恒等式见 F-307，剩余字段属推测（S3.5）。
- `Glyph::width()` 步进算法 —— 无字宽表，按右缘墨迹+1 推断（**推测**，见 F-303）。
- coredll 内 `rand()` 的实际算法 —— exe 内只有 thunk，无法实测（F-320，标 `推测`）。
- REA 波动性可用（F-319）：冒烟优先，失败即退主路径；非永久不可用（S3.2 可用 EXP-039 / S3.12 失败 EXP-036）。
- REA 结构化能力（CFG/调用图/类型恢复/值追踪/重建验证）未利用 —— 见 F-321，待 S3.12/S4 探针。
- **没有可运行原版的环境**：WinMo 模拟器仅 x86 镜像，本程序是 ARM → 视觉对照改用「资源解码独立参考实现逐像素比对」，可选后续 QEMU-CE（未验证可行性）。

**已闭合（原列于此，现已补齐，勿再当缺口）**：`name→key` 哈希算法（S1.1 → F-204/F-209/F-210）、`.Map` 记录字段语义（S1.4/S1.8 → F-306/F-307/F-308/F-311）、`.Fnt`/`font.dat` 全布局（S1.5/S1.5b → F-303）、coredll 序号→API 名（S0.4 → `ref/imports.json` 116 个）。

## 逆向工作流（固定，勿改）

```
1) IDA headless 导出："D:\soft\IDA Professional 9.4\idat.exe" -A -Lref\ida_run.log -Sscripts\ida_dump.py TreasureIsl.exe
   产出 ref/pseudocode/*.c + ref/functions.csv
2) 分类打标：cargo run -p ti-tools -- classify   → ref/classify.json
3) 结论回写 docs/facts/<分卷>.md（带证据），再更新 WBS/台账

4) 控件记录导出：cargo run -p ti-tools --bin controls -- --out ref   → ref/controls.json（S1.4b）
```