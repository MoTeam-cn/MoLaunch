/**
 * mcmod.cn scraper for MoLaunch moddata.txt
 *
 * Usage:
 *   node scrape.mjs             全量爬取
 *   node scrape.mjs discover    仅发现 ID
 *   node scrape.mjs scrape      仅爬详情（需先 discover）
 *   node scrape.mjs incremental 增量（只爬新 ID）
 */

import https from 'node:https';
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import readline from 'node:readline';
import { fileURLToPath } from 'node:url';

const BASE = 'https://www.mcmod.cn';
const HERE = path.dirname(fileURLToPath(import.meta.url));
const OUT  = path.join(HERE, 'moddata.txt');
const LOGF = path.join(HERE, 'scrape_log.txt');
const CPF  = path.join(HERE, 'scrape_checkpoint.json');
const UA   = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36';

// ── 限流器 ───────────────────────────────────────────────
class Limiter {
  constructor(rps) { this.rps = rps; this.tokens = rps; this.last = Date.now(); }
  async wait() {
    const now = Date.now();
    this.tokens += (now - this.last) / 1000 * this.rps;
    if (this.tokens > this.rps) this.tokens = this.rps;
    this.last = now;
    if (this.tokens >= 1) { this.tokens -= 1; return; }
    const need = (1 - this.tokens) / this.rps * 1000;
    this.tokens = 0;
    await new Promise(r => setTimeout(r, Math.ceil(need)));
  }
}

// ── 并发池 ───────────────────────────────────────────────
async function pool(items, conc, fn, onProgress) {
  const results = [];
  let idx = 0, done = 0;

  async function worker() {
    while (idx < items.length) {
      const i = idx++;
      try { results[i] = await fn(items[i], i); } catch { results[i] = null; }
      done++;
      if (onProgress) onProgress(done, items.length);
    }
  }

  await Promise.all(Array.from({ length: Math.min(conc, items.length) }, () => worker()));
  return results;
}

// ── HTTP ─────────────────────────────────────────────────
const agents = {
  http:  new http.Agent({ keepAlive: true, maxSockets: 50 }),
  https: new https.Agent({ keepAlive: true, maxSockets: 50 }),
};

function fetch(url) {
  return new Promise((resolve) => {
    const mod = url.startsWith('https') ? https : http;
    const req = mod.get(url, {
      agent: agents[url.startsWith('https') ? 'https' : 'http'],
      headers: { 'User-Agent': UA, 'Accept': 'text/html,*/*', 'Accept-Language': 'zh-CN,zh' },
      timeout: 15000,
    }, res => {
      if (res.statusCode >= 301 && res.statusCode <= 303) {
        resolve(fetch(new URL(res.headers.location, url).href));
        return;
      }
      let b = '';
      res.on('data', c => b += c);
      res.on('end', () => resolve(res.statusCode >= 400 ? null : b));
    });
    req.on('error', () => resolve(null));
    req.on('timeout', () => { req.destroy(); resolve(null); });
  });
}

// ── 日志 / TTY ──────────────────────────────────────────
function logRaw(m) {
  const d = new Date(new Date().getTime() + 8*3600000);
  const ts = d.toISOString().replace('Z', '+0800');
  fs.appendFileSync(LOGF, `[${ts}] ${m}\n`, 'utf-8');
}
function logBatch(msgs) {
  const d = new Date(new Date().getTime() + 8*3600000);
  const ts = d.toISOString().replace('Z', '+0800');
  const text = msgs.map(m => `[${ts}] ${m}`).join('\n');
  fs.appendFileSync(LOGF, text + '\n', 'utf-8');
}

function fmtTime(ms) {
  const s = Math.floor(ms / 1000), m = Math.floor(s / 60), h = Math.floor(m / 60);
  if (h) return `${h}h ${m%60}m`;
  if (m) return `${m}m ${s%60}s`;
  return `${s}s`;
}

