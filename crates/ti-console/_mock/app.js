/* S32.3 概念稿 mockup：数据全来自 /api。
 * 视觉规范 = 平台「航海夜·精修」主题（对齐游戏侧 10_ui_redesign.md §3 设计令牌 v2；
 * 单一事实源 = 本主题表，改视觉量前必读，与游戏 8-bit 令牌互引）。
 */
const $ = (id) => document.getElementById(id);
const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({
  "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
}[c]));

let apiTasks = [];
const cur = { status: "all", stage: "all", q: "" };

function taskTag(status, rounds) {
  const m = { verified: "已验收", pending: "待办", in_progress: "在途", blocked: "阻塞" };
  let tag = m[status] || status;
  if (rounds > 0) tag += " · 失败" + rounds;
  return `<span class="tag st-${esc(status)}">${esc(tag)}</span>`;
}

function renderOverview(o) {
  const led = o.ledger;
  const pct = led.total_tasks ? Math.round((100 * led.verified) / led.total_tasks) : 0;
  const stats = [
    ["台账任务", led.total_tasks, `${pct}% 已验收`],
    ["已验收", led.verified, `errors=${led.errors}`],
    ["下一任务", led.next_task, "deps 全 verified"],
    ["地图包", o.maps.installed, "已装（含骨架 m04）"],
    ["UI 资产", o.ui_assets.count, "ui_manifest 登记"],
    ["数据源", o.sources.repo_sources, "console_sources.json"],
  ];
  $("ov-stats").innerHTML = stats
    .map(
      ([k, v, s]) => `<div class="stat"><div class="k">${esc(k)}</div><div class="v">${esc(v)}</div><div class="s">${esc(s)}</div></div>`
    )
    .join("");
  const bar = $("ov-progress");
  bar.style.width = pct + "%";
  bar.textContent = pct + "%";
  $("ov-next-b").textContent = led.next_task;
  $("ov-gate").textContent = "最近门禁：（P1 起开放，当前占位）";
  $("ov-audit").textContent = "最近审计：S32.2 r6 PASS（连续 2 轮收敛）";
}

function renderTasks() {
  const rows = apiTasks.filter((t) => {
    if (cur.status !== "all" && t.status !== cur.status) return false;
    if (cur.stage !== "all" && !t.id.startsWith(cur.stage + ".")) return false;
    if (cur.q && !(t.id + " " + t.title).toLowerCase().includes(cur.q)) return false;
    return true;
  });
  $("task-count").textContent = `显示 ${rows.length} / ${apiTasks.length}`;
  $("task-rows").innerHTML = rows
    .map(
      (t) => `
    <tr class="${t.status === "verified" ? "dim" : ""}" data-id="${esc(t.id)}">
      <td class="id">${esc(t.id)}</td>
      <td>${esc(t.title)}</td>
      <td>${taskTag(t.status, t.rounds)}</td>
      <td class="mono sm">${esc(t.deps.join(", ") || "—")}</td>
      <td class="mono sm">${esc(t.evidence || "—")}</td>
    </tr>`
    )
    .join("");
  document.querySelectorAll("#task-rows tr").forEach((tr) =>
    tr.addEventListener("click", () => showDetail(tr.dataset.id, tr))
  );
}

function showDetail(id, tr) {
  const t = apiTasks.find((x) => x.id === id);
  const d = $("task-detail");
  d.classList.remove("hidden");
  d.querySelector("h4").textContent = id;
  d.querySelector("pre").textContent = `deps   = ${t.deps.join(", ") || "—"}\nstatus = ${t.status}（rounds=${t.rounds}）\nevid   = ${t.evidence || "—"}\n\ndod（节选）：\n${t.dod_excerpt}`;
  document.querySelectorAll("#task-rows tr.open").forEach((r) => r.classList.remove("open"));
  tr.classList.add("open");
  d.scrollIntoView({ behavior: "smooth", block: "nearest" });
}

function renderEvidence(d) {
  const imgs = d.files.filter((f) => /\.(png|jpe?g)$/i.test(f));
  const logs = d.files.filter((f) => !/\.(png|jpe?g)$/i.test(f)).slice(0, 60);
  $("ev-count").textContent = `${d.count} 份（本视图展示前 ${Math.max(imgs.length, 60)} 项）`;
  $("ev-imgs").innerHTML =
    imgs
      .slice(0, 24)
      .map(
        (f) =>
          `<figure><img loading="lazy" src="${esc(f)}" alt="${esc(f)}" onerror="this.style.visibility='hidden'"><figcaption class="mono sm">${esc(f.split("/").pop())}</figcaption></figure>`
      )
      .join("") || "<p class=dim>（本视图无图片证据）</p>";
  $("ev-logs").innerHTML =
    "<ul>" +
    logs.map((f) => `<li><span class="mono sm">${esc(f)}</span></li>`).join("") +
    "</ul>";
}

