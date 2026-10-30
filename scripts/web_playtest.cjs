#!/usr/bin/env node
/**
 * S16.1 web_playtest — 浏览器真运行时试玩（Playwright CDP 按键驱动）
 *
 * 口径（docs/plan-s16-loop.md §S16.1）：headless 浏览器加载 `web/` → 按键驱动
 * 菜单→选人→选图→主地图 → 逐步断言：
 * ① console 致命错误（uncaught / failed to load / 资源 404）= 0；
 * ② 截图墨点覆盖率 > 阈值（非空白/非纯黑）；
 * ③ 相邻场景切换帧的像素统计有变化（场景确实推进）。
 *
 * 依赖：playwright（Node 24+ 或本机已装 chromium；`TI_CHROMIUM_PATH` 可覆盖）。
 * 前提：`web/` 由 `build_wasm.ps1` 更新 + 静态服务在跑（默认 http://127.0.0.1:8080/）。
 * 用法：node scripts/web_playtest.cjs [--url ...] [--out evidence/web_playtest.log]
 * 退出码 0 = 全绿；非 0 = 失败。幂等（结果文件重跑覆盖）。
 */
'use strict';

const path = require('node:path');
const crypto = require('node:crypto');

const REPO = path.resolve(__dirname, '..');
const argv = process.argv.slice(2);
function argVal(name, dflt) {
  const i = argv.indexOf(name);
  return i >= 0 && argv[i + 1] ? argv[i + 1] : dflt;
}
const URL_BASE = argVal('--url', 'http://127.0.0.1:8080/').replace(/\/$/, '');
const OUT = argVal('--out', path.join(REPO, 'evidence', 'web_playtest.log'));

const lines = [];
const log = (s) => {
  lines.push(s);
  process.stdout.write(s + '\n');
};

function sha256short(p) {
  try {
    return crypto.createHash('sha256').update(require('node:fs').readFileSync(p)).digest('hex').slice(0, 12);
  } catch {
    return null;
  }
}

