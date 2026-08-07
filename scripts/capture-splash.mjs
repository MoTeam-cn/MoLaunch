import { createRequire } from 'node:module'
import { mkdir, rm } from 'node:fs/promises'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const require = createRequire(import.meta.url)
const puppeteer = require(join(process.env.TEMP, 'splash-gif', 'node_modules', 'puppeteer-core'))

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const framesDir = join(root, '.splash-frames')
const url = 'file://' + join(root, 'public', 'splash.html').replaceAll('\\', '/')

const W = 640
const H = 180
const FPS = 30
const DURATION_MS = 5000

// 门闩：拦截 splash.js 的 DOMContentLoaded 监听器，页面加载期间动画不会启动，
// 待页面就绪后手动 __unlock()，确保录制从动画起点（t=0）开始，不丢开头帧。
const gateJs = `
  window.__gateQueue = [];
  const _origAdd = document.addEventListener.bind(document);
  document.addEventListener = function (type, fn, opts) {
    if (type === 'DOMContentLoaded') { window.__gateQueue.push(fn); return undefined; }
    return _origAdd(type, fn, opts);
  };
  window.__unlock = function () {
    const q = window.__gateQueue; window.__gateQueue = null;
    for (const fn of q) fn();
  };
`

await rm(framesDir, { recursive: true, force: true })
await mkdir(framesDir, { recursive: true })

const browser = await puppeteer.launch({
  executablePath: 'C:/Program Files/Google/Chrome/Application/chrome.exe',
  headless: true,
  args: ['--no-sandbox', '--disable-gpu'],
})
const page = await browser.newPage()
await page.setViewport({ width: W, height: H })
await page.evaluateOnNewDocument(gateJs)
await page.goto(url, { waitUntil: 'networkidle0', timeout: 30000 })

await page.evaluate(() => window.__unlock())

const total = Math.floor(DURATION_MS / (1000 / FPS))
let frame = 0
const start = Date.now()
for (let i = 0; i < total; i++) {
  const t = Date.now() - start
  if (t > DURATION_MS) break
  const file = join(framesDir, `frame-${String(frame).padStart(4, '0')}.png`)
  await page.screenshot({ path: file })
  frame++
  const next = (i + 1) * (1000 / FPS)
  const wait = next - (Date.now() - start)
  if (wait > 0) await new Promise(r => setTimeout(r, wait))
}

await browser.close()
console.log(`captured ${frame} frames -> ${framesDir}`)
