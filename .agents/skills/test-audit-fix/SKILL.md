# TEST-AUDIT-FIX 流水线 Skill

## 触发条件
- 用户要求跑流水线 / 修复测试失败 / 全量回归
- `cargo test --workspace` 有失败
- 新代码合入后验证

## 流水线步骤

### Step 1: Tester — 批量测试
```bash
cargo test --workspace 2>&1
```
收集所有失败的测试名 + 错误信息 + 堆栈。

### Step 2: Auditor-1 — 根因分析
对每个失败：
1. 读测试代码，理解测试意图
2. 读被测源码，定位根因
3. 给出修复方案（文件+行号+改法）
4. 按 P0/P1/P2 排序

### Step 3: Auditor-2 — 独立复核
1. 自己读源码，不信任 Auditor-1
2. 验证根因 + 检查遗漏
3. 输出：确认/补充/反驳

### Step 4: Fixer — 修复
1. 按复核方案修复
2. 运行相关测试验证
3. 引入新问题则回退

### Step 5: 再测试
回到 Step 1，直到全绿。

## 终止条件
- `cargo test --workspace` 全绿
- Auditor-2 确认无遗漏
- `scripts/gate.ps1` 全绿

## 经验文档
- `docs/experience/index.md` — 踩坑记录
- `docs/facts/index.md` — 逆向事实
- `docs/pipeline.md` — 流水线定义