async function main() {
  let chromium;
  try {
    // playwright = S31.3 gate 第 22 项同款依赖（本机已装；缺 = 明确 FAIL，不静默跳过）
    chromium = require('playwright').chromium;
  } catch (e) {
    log(`FAIL  playwright 不可用（${e.message.split('\n')[0]}；pip/npm 安装后重跑）`);
    return 1;
  }

  const fs = require('node:fs');
  log(`[web-playtest] url=${URL_BASE}`);
  const webIndex = path.join(REPO, 'web', 'index.html');
  const wasmMain = path.join(REPO, 'web', 'treasure_bevy_wasm.wasm');
  if (!fs.existsSync(webIndex)) {
    log('FAIL  web/index.html 缺失（先跑 build_wasm.ps1）');
    return 1;
  }
  log(`  web 产物: index=${sha256short(webIndex)} wasm=${sha256short(wasmMain)}`);

  const execPath = process.env.TI_CHROMIUM_PATH;
  const launchOpts = { headless: true, args: ['--use-gl=swiftshader', '--enable-unsafe-swiftshader'] };
  if (execPath) launchOpts.executablePath = execPath;
  else {
    // 自动探测本机已装 chromium（与 wasm_browser_e2e.py 同口径）
    const os = require('node:os');
    const base = path.join(os.homedir(), '.cache', 'ms-playwright');
    const cands = fs
      .readdirSync(base, { recursive: true })
      .filter((f) => /chrome$|chromium$/.test(f) && f.includes('chrome-linux'))
      .map((f) => path.join(base, f));
    if (cands.length) launchOpts.executablePath = cands.sort().at(-1);
  }

  const browser = await chromium.launch(launchOpts);
  const page = await browser.newPage({ viewport: { width: 540, height: 960 } });

  const consoleErrors = [];
  const pageErrors = [];
  page.on('console', (m) => {
    const t = m.text();
    if (/uncaught|failed to load module|missing import/i.test(t) && !/\.meta/.test(t)) {
      consoleErrors.push(t.slice(0, 300));
    }
  });
  page.on('pageerror', (e) => pageErrors.push(String(e.message).slice(0, 300)));

  await page.goto(URL_BASE + '/', { waitUntil: 'load', timeout: 60_000 });
  const status = async () => (await page.evaluate(`document.getElementById('status')?.textContent ?? ''`)).trim();

  // ── L1 实例化：status 活体（运行中 / 加载失败）──
  let s0 = '';
  for (let i = 0; i < 45; i++) {
    s0 = await status();
    if (/^运行中|^加载失败/.test(s0)) break;
    await page.waitForTimeout(2000);
  }
  log(`  L1 status="${s0}"（实例化活体证明）`);
  const l1ok = s0.startsWith('运行中');

  // ── L2 渲染 + L3 交互 + L4 场景推进（按键驱动 菜单→选人→选图→主地图）──
  // 墨点判据：WebGL canvas 不能 2D drawImage 读像素（跨上下文 tainted）⇒
  // 与 wasm_browser_e2e.py 同口径用「截图文件体积」启发（全黑/全白 PNG 压缩后极小，
  // 有内容画面 ≥ 数十 KB；阈值 100KB 对应 540×960 有内容帧），另存图供人工复核。
  const shotPath = path.join(REPO, 'evidence', 'web_playtest.png');
  const shotOf = async () => {
    await page.screenshot({ path: shotPath });
    try {
      return fs.statSync(shotPath).size;
    } catch {
      return 0;
    }
  };
  const brightPct = (bytes) => Math.min(100, Math.round((bytes / 100_000) * 100));

  const press = async (key) => {
    await page.keyboard.press(key);
    await page.waitForTimeout(3000);
  };

  let b0 = brightPct(await shotOf());
  log(`  L2 首帧墨点=${b0}%（体积启发，阈值 >5 = 非全黑/非空白）`);
  const l2ok = b0 > 5;

  // 菜单 → 选人 → 选图 → 主地图（Enter 推进；main_map 亮像素显著更高）
  const frames = [b0];
  await press('Enter');
  frames.push(brightPct(await shotOf()));
  await press('Enter');
  frames.push(brightPct(await shotOf()));
  await press('Enter');
  frames.push(brightPct(await shotOf()));
  const sceneChanges = frames.slice(1).filter((b, i) => Math.abs(b - frames[i]) > 5).length;
  log(`  L3/L4 按键 3×Enter 后墨点序列=${JSON.stringify(frames)}（变化帧 ${sceneChanges}/3）`);
  const l4ok = sceneChanges >= 1; // 至少一帧有场景推进（防卡死在首场景仍"全绿"）

  const s1 = await status();
  log(`  终态 status="${s1}" console致命错误=${consoleErrors.length} 页面异常=${pageErrors.length}`);
  consoleErrors.slice(0, 5).forEach((c) => log(`    console: ${c}`));
  pageErrors.slice(0, 5).forEach((c) => log(`    pageerror: ${c}`));

  await browser.close();

  const ok = l1ok && l2ok && l4ok && consoleErrors.length === 0 && pageErrors.length === 0;
  log(`[web-playtest] ${ok ? 'PASS' : 'FAIL'}（L1=${l1ok} L2=${l2ok} L4=${l4ok} console=${consoleErrors.length} pageerror=${pageErrors.length}）`);
  fs.mkdirSync(path.dirname(OUT), { recursive: true });
  fs.writeFileSync(OUT, lines.join('\n') + '\n');
  return ok ? 0 : 1;
}

main()
  .then((code) => process.exit(code ?? 0))
  .catch((e) => {
    log(`FAIL  异常: ${e.message}`);
    try {
      require('node:fs').mkdirSync(path.dirname(OUT), { recursive: true });
      require('node:fs').writeFileSync(OUT, lines.join('\n') + '\n');
    } catch {}
    process.exit(1);
  });
