/**
 * cubiomes WASM 绑定层
 *
 * 统一封装 WASM 状态、内存安全与初始化，供 generatorWorker 及其子模块共享
 * （WASM 为单实例，内存增长可能 detach 旧视图，所有读取必须经 ensureHeap）。
 */
import type { InitMsg } from './types'
/** Emscripten 胶水代码（?raw 构建期内联，来源固定为仓库产物，不接收运行时输入） */
import cubiomesGlueCode from '@/assets/seedmap/cubiomes.js?raw'

/** 共享 WASM 单例状态（render / structure-search 经此访问 Module） */
export const wasm = {
  module: null as any,
  ready: false,
}

/** cubiomes 内置 biome 颜色表（init 时从 WASM 读取并复制到独立 buffer） */
let wasmBiomeColors: Uint8Array | null = null

export function getWasmBiomeColors(): Uint8Array | null {
  return wasmBiomeColors
}

/**
 * 重新读取并验证 Module.HEAPU8
 *
 * 必须在每次 WASM 调用后使用，因为 _cubiomes_gen_biomes_with_height 等函数可能触发
 * WASM 内存增长（memory.grow），增长后旧的 typed array 视图可能被 detach。
 */
export function ensureHeap(): Uint8Array {
  const h = wasm.module?.HEAPU8
  if (h && typeof h.set === 'function' && h.buffer && h.buffer.byteLength > 0) {
    return h
  }
  const h32 = wasm.module?.HEAP32
  if (h32 && h32.buffer && h32.buffer.byteLength > 0) {
    console.warn('[cubiomes] HEAPU8 失效，从 HEAP32.buffer 恢复')
    const recovered = new Uint8Array(h32.buffer)
    wasm.module.HEAPU8 = recovered
    return recovered
  }
  throw new Error('HEAPU8 不可用')
}

export function checkModule(): void {
  if (!wasm.ready || !wasm.module) throw new Error('WASM 未就绪')
}

/**
 * 将 seed 字符串写入 WASM 内存，返回指针
 *
 * cubiomes_wrapper.c 的函数接受 const char* seed_str，
 * JS 端通过 _malloc + HEAPU8.set 写入 UTF-8 字节，末尾补 \0。
 */
export function writeSeedString(seed: string): number {
  const encoder = new TextEncoder()
  const bytes = encoder.encode(seed)
  const ptr = wasm.module._malloc(bytes.length + 1)
  if (!ptr) throw new Error('_malloc seed 失败')
  const heap = ensureHeap()
  heap.set(bytes, ptr)
  heap[ptr + bytes.length] = 0
  return ptr
}

/**
 * 加载并初始化 cubiomes WASM（工厂函数 createCubiomesModule，Emscripten MODULARIZE）
 *
 * 渲染管线：biome IDs → BIOME_COLORS 上色 → rgba；height floats → applyTerrainShading → createImageBitmap。
 * 可用 WASM API（cubiomes_wrapper.c）：
 * - _cubiomes_gen_biomes_with_height：群系+高度
 * - _cubiomes_get_structure_pos：结构查找（region 遍历）
 * - _cubiomes_estimate_spawn：出生点
 * - _cubiomes_find_strongholds：多座要塞迭代（上限 128）
 * - _cubiomes_is_slime_chunk：史莱姆区块（按 chunk 逐个判断）
 * - _cubiomes_find_ravines：峡谷系列（mega 需 carveCanyon 验证规模）
 * - _cubiomes_find_nether_fossils / _cubiomes_find_fossils：化石启发式
 */
export async function initModule(msg: InitMsg): Promise<void> {
  if (wasm.module) {
    wasm.ready = true
    return
  }

  // 胶水代码为仓库内静态产物（Vite ?raw 构建期内联）：校验指纹后执行，杜绝任意代码注入
  if (!cubiomesGlueCode.startsWith('var createCubiomesModule=')) {
    throw new Error('cubiomes.js 产物异常，拒绝执行')
  }
  const factoryFn = new Function(cubiomesGlueCode + '\nreturn createCubiomesModule;')
  const factory = factoryFn()
  if (typeof factory !== 'function') {
    throw new Error(`createCubiomesModule 非函数 (typeof=${typeof factory})`)
  }

  // WASM 二进制：优先主线程缓存并 postMessage 传入的字节（各 Worker 共享，不再各自 fetch）；
  // 无缓存时回退到按 URL fetch（兼容独立调用场景）
  let wasmBinary: ArrayBuffer
  if (msg.wasmBytes) {
    wasmBinary = msg.wasmBytes
  } else {
    const wasmResp = await fetch(msg.wasmUrl)
    if (!wasmResp.ok) throw new Error(`加载 cubiomes.wasm 失败: HTTP ${wasmResp.status}`)
    wasmBinary = await wasmResp.arrayBuffer()
  }

  wasm.module = await factory({
    locateFile: () => msg.wasmUrl,
    instantiateWasm: (imports: WebAssembly.Imports, cb: (inst: WebAssembly.Instance) => any) => {
      return new Promise<any>((resolve, reject) => {
        WebAssembly.instantiate(wasmBinary, imports).then(r => {
          const exports = cb(r.instance)
          resolve(exports)
        }).catch(err => {
          console.error('[cubiomes] instantiateWasm failed:', err)
          reject(err)
        })
      })
    },
  })

  ensureHeap()

  try {
    wasm.module._cubiomes_init_biome_colors()
    const colorsPtr = wasm.module._cubiomes_get_all_biome_colors()
    if (colorsPtr) {
      const heap = ensureHeap()
      wasmBiomeColors = new Uint8Array(heap.buffer, colorsPtr, 256 * 3).slice()
      console.log('[cubiomes] biome colors loaded from WASM (256 entries)')
    }
  } catch (e) {
    console.warn('[cubiomes] init biome colors failed, fallback to hardcoded BIOME_COLORS:', e)
  }

  wasm.ready = true
  console.log('[cubiomes] init done, moduleReady=true')
}