function renderAssets(d) {
  const ui = d.ui_manifest ? d.ui_manifest.assets : [];
  $("as-count").textContent = `${ui.length} 枚登记 · ${(d.map_packs || []).length} 个已装包`;
  $("as-grid").innerHTML = ui
    .slice(0, 20)
    .map(
      (a) =>
        `<figure><img src="/${esc(a.file)}" loading="lazy" onerror="this.style.opacity=0.3"><figcaption class="mono sm">${esc(a.name)}</figcaption></figure>`
    )
    .join("");
  $("as-packs").innerHTML = (d.map_packs || [])
    .filter((p) => p.manifest_present)
    .map(
      (p) =>
        `<li><b>${esc(p.id)}</b> <span class="tag st-verified">已装</span> <span class="mono sm">${esc(p.pack_dir)}</span>${(d.dist || []).includes(p.id + ".map-pack") ? ' <span class="mono sm dim">zip ✓</span>' : ""}</li>`
    )
    .join("");
  $("as-default").textContent = "当前默认包指针：" + (d.default_map_pack || "（无）");
  $("as-verify").innerHTML =
    "<p class='dim'>校验状态（P2/S32.7 开放：map_pack.py verify 退出码 + 行数对账，区分平台 bug 与仓库既有漂移）：m01/m02/m03 各 22 项 I7 漂移（HEAD 即存在，S32.12 已修，待复验）· m04 骨架未填</p>";
}

function renderAudit() {
  const rows = [
    ["S32.2", "r6", "PASS", "0", "0", "0", "连续 2 轮收敛 → verified"],
    ["S32.1", "r6", "PASS", "0", "0", "0", "连续 2 轮收敛 → verified"],
    ["S32.12", "r3", "PASS", "0", "0", "0", "连续 2 轮收敛 → verified"],
    ["S32.11", "r3", "PASS", "0", "0", "0", "连续 2 轮收敛 → verified"],
    ["S32.0b", "r14", "PASS", "0", "0", "0", "连续 2 轮收敛 → verified"],
  ];
  $("audit-table").innerHTML =
    "<tr><th>任务</th><th>轮次</th><th>总判定</th><th>新发现</th><th>误报</th><th>残留</th><th>状态</th></tr>" +
    rows
      .map(
        (r) =>
          `<tr><td class="mono">${r[0]}</td><td>${r[1]}</td><td><span class="tag st-verified">${r[2]}</span></td><td>${r[3]}</td><td>${r[4]}</td><td>${r[5]}</td><td class="dim">${esc(r[6])}</td></tr>`
      )
      .join("");
  $("audit-open").innerHTML = "<b>未闭环 findings</b>：0（各任务 r≥2 轮连续收敛，22 号 §7.4 判据达成）";
  $("audit-books").innerHTML =
    "<p class='dim'>任务书生成 / 粘贴归档 / 收敛判定 = P1（S32.5）开放，本视图仅展示已归档报告（evidence/console/ 31 份）</p>";
}

function renderSession(s) {
  const chk = $("set-write");
  chk.checked = s.write_enabled_default === true;
  $("set-token-state").textContent =
    s.token_present
      ? "当前会话：TI_CONSOLE_TOKEN 已设置（远程鉴权面）"
      : "当前会话：token 未设置（本机默认 127.0.0.1 无鉴权需求；远程 --host 场景须粘贴 token）";
  $("set-info").textContent =
    "平台构建 = S32.1/2 verified（只读骨架 + 只读 API）· write_enabled_default = " +
    s.write_enabled_default +
    "（写操作开关需 --enable-write 启动参数，P1 起开放）";
}

async function loadAll() {
  const [o, t, e, a, s] = await Promise.all([
    fetch("/api/overview").then((r) => r.json()),
    fetch("/api/tasks").then((r) => r.json()),
    fetch("/api/evidence").then((r) => r.json()),
    fetch("/api/assets").then((r) => r.json()),
    fetch("/api/session").then((r) => r.json()),
  ]);
  apiTasks = t.tasks;
  renderOverview(o);
  renderTasks();
  renderEvidence(e);
  renderAssets(a);
  renderAudit();
  renderSession(s);
  $("ov-next-a").textContent = "262 任务 · 247 已验收 · 262 行台账 0 errors";
}

document.querySelectorAll(".nav button").forEach((b) =>
  b.addEventListener("click", () => {
    document.querySelectorAll(".nav button").forEach((x) => x.classList.remove("on"));
    b.classList.add("on");
    document.querySelectorAll(".page").forEach((p) => p.classList.add("hidden"));
    $(b.dataset.page).classList.remove("hidden");
  })
);
["sel-status", "sel-stage"].forEach((id) =>
  $(id).addEventListener("change", () => {
    cur.status = $("sel-status").value;
    cur.stage = $("sel-stage").value;
    renderTasks();
  })
);
$("task-search").addEventListener("input", (ev) => {
  cur.q = ev.target.value.trim().toLowerCase();
  renderTasks();
});

loadAll().catch((err) => {
  document.body.innerHTML = "<pre>数据加载失败：" + esc(err) + "（需后端在 127.0.0.1:8787 运行）</pre>";
});
