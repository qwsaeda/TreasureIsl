---
name: exp-record
description: Use after fixing a bug, doc drift, or gate failure, to record the lesson into docs/experience/ so the next session doesn't repeat it. Triggers: "回写经验", "记一条 EXP", "踩坑", "教训", "别下次再犯", "EXP-xxx".
---

# 踩坑回写 experience/

每次修完一个非平凡问题（bug / 文档漂移 / gate 失败 / 架构决策），必须回写 `docs/experience/`，否则下一轮会话重复踩坑。

## 流程

1. **选卷**：`docs/experience/index.md` 的 6 卷归属
   - `formats.md` 二进制/资源格式
   - `reverse.md` 逆向方法论
   - `tools.md` 本机工具链/IDA/Python/REA
   - `build.md` 构建/三端产物/wasm 壳/浏览器自动化
   - `process.md` 流程与协作（验收纪律、并行会话、文档治理、oracle 规约）
   - `code.md` 代码级教训（文档与实现不一致、类型转换陷阱、UTF-8 编辑陷阱、门禁扫描边界、clippy pedantic、固化脚本契约、const 表 i18n、oracle 边界脚本）

2. **取新编号**：`grep -o "EXP-[0-9]*" docs/experience/*.md | sort -u | tail -5`，取当前最大编号 +1。**绝不重号**（重号会被 `doccheck` 第 1 项拦，历史上 EXP-048 就重过号，见 `index.md` 修正记录）。

3. **写正文**：在对应卷文件末尾追加一段，格式：
   ```markdown
   ### EXP-0XX（卷名）
   - **现象**：一句话说清踩了什么坑。
   - **根因**：为什么会踩（不是怎么修，是为什么之前没发现）。
   - **解法**：怎么修/怎么避免（可执行动作）。
   - **验证**：用什么命令/测试证明没再犯（能机器判定最好）。
   ```

4. **登记 index.md**：在 `docs/experience/index.md` 条目表里加一行 `| EXP-0XX | 卷名 | 一句话标题 |`，同时更新该卷的条数统计列。

5. **门禁校验**：
   ```powershell
   cargo run -q -p ti-tools --bin doccheck   # 第 9 项，EXP 唯一性 + 跨文档引用
   powershell -ExecutionPolicy Bypass -File scripts/gate.ps1
   ```
   `doccheck` 报「experience `EXP-0XX` defined 2 times」= 重号，回查步骤 2。

## 什么时候必须写

- 修了一个 `gate.ps1` 没拦住、靠人肉发现的 bug（尤其是文档↔代码漂移类）→ 考虑顺手补进 `doccheck2` 变成可重跑断言（见 `doc-check` skill），再在 `build.md`/`code.md` 记一条「已固化」标注。
- 并行会话半成品导致 gate FAIL、做了归因（见 `gate-green` skill）→ 记一条 `process.md` 教训。
- 发现某个脚本/工具行为与文档描述不符 → 记 `tools.md` 或 `build.md`。

## 硬约束

- 不许只写现象不写根因（「下次注意」不是根因）。
- 验证命令必须真实跑过，不是推测（`legacy/docs/facts/` 硬规则 4 同样适用于经验条目的「验证」字段）。
- 一条经验只写一处，其他文档用 ID 引用（`EXP-0XX`）不重复叙述（AGENTS.md 硬规则 6）。
