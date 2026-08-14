/**
 * WASM 加载工具：经 Tauri `res://` 协议加载后端嵌入的 WASM 文件。
 * 协议格式 web-common/{type}/{file}（Windows: https://res.localhost，macOS/Linux: res://localhost）。
 * 提供 loadWasm() 加载 + wasmUrl() 获取 URL 供 Worker fetch。后端对应 src-tauri/src/res_scheme.rs。
 */

/** res 协议的 scheme 名（与后端 res_scheme.rs 的 RES_SCHEME 一致） */
const RES_SCHEME = 'res'

/** res 协议根路径前缀（与后端 res_scheme.rs 的 RES_ROOT 一致） */
const RES_ROOT = 'web-common'

/**
 * 判断当前是否为 Windows/Android 平台
 *
 * Tauri v2 + useHttpsScheme=true 时：
 *   - Windows/Android: https://res.localhost/
 *   - macOS/Linux: res://localhost/
 *
 * 不依赖 @tauri-apps/plugin-os（项目未引入），改用 navigator.userAgent 探测。
 */
function isHttpsSchemePlatform(): boolean {
  if (typeof navigator === 'undefined') return false
  const ua = navigator.userAgent || navigator.platform || ''
  return /Windows|Android/i.test(ua)
}

/**
 * 获取 res:// 协议在当前平台的完整 URL 前缀
 *
 * Tauri v2 + useHttpsScheme=true：Windows/Android → https://res.localhost/，macOS/Linux → res://localhost/。
 */
function getResBaseUrl(): string {
  if (isHttpsSchemePlatform()) {
    return `https://${RES_SCHEME}.localhost/${RES_ROOT}/`
  }
  return `${RES_SCHEME}://localhost/${RES_ROOT}/`
}

/** 缓存 res 基础 URL，避免每次重算 */
let cachedBaseUrl: string | null = null

function ensureBaseUrl(): string {
  if (!cachedBaseUrl) {
    cachedBaseUrl = getResBaseUrl()
  }
  return cachedBaseUrl
}

/**
 * 构造 WASM 文件的 res:// URL
 *
 * @param filename WASM 文件名（如 'cubiomes.wasm'）
 * @param type 资源类型子目录（默认 'wasm'），对应 res://web-common/{type}/{filename}
 * @returns 完整 URL，如 'https://res.localhost/web-common/wasm/cubiomes.wasm'
 */
export function resUrl(filename: string, type = 'wasm'): string {
  return `${ensureBaseUrl()}${type}/${filename}`
}

/**
 * 加载并编译 WASM 模块（主线程使用）
 *
 * Worker 内部请直接用 `resUrl()` 获取 URL 后 `fetch()` + `WebAssembly.instantiate()`，
 * 不要用此函数（主线程/Worker 的 WebAssembly API 略有差异）。
 *
 * @param filename WASM 文件名
 * @param type 资源类型子目录
 * @returns WebAssembly.Module（未实例化，调用方按需 instantiate）
 */
export async function loadWasmModule(
  filename: string,
  type = 'wasm',
): Promise<WebAssembly.Module> {
  const url = resUrl(filename, type)
  const response = await fetch(url)
  if (!response.ok) {
    throw new Error(`WASM 加载失败: ${filename} (HTTP ${response.status})`)
  }
  // compileStreaming 需要 CORS + 正确 MIME，Tauri res:// 协议已配置
  return WebAssembly.compileStreaming(response)
}

/**
 * 加载 WASM 并返回 ArrayBuffer（用于 Worker 间通过 postMessage 传递）
 *
 * 当 WebAssembly.Module 无法跨 Worker 结构化克隆时（部分浏览器），
 * 改用 ArrayBuffer 让 Worker 自己 instantiate。
 */
export async function loadWasmBytes(
  filename: string,
  type = 'wasm',
): Promise<ArrayBuffer> {
  const url = resUrl(filename, type)
  const response = await fetch(url)
  if (!response.ok) {
    throw new Error(`WASM 加载失败: ${filename} (HTTP ${response.status})`)
  }
  return response.arrayBuffer()
}

/**
 * 预取 WASM URL（同步返回，便于在 Worker 创建前就构造好）
 *
 * 用法：
 *   const url = prefetchWasmUrl('cubiomes.wasm')
 *   // ... 创建 Worker
 *   worker.postMessage({ type: 'init', wasmUrl: url })
 */
export function prefetchWasmUrl(filename: string, type = 'wasm'): string {
  return resUrl(filename, type)
}

/** Emscripten 胶水 JS + WASM 二进制（种子地图 Worker 初始化用） */
export interface WasmBundle {
  /** 胶水 JS 代码文本（cubiomes.js） */
  jsCode: string
  /** WASM 二进制（cubiomes.wasm） */
  wasmBytes: ArrayBuffer
  /** 胶水 JS 的 res:// URL（回退与日志用） */
  wasmJsUrl: string
  /** WASM 的 res:// URL（Emscripten locateFile 回退用） */
  wasmUrl: string
}

/** 主线程缓存：同一份胶水 JS + WASM 只 fetch 一次，Worker 通过 postMessage 共享字节 */
const bundleCache = new Map<string, Promise<WasmBundle>>()

/**
 * 获取 Emscripten 胶水 JS + WASM 二进制（主线程缓存，Worker 不再各自 fetch）
 *
 * 每次进入页面重复创建 Worker 时，由主线程先把字节拉取一次并缓存，
 * init 消息携带字节传给各 Worker，避免每个 Worker 都走一次 res:// 加载。
 *
 * @param jsFilename 胶水 JS 文件名（如 'cubiomes.js'），wasm 取同名 .wasm
 * @param type 资源类型子目录（默认 'wasm'）
 */
export function getWasmBundle(jsFilename: string, type = 'wasm'): Promise<WasmBundle> {
  const wasmFilename = jsFilename.replace(/\.js$/, '.wasm')
  const key = `${type}/${jsFilename}`
  let cached = bundleCache.get(key)
  if (!cached) {
    cached = (async () => {
      const wasmJsUrl = resUrl(jsFilename, type)
      const wasmUrl = resUrl(wasmFilename, type)
      const [jsResp, wasmBytes] = await Promise.all([fetch(wasmJsUrl), loadWasmBytes(wasmFilename, type)])
      if (!jsResp.ok) {
        throw new Error(`加载 ${jsFilename} 失败: HTTP ${jsResp.status}`)
      }
      return { jsCode: await jsResp.text(), wasmBytes, wasmJsUrl, wasmUrl }
    })()
    bundleCache.set(key, cached)
  }
  return cached
}
