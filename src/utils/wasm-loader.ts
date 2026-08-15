/**
 * WASM 加载工具：从 Vite assets 加载 cubiomes WASM 文件。
 * 产物位于 src/assets/seedmap/，经 import.meta.glob(...?url) 交给 Vite 处理——
 * dev 由 dev server 提供源文件，build 输出带 hash 的产物并自动替换 URL。
 * 提供 loadWasm() 加载 + seedmapUrl() 获取 URL 供 Worker fetch。
 */

// Vite assets 清单：cubiomes.{js,wasm}（?url 强制按静态资产处理，build 后带 hash 文件名）
const seedmapAssets = import.meta.glob<string>('../assets/seedmap/*.{js,wasm}', {
  query: '?url',
  import: 'default',
  eager: true,
})

// basename → 最终 URL 映射（如 'cubiomes.js' → '/assets/cubiomes-<hash>.js'）
const seedmapUrlMap = new Map<string, string>(
  Object.entries(seedmapAssets).map(([key, url]) => [key.split('/').pop()!, url]),
)

/**
 * 获取 seedmap WASM/JS 资产的最终 URL
 * dev 为源文件路径（/src/assets/seedmap/...），build 为带 hash 的产物路径
 *
 * @param filename 资源文件名（如 'cubiomes.wasm'）
 */
export function seedmapUrl(filename: string): string {
  const url = seedmapUrlMap.get(filename)
  if (!url) {
    throw new Error(`未知的 seedmap 资源: ${filename}`)
  }
  return url
}

/**
 * 加载并编译 WASM 模块（主线程使用）
 *
 * Worker 内部请直接用 `seedmapUrl()` 获取 URL 后 `fetch()` + `WebAssembly.instantiate()`，
 * 不要用此函数（主线程/Worker 的 WebAssembly API 略有差异）。
 *
 * @param filename WASM 文件名
 * @returns WebAssembly.Module（未实例化，调用方按需 instantiate）
 */
export async function loadWasmModule(
  filename: string,
): Promise<WebAssembly.Module> {
  const url = seedmapUrl(filename)
  const response = await fetch(url)
  if (!response.ok) {
    throw new Error(`WASM 加载失败: ${filename} (HTTP ${response.status})`)
  }
  // compileStreaming 需要 CORS + 正确 MIME，Vite assets 已配置
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
): Promise<ArrayBuffer> {
  const url = seedmapUrl(filename)
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
export function prefetchWasmUrl(filename: string): string {
  return seedmapUrl(filename)
}

/** Emscripten 胶水 JS + WASM 二进制（种子地图 Worker 初始化用） */
export interface WasmBundle {
  /** 胶水 JS 代码文本（cubiomes.js） */
  jsCode: string
  /** WASM 二进制（cubiomes.wasm） */
  wasmBytes: ArrayBuffer
  /** 胶水 JS 的资产 URL（回退与日志用） */
  wasmJsUrl: string
  /** WASM 的资产 URL（Emscripten locateFile 回退用） */
  wasmUrl: string
}

/** 主线程缓存：同一份胶水 JS + WASM 只 fetch 一次，Worker 通过 postMessage 共享字节 */
const bundleCache = new Map<string, Promise<WasmBundle>>()

/**
 * 获取 Emscripten 胶水 JS + WASM 二进制（主线程缓存，Worker 不再各自 fetch）
 *
 * 每次进入页面重复创建 Worker 时，由主线程先把字节拉取一次并缓存，
 * init 消息携带字节传给各 Worker，避免每个 Worker 都走一次资产加载。
 *
 * @param jsFilename 胶水 JS 文件名（如 'cubiomes.js'），wasm 取同名 .wasm
 */
export function getWasmBundle(jsFilename: string): Promise<WasmBundle> {
  const wasmFilename = jsFilename.replace(/\.js$/, '.wasm')
  let cached = bundleCache.get(jsFilename)
  if (!cached) {
    cached = (async () => {
      const wasmJsUrl = seedmapUrl(jsFilename)
      const wasmUrl = seedmapUrl(wasmFilename)
      const [jsResp, wasmBytes] = await Promise.all([fetch(wasmJsUrl), loadWasmBytes(wasmFilename)])
      if (!jsResp.ok) {
        throw new Error(`加载 ${jsFilename} 失败: HTTP ${jsResp.status}`)
      }
      return { jsCode: await jsResp.text(), wasmBytes, wasmJsUrl, wasmUrl }
    })()
    bundleCache.set(jsFilename, cached)
  }
  return cached
}