const tty = process.stdout.isTTY;
const SCR_H = 5; // 固定行数
function scr(rows) {
  if (!tty) { rows.forEach(r => console.log(r)); return; }
  readline.cursorTo(process.stdout, 0, 0);
  // 补齐到固定行数，避免残留
  const buf = [];
  for (let i = 0; i < SCR_H; i++) buf.push(rows[i] || '');
  process.stdout.write(buf.join('\n') + '\x1b[J');
}

// ── Phase 1 ──────────────────────────────────────────────
function extractIds(html) {
  const s = new Set();
  for (const m of html.matchAll(/\/class\/(\d+)\.html/g)) s.add(+m[1]);
  return s;
}

async function discoverAllIds() {
  logRaw('Phase1 start');
  const all = new Set();
  const total = 890;
  const t0 = Date.now();
  const lim = new Limiter(10);
  let totalIds = 0;

  scr(['▶ Phase 1: 扫描列表页  (0.0%)',
       `[${'░'.repeat(30)}] 0/${total}`,
       `已发现: 0 ID  `,
       '并发: 初始化...',
       '用时: 0s']);

  const results = await pool(
    Array.from({ length: total }, (_, i) => i + 1),
    6,
    async (pg) => {
      await lim.wait();
      const r = await fetch(`${BASE}/modlist.html?page=${pg}`);
      if (!r) return null;
      const ids = extractIds(r);
      if (ids.size) all.add(ids);
      return ids;
    },
    (done) => {
      const el = Date.now() - t0;
      const pct = done / total;
      const bar = '█'.repeat(Math.round(pct*30)) + '░'.repeat(30 - Math.round(pct*30));
      const eta = done > 0 ? (el / done) * (total - done) : 0;
      // 计算总 ID 数（粗估：用 all 的 size）
      let idCount = 0;
      for (const s of all) idCount += s.size;

       scr([`▶ Phase 1: 扫描列表页  (${(pct*100).toFixed(1)}%)`,
           `[${bar}] ${done}/${total}`,
           `已发现: ${idCount.toLocaleString()} 个 ID`,
           `并发: 6 线程  速率: ${(done/(el/1000)).toFixed(1)} pg/s`,
           `用时: ${fmtTime(el)}  剩余: ${fmtTime(eta)}`]);
       // 每 100 页写一次日志
       if (done % 100 === 0) logRaw(`discover ${done}/${total} pages, ${idCount} IDs`);
    }
  );

  const merged = new Set();
  for (const r of results) if (r) for (const id of r) merged.add(id);
  const arr = [...merged].sort((a, b) => a - b);

  scr([`✓ Phase 1 完成`,
       `发现 ${arr.length.toLocaleString()} 个 ID (${arr[0]} ~ ${arr[arr.length-1]})`,
       `总用时 ${fmtTime(Date.now() - t0)}`, '', '']);
  logRaw(`Phase1 done: ${arr.length} IDs`);
  return arr;
}

// ── Phase 2 ──────────────────────────────────────────────
function decodeLink(href) {
  const m = href.match(/\/target\/([A-Za-z0-9+/=]+)/);
  if (!m) return null;
  try { return Buffer.from(m[1], 'base64').toString('utf-8'); } catch { return null; }
}

function parsePage(html) {
  const t = html.match(/<title>([^<]+)<\/title>/);
  if (!t) return null;
  let raw = t[1].replace(/\s*[-–|].*$/, '').trim();
  let en = '';
  const p = raw.match(/\(([^)]*)\)\s*$/);
  if (p) { en = p[1].trim(); raw = raw.slice(0, raw.lastIndexOf('(')).trim(); }
  raw = raw.replace(/\[[^\]]*\]\s*/, '').trim();
  const cn = raw || en;

  const idx = html.indexOf('相关链接');
  let cf = null, mr = null;
  if (idx >= 0) {
    const sec = html.substring(idx, idx + 4000);
    const re = /<a[^>]*data-original-title="(CurseForge|Modrinth)"[^>]*href="([^"]*)"[^>]*>/g;
    let m;
    while ((m = re.exec(sec)) !== null) {
      const decoded = decodeLink(m[2]);
      if (!decoded) continue;
      if (m[1] === 'CurseForge') {
        const sm = decoded.match(/curseforge\.com\/minecraft\/mc-mods\/([\w-]+)/i);
        if (sm) cf = sm[1];
      } else {
        const sm = decoded.match(/modrinth\.com\/mod\/([\w-]+)/i);
        if (sm) mr = sm[1];
      }
    }
  }
  return { cn, en, cf, mr };
}

