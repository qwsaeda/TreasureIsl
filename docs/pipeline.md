# TEST-AUDIT-FIX 流水线

多代理协作的测试-审计-修复循环。目标：批量发现 → 独立分析 → 复核 → 修复 → 再测，直到全绿。

---

## 1. 角色定义

| 角色 | 职责 | 输入 | 输出 | 加载文档 |
|---|---|---|---|---|
| **Tester** | 批量运行所有测试，收集失败 | 代码 + 测试文件 | 失败清单（测试名 + 错误 + 堆栈） | `experience/build.md`、`experience/code.md` |
| **Auditor** | 根因分析 + 修复方案 | 失败清单 + 源码 | 按优先级排序的修复方案 | `experience/reverse.md`、`experience/tools.md`、`facts/index.md` |
| **Auditor-2** | 独立复核 Auditor-1 | Auditor-1 方案 + 源码 | 确认/补充/反驳 | 同上 + `experience/process.md` |
| **Fixer** | 按复核方案修复代码 | 复核后的方案 | 代码修复 + 测试通过 | `style.md`、`experience/code.md` |

---

## 2. 流水线流程

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────┐
│ Tester   │───→│ Auditor  │───→│ Auditor  │───→│ Fixer│
│ 批量测试  │    │ 根因分析  │    │ 独立复核  │    │ 修复  │
└──────────┘    └──────────┘    └──────────┘    └──────┘
      ↑                                                │
      └────────────────────────────────────────────────┘
                    Loop until all green
```

### 2.1 Tester

```bash
cargo test --workspace 2>&1
```

输出：失败测试列表 + 每个失败的错误信息 + 堆栈跟踪。

### 2.2 Auditor-1

对每个失败：
1. 读测试代码，理解测试意图
2. 读被测源码，定位根因
3. 给出修复方案（具体到文件+行号+改法）
4. 按影响范围排序（P0 阻塞 > P1 功能 > P2 边界）

### 2.3 Auditor-2（独立复核）

1. 自己读源码，不信任 Auditor-1 的结论
2. 验证每个根因是否正确
3. 检查是否有遗漏的关联问题
4. 输出：确认 / 补充 / 反驳

### 2.4 Fixer

1. 按复核方案修复
2. 运行相关测试验证
3. 如果修复引入新问题，回退并报告

---

## 3. 终止条件

1. `cargo test --workspace` 全绿
2. Auditor-2 确认无遗漏
3. `scripts/gate.ps1` 全绿

---

## 4. 经验积累

每轮流水线结束后：
- 新发现的坑写入 `docs/experience/index.md`
- 新发现的事实写入 `docs/facts/index.md`
- 流水线本身的问题写入本文件

---

## 5. 当前状态

- 26 单测 + 5 音频测试全绿
- 4 个 e2e 测试 2 失败（`e2e_invalid_inputs_no_crash`、`e2e_full_game_session`）
- 待流水线修复
