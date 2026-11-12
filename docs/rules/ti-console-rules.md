---
paths:
  - "crates/ti-console/src/**"
  - "crates/ti-devmeta/src/**"
---
# ti-console 平台红线（飞轮 R10 起审计基准面）

> 平台 = TI Dev Console（`crates/ti-console` + `crates/ti-devmeta`，纯管理/辅助开发，与游戏运行时解耦；
> 单一事实源 = `docs/process/22_dev_platform.md` 定位/边界/§4.2 白名单/§10 执行面）。
> 本红线 = 平台 doc↔code↔data 一致性审计基准面（飞轮 R3~R9 未覆盖平台面，R10 起纳入；
> CCGS `rules/*.md` `paths:` 模式，S50.3 同源 + 飞轮 R10 平台面扩展）。

## 1. 写面白名单（repo/ 写点纪律）
- 平台写面 = **白名单动作 + 枚举/id 参数**，禁任意 shell（22 号 §10.3 + 30 号 §2）。
- 文档声称的每个写端点（`POST /api/...`）须在 `crates/ti-console/src/routes/` 实存对应 handler；
  反向 `routes/` 里每个 POST 写端点须有 `X-TI-Token` 鉴权 + `enable_write` 门禁（22 号 §4.2 写面默认关）。
- 写点探针 = `repo/guard.rs`（内容级快照判定），新增写面须登记 30 号 §3.1 写点表。

## 2. 端点 ↔ 文档对账
- 22/30/31 号声称的每个 `GET/POST /api/<name>` 须在 `routes.rs`/`routes/*.rs` 实存（幻影端点 = ERROR）。
- 端点参数契约（body 字段 / 白名单 regex / 鉴权次序 403→400→413→400）文档↔代码同义。

## 3. 程序白名单（runner/ spawn 面）
- `runner/` 后台 spawn 的程序 = test_manifest `command` 白名单 `cargo/python3/pwsh/powershell`，
  逐参数 `Command::new` 非 shell 拼接（31 号 §2/§3 + 22 号 §10.3）。
- 文档声称的程序白名单 ↔ `runner/jobs.rs`/`spawn.rs` 实存白名单 1:1。

## 4. 双审计协议（audit/ 面）
- `audit/judge.rs` 收敛判定（§7.3/§7.4 四行机器字段 task/round/侧/新发现/误报/残留）↔
  `GET /api/audit` 收敛表口径；`audit/taskbook.rs` 八要素模板机器断言（`uncovered_elements` 空 = 全覆盖）。
- 双审计 2 轮 0/0/0 停机判据（22 号 §7.4）= CCGS 口径（docs/process/06_flywheel.md §4 同源）。

## 5. 资产/地图数据面
- 资产直读 = `assets/ui/` 白名单前缀（防路径穿越）；地图包 = `assets/maps/<id>/` I1~I9 不变量
  （权威实现 = `scripts/map_pack.py`，平台编辑器 `repo/import.rs` 前端校验条须镜像其 `verify_pack`）。
- AI 生成面（`POST /api/gen-island`/`gen-plot`）= map_id `^[a-z0-9_]{1,16}$` 防穿越 + size/ratio 白名单
  + `AGNES_API_KEY` 缺失 = 400 如实上报（22 号 设计图/地图页段）。

## 6. 与游戏 crate 解耦
- 平台改动 0 游戏 crate diff（ti-core/ti-app/ti-engine/ti-render/ti-platform/ti-formats 全 0 diff）；
  平台 = ti-console + ti-devmeta 专属（`dev` 依赖，不进 Bevy 渲染壳/游戏逻辑）。