function cap(s) { return s.replace(/(?:^|-)([a-z])/g, (_, c) => c.toUpperCase()); }

function encode(cn, en, cf, mr) {
  const slug = cf && mr ? (cf === mr ? `${cf}@` : `${cf}@${mr}`)
              : mr ? `@${mr}`
              : cf || en?.toLowerCase().replace(/[^a-z0-9_-]/g, '-') || '';
  const name = cn && en
    ? ((cf || mr) && cap(slug.replace(/-/g, ' ')).toLowerCase() === en.toLowerCase()
       ? cn + '*' : `${cn} (${en})`)
    : cn || en || slug;
  return `${slug}|${name}`;
}

async function scrapeOne(id) {
  const html = await fetch(`${BASE}/class/${id}.html`);
  if (!html) return null;
  return parsePage(html);
}

// ── 文件输出 ─────────────────────────────────────────────
function readExistingLines() {
  if (!fs.existsSync(OUT)) return [];
  const text = fs.readFileSync(OUT, 'utf-8');
  return text.split('\n');
}

function writeOut(scrapedIds, map, pathOut) {
  const oldLines = readExistingLines();
  // 去除尾部空行（文件末尾 \n 导致的 split 多余元素）
  while (oldLines.length && oldLines[oldLines.length - 1] === '') oldLines.pop();
  const oldData = oldLines.length > 0 ? oldLines.slice(0, -1) : [];
  const oldPop = oldLines.length > 0 ? oldLines[oldLines.length - 1] : '';

  const maxId = Math.max(scrapedIds.length ? scrapedIds[scrapedIds.length - 1] : 0, oldData.length);
  const lines = [];
  const popVals = [];

  for (let id = 1; id <= maxId; id++) {
    const m = map.get(id);
    if (m) {
      lines.push(encode(m.cn, m.en, m.cf, m.mr));
    } else if (id <= oldData.length && oldData[id - 1].trim()) {
      lines.push(oldData[id - 1]);
    } else {
      lines.push('');
    }
    // 解析旧 popularity 值
    if (id <= oldData.length && oldPop) {
      const parts = oldPop.split('|');
      popVals.push(parts[id] || '0');
    } else {
      popVals.push('0');
    }
  }
  lines.push('|' + popVals.join('|'));
  fs.writeFileSync(pathOut, lines.join('\n'), 'utf-8');
}

function existingIds() {
  if (!fs.existsSync(OUT)) return new Set();
  let lines = fs.readFileSync(OUT, 'utf-8').split('\n');
  while (lines.length && lines[lines.length - 1] === '') lines.pop();
  lines.pop(); // 去掉 popularity 行
  const set = new Set();
  lines.forEach((l, i) => { if (l.trim()) set.add(i + 1); });
  return set;
}

// ── Main ─────────────────────────────────────────────────
async function main() {
  const args = process.argv.slice(2);
  const mode = args[0] || 'full';

  if (mode === 'help') { console.log(`Usage: node scrape.mjs [full|discover|scrape|incremental]`); return; }

  process.on('exit', () => tty && process.stdout.write('\x1b[?25h'));
  process.on('SIGINT', () => { tty && process.stdout.write('\n\x1b[?25h'); process.exit(0); });

  if (tty) process.stdout.write('\n\n\n\n\x1b[?25l');

  let ids = [];

  if (mode === 'discover' || mode === 'full' || mode === 'incremental') {
    ids = await discoverAllIds();
    fs.writeFileSync(CPF, JSON.stringify({ discoveredIds: ids, time: new Date().toISOString() }), 'utf-8');
  } else {
    ids = fs.existsSync(CPF) ? JSON.parse(fs.readFileSync(CPF, 'utf-8')).discoveredIds || [] : [];
    if (!ids.length) { console.log('请先运行 discover'); return; }
  }

  if (mode === 'incremental') {
    const covered = existingIds();
    ids = ids.filter(id => !covered.has(id));
    if (!ids.length) { console.log('没有新 ID'); return; }
    logRaw(`incremental: ${ids.length}`);
  }

  if (mode === 'scrape' || mode === 'full' || mode === 'incremental') {
    if (!ids.length) { console.log('nothing to scrape'); return; }
    logRaw(`Phase2 start: ${ids.length} IDs`);
    const map = new Map();
    const t0 = Date.now();
    const total = ids.length;
    const lim = new Limiter(15);
    let lastLogDone = 0;

    scr(['▶ Phase 2: 抓取详情  (0.0%)',
         `[${'░'.repeat(30)}] 0/${total.toLocaleString()}`,
         '成功: 0  失败: 0',
         '当前: 初始化中...',
         '用时: 0s']);

    const results = await pool(
      ids, 8,
      async (id) => { await lim.wait(); return scrapeOne(id); },
      (done) => {
        const el = Date.now() - t0;
        const pct = done / total;
        const bar = '█'.repeat(Math.round(pct*30)) + '░'.repeat(30 - Math.round(pct*30));
        const eta = done > 0 ? (el / done) * (total - done) : 0;
        let ok = 0, fail = 0, cur = '';
        for (let i = 0; i < done; i++) {
          if (results[i] === undefined) continue;
          if (results[i] === null) fail++;
          else { ok++; cur = results[i].cn + (results[i].en ? ' ('+results[i].en+')' : ''); }
        }

       const curTrunc = cur.length > 50 ? cur.substring(0, 47) + '...' : cur;
       scr([`▶ Phase 2: 抓取详情  (${(pct*100).toFixed(1)}%)`,
           `[${bar}] ${done.toLocaleString()}/${total.toLocaleString()}`,
           `成功: ${ok}  失败: ${fail}  速率: ${(done/(el/1000/60)).toFixed(0)}/min`,
           `当前: class/${ids[done-1]}.html  ${curTrunc}`,
           `用时: ${fmtTime(el)}  剩余: ${fmtTime(eta)}`]);

        // 每 100 条写日志
        if (done - lastLogDone >= 100) {
          lastLogDone = done;
          logRaw(`scrape ${done}/${total}  ok=${ok} fail=${fail}`);
        }
        if (done % 300 === 0) {
          for (let i = 0; i < done; i++) if (results[i] && results[i] !== null) map.set(ids[i], results[i]);
          writeOut(ids, map, OUT);
        }
      }
    );

    let ok = 0, fail = 0;
    for (let i = 0; i < results.length; i++) {
      if (results[i] && results[i] !== null) { map.set(ids[i], results[i]); ok++; }
      else fail++;
    }
    writeOut(ids, map, OUT);

    scr([`✓ Phase 2 完成`,
         `成功: ${ok}  失败: ${fail}  总计: ${total}`,
         `总用时 ${fmtTime(Date.now() - t0)}`,
         `已更新 moddata.txt`,
         '']);
    logRaw(`Phase2 done: ok=${ok} fail=${fail}`);
  }

  if (tty) process.stdout.write('\x1b[?25h');
}

main().catch(e => {
  if (tty) process.stdout.write('\x1b[?25h');
  console.error('错误:', e.message);
  logRaw('FATAL: ' + e.message);
  process.exit(1);
});
