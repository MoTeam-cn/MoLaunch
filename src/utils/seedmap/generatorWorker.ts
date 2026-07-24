/**
 * 种子地图生成 Worker
 *
 * 基于上游 cubiomes（fork: https://github.com/MoTeam-cn/cubiomes，支持 MC_26_2）。
 *
 * API：
 * - 工厂函数名：createCubiomesModule（Emscripten 标准 MODULARIZE）
 * - 群系+高度：_cubiomes_gen_biomes_with_height（cubiomes_wrapper.c）
 * - 结构查找：_cubiomes_get_structure_pos（region 遍历）
 * - 出生点：_cubiomes_estimate_spawn
 * - 要塞：_cubiomes_find_strongholds（多座要塞迭代，上限 128）
 * - 史莱姆区块：_cubiomes_is_slime_chunk（按 chunk 逐个判断）
 * - 峡谷系列：_cubiomes_find_ravines（checkCanyonStart + carveCanyon for mega）
 * - 下界化石：_cubiomes_find_nether_fossils（soul_sand_valley 中心启发式）
 * - 化石系列：_cubiomes_find_fossils（desert/swamp/mangrove 中心启发式；diamond 额外要求 deep_dark）
 *
 * 渲染：
 * - biome IDs → BIOME_COLORS 上色 → rgba
 * - height floats → applyTerrainShading（hillshade + terrace + contour）
 * - createImageBitmap → 主线程
 *
 * 架构要点：
 * 1. 消息串行化：所有消息进入单一队列，避免并发 WASM 内存操作
 * 2. WASM 加载：new Function + instantiateWasm 回调预实例化（避免 res:// fetch 问题）
 * 3. HEAPU8 安全访问：每次 WASM 调用后重新读取（内存增长可能 detach 旧视图）
 *
 * WASM 来源：src-tauri/resources/wasm/cubiomes.{js,wasm}（build.rs 自动编译）
 * 前端通过 res:// 协议加载（src/utils/wasm-loader.ts + src-tauri/src/res_scheme.rs）
 */
import { BIOME_COLORS, DEFAULT_COLOR } from './constants'
import { getStructuresByDimension } from './structures'
import { applyTerrainShading } from './terrainShading'
import type {
  MainToWorkerMsg, WorkerToMainMsg,
  GenerateTileMsg, FindStructuresMsg, InitMsg, SpecialsMsg, PrepareSeedMsg,
  BiomeAtPointMsg,
  WorkerStructure,
} from './types'

const TILE_SIZE = 64

let Module: any = null
let moduleReady = false

/**
 * cubiomes 内置 biome 颜色表（256×3 RGB，init 时从 WASM 读取并复制到独立 buffer）
 *
 * 覆盖所有 biome ID（0~255），比前端硬编码的 BIOME_COLORS 更完整，
 * 不会出现 DEFAULT_COLOR 灰色填充。WASM 未初始化或读取失败时回退到 BIOME_COLORS。
 */
let wasmBiomeColors: Uint8Array | null = null

// 消息队列：所有消息（含 init）进入单一队列，串行处理
const queue: MainToWorkerMsg[] = []
let draining = false

// ===== 消息收发 =====

function post(msg: WorkerToMainMsg, transfer: Transferable[] = []) {
  ;(self as any).postMessage(msg, transfer)
}

function postError(jobId: string, err: unknown) {
  post({ type: 'error', jobId, error: err instanceof Error ? err.message : String(err) })
}

// ===== WASM 内存安全 =====

/**
 * 重新读取并验证 Module.HEAPU8
 *
 * 必须在每次 WASM 调用后使用，因为 _cubiomes_gen_biomes_with_height 等函数可能触发
 * WASM 内存增长（memory.grow），增长后旧的 typed array 视图可能被 detach。
 */
function ensureHeap(): Uint8Array {
  const h = Module?.HEAPU8
  if (h && typeof h.set === 'function' && h.buffer && h.buffer.byteLength > 0) {
    return h
  }
  // 兜底：从 HEAP32.buffer 恢复
  const h32 = Module?.HEAP32
  if (h32 && h32.buffer && h32.buffer.byteLength > 0) {
    console.warn('[cubiomes] HEAPU8 失效，从 HEAP32.buffer 恢复')
    const recovered = new Uint8Array(h32.buffer)
    Module.HEAPU8 = recovered
    return recovered
  }
  throw new Error('HEAPU8 不可用')
}

function checkModule(): void {
  if (!moduleReady || !Module) throw new Error('WASM 未就绪')
}

// ===== 字符串 seed 指针管理 =====

/**
 * 将 seed 字符串写入 WASM 内存，返回指针
 *
 * cubiomes_wrapper.c 的函数接受 const char* seed_str，
 * JS 端通过 _malloc + HEAPU8.set 写入 UTF-8 字节，末尾补 \0。
 */
function writeSeedString(seed: string): number {
  const encoder = new TextEncoder()
  const bytes = encoder.encode(seed)
  const ptr = Module._malloc(bytes.length + 1)
  if (!ptr) throw new Error('_malloc seed 失败')
  const heap = ensureHeap()
  heap.set(bytes, ptr)
  heap[ptr + bytes.length] = 0  // null terminator
  return ptr
}

// ===== 消息串行处理（防止并发 WASM 内存操作） =====

self.onmessage = (e: MessageEvent<MainToWorkerMsg>) => {
  queue.push(e.data)
  if (!draining) drainQueue()
}

async function drainQueue() {
  if (draining) return
  draining = true
  while (queue.length > 0) {
    const msg = queue.shift()!
    try {
      if (msg.type === 'init') {
        await handleInit(msg)
      } else if (msg.type === 'dispose') {
        queue.length = 0
        break
      } else if (moduleReady) {
        await handleMessage(msg)
      } else {
        queue.unshift(msg)
        break
      }
    } catch (err) {
      if ('jobId' in msg) {
        postError(msg.jobId, err)
      } else if (msg.type === 'prepare_seed') {
        console.error('[cubiomes] prepare_seed 失败:', err)
      }
    }
  }
  draining = false
}

async function handleMessage(msg: MainToWorkerMsg) {
  switch (msg.type) {
    case 'prepare_seed':
      handlePrepareSeed(msg)
      break
    case 'generate':
      await handleGenerate(msg)
      break
    case 'find_structures':
      await handleFindStructures(msg)
      break
    case 'specials':
      handleSpecials(msg)
      break
    case 'biome_at_point':
      handleBiomeAtPoint(msg)
      break
    case 'obsolete':
      break
  }
}

// ===== Init：加载 WASM =====

async function handleInit(msg: InitMsg) {
  if (Module) {
    moduleReady = true
    post({ type: 'init_complete', jobId: msg.jobId, seedEpoch: msg.seedEpoch })
    return
  }

  // 1. 并行 fetch cubiomes.js（胶水代码）和 cubiomes.wasm（二进制）
  const [jsResp, wasmResp] = await Promise.all([
    fetch(msg.wasmJsUrl),
    fetch(msg.wasmUrl),
  ])
  if (!jsResp.ok) throw new Error(`加载 cubiomes.js 失败: HTTP ${jsResp.status}`)
  if (!wasmResp.ok) throw new Error(`加载 cubiomes.wasm 失败: HTTP ${wasmResp.status}`)
  const [jsCode, wasmBinary] = await Promise.all([jsResp.text(), wasmResp.arrayBuffer()])

  // 2. 用 new Function 执行 Emscripten 胶水代码
  //    上游 cubiomes 工厂函数名是 createCubiomesModule（非原站的 CubiomesModule）
  const factoryFn = new Function(jsCode + '\nreturn createCubiomesModule;')
  const factory = factoryFn()
  if (typeof factory !== 'function') {
    throw new Error(`createCubiomesModule 非函数 (typeof=${typeof factory})`)
  }

  // 3. 用 instantiateWasm 回调传入预 fetch 的 wasmBinary
  //    避免 res:// 协议下 Emscripten 内部 fetch/instantiateStreaming 兼容性问题
  //    必须返回 Promise，确保 HEAPU8 赋值（updateMemoryViews）完成后才 resolve
  Module = await factory({
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

  // 4. 验证 HEAPU8
  ensureHeap()

  // 5. 初始化 cubiomes 内置 biome 颜色表（256×3 RGB）
  //    initBiomeColors 覆盖所有 biome ID，比前端硬编码 BIOME_COLORS 更完整。
  //    复制到独立 Uint8Array，避免 WASM 内存增长导致 HEAPU8 视图失效。
  try {
    Module._cubiomes_init_biome_colors()
    const colorsPtr = Module._cubiomes_get_all_biome_colors()
    if (colorsPtr) {
      const heap = ensureHeap()
      wasmBiomeColors = new Uint8Array(heap.buffer, colorsPtr, 256 * 3).slice()
      console.log('[cubiomes] biome colors loaded from WASM (256 entries)')
    }
  } catch (e) {
    console.warn('[cubiomes] init biome colors failed, fallback to hardcoded BIOME_COLORS:', e)
  }

  moduleReady = true
  console.log('[cubiomes] init done, moduleReady=true')
  post({ type: 'init_complete', jobId: msg.jobId, seedEpoch: msg.seedEpoch })
}

// ===== Prepare Seed =====
// 上游 cubiomes API 无状态化（每次调用都 setupGenerator + applySeed），
// prepare_seed 仅作协议兼容，无实际操作。

function handlePrepareSeed(msg: PrepareSeedMsg) {
  checkModule()
  post({ type: 'seed_prepared', seed: msg.seed, seedEpoch: msg.seedEpoch })
}

// ===== Generate Tile =====

async function handleGenerate(msg: GenerateTileMsg) {
  checkModule()

  const { seed, mcVersion, dimension, largeBiomes, blockX, blockZ, sx, sz, scale } = msg
  const y = msg.y ?? 64  // 默认海平面

  // ===== 坐标系转换（核心：block → scale） =====
  // 主线程传入的 blockX/blockZ 是方块坐标（tile 左上角 / 西北角）。
  // 但 cubiomes Range.x/z 期望的是 **scale 坐标**（每单位 = scale 个方块），
  // 见 cubiomes/biomenoise.h Range 注释 + cubiomes-viewer/src/world.cpp:436
  //   r = {scale, x, z, w, h, y, 1};  其中 x = ti*pixs, z = tj*pixs（scale 坐标）
  // 若直接传 block 坐标，cubiomes 会按 scale 倍偏移生成，导致 tile 内容
  // 完全错位、相邻 tile 边界不连续（这是上一版的核心 bug）。
  if (scale <= 0 || !Number.isInteger(scale)) {
    throw new Error(`invalid scale=${scale}（必须为正整数）`)
  }
  if (blockX % scale !== 0 || blockZ % scale !== 0) {
    throw new Error(
      `blockX/Z(${blockX},${blockZ}) 必须是 scale(${scale}) 的整数倍，` +
      `否则 tile 边界无法对齐 scale 网格`,
    )
  }
  const rangeX = (blockX / scale) | 0
  const rangeZ = (blockZ / scale) | 0

  // ===== 高度图生成策略 =====
  // cubiomes_gen_biomes_with_height_static 内部用 1:4 分辨率生成 height：
  //   hw = ceil(sx*scale / 4), hh = ceil(sz*scale / 4)
  // 当 scale 较大时 height buffer 急剧膨胀：
  //   scale=16, sx=64 → hw=256   → 256KB   （可接受）
  //   scale=64, sx=64 → hw=1024  → 4MB     （偏重）
  //   scale=256,sx=64 → hw=4096  → 64MB    （爆炸）
  // 因此 scale > 16 时改用 _cubiomes_gen_biomes_static（仅 biome，不生成 height），
  // 跳过地形阴影。远观级别（scale=64/256）阴影细节本就不可见。
  const SKIP_HEIGHT_THRESHOLD = 16
  const withHeight = scale <= SKIP_HEIGHT_THRESHOLD

  // ===== 调用 WASM（pointer 模式：结果存入 C 端内部 buffer） =====
  const seedPtr = writeSeedString(seed)
  try {
    let ret: number
    if (withHeight) {
      // cubiomes_gen_biomes_with_height_static(seed, mc, dim, large, scale, x, z, sx, sz, y)
      ret = Module._cubiomes_gen_biomes_with_height_static(
        seedPtr, mcVersion, dimension, largeBiomes ? 1 : 0,
        scale, rangeX, rangeZ, sx, sz, y,
      )
      if (ret !== 0) {
        throw new Error(`_cubiomes_gen_biomes_with_height_static 失败 (code=${ret})`)
      }
    } else {
      // cubiomes_gen_biomes_static(seed, mc, dim, large, scale, x, z, sx, sz)
      ret = Module._cubiomes_gen_biomes_static(
        seedPtr, mcVersion, dimension, largeBiomes ? 1 : 0,
        scale, rangeX, rangeZ, sx, sz,
      )
      if (ret !== 0) {
        throw new Error(`_cubiomes_gen_biomes_static 失败 (code=${ret})`)
      }
    }

    // ===== 从 C 端内部 buffer 读取 biome =====
    const heap = ensureHeap()
    const biomePtr = Module._cubiomes_get_biome_data_pointer()
    const biomeSize = Module._cubiomes_get_biome_data_size()
    if (!biomePtr || biomeSize < sx * sz) {
      throw new Error(`biome buffer 无效 (ptr=${biomePtr}, size=${biomeSize}, expect=${sx * sz})`)
    }
    const biomeData = new Int32Array(heap.buffer, biomePtr, sx * sz)

    // ===== 渲染：biome 上色 =====
    // OL TileGrid 用 top-left origin（origin=[-EXTENT_HALF, +EXTENT_HALF]），
    // tile y=0 在屏幕顶部 = 投影 max Y = MC max Z（本项目约定 +Z=北方）。
    // 主线程计算 startBlockZ = EXTENT_HALF - (y+1)*blocksPerTile = tile 的 min Z（南方边缘）。
    // cubiomes Range.z = min Z（scale 坐标），生成数据 gz=0 → min Z（南），gz=sz-1 → max Z（北）。
    // 因此图像 py=0（顶部=北）需对应 cubiomes gz=sz-1（北）→ Z 轴翻转。
    // （cubiomes-viewer 不翻转是因为它用 bottom-up tile 索引，与 cubiomes Z 方向天然一致）
    const rgba = new Uint8ClampedArray(TILE_SIZE * TILE_SIZE * 4)
    for (let py = 0; py < TILE_SIZE; py++) {
      const gz = sz - 1 - Math.min(sz - 1, Math.floor(py * sz / TILE_SIZE))
      for (let px = 0; px < TILE_SIZE; px++) {
        const gx = Math.min(sx - 1, Math.floor(px * sx / TILE_SIZE))
        const id = biomeData[gz * sx + gx]
        const idx = (py * TILE_SIZE + px) * 4
        // 优先用 WASM 内置颜色表（覆盖全部 256 个 biome），fallback 到前端硬编码
        if (wasmBiomeColors && id >= 0 && id < 256) {
          const o = id * 3
          rgba[idx] = wasmBiomeColors[o]
          rgba[idx + 1] = wasmBiomeColors[o + 1]
          rgba[idx + 2] = wasmBiomeColors[o + 2]
        } else {
          const c = BIOME_COLORS[id] ?? DEFAULT_COLOR
          rgba[idx] = c[0]
          rgba[idx + 1] = c[1]
          rgba[idx + 2] = c[2]
        }
        rgba[idx + 3] = 255
      }
    }

    // ===== 应用地形阴影（仅 withHeight=true） =====
    if (withHeight) {
      const heap2 = ensureHeap()  // 重新读取（mapApproxHeight 可能触发内存增长）
      const dimsPtr = Module._cubiomes_get_height_grid_dims()
      const dims = new Int32Array(heap2.buffer, dimsPtr, 2)
      const hw = dims[0]
      const hh = dims[1]
      if (hw > 0 && hh > 0) {
        const heightPtr = Module._cubiomes_get_height_data_pointer()
        if (heightPtr) {
          const heightData = new Float32Array(heap2.buffer, heightPtr, hw * hh)
          const heightCellPx = TILE_SIZE / hw
          // 翻转 height 数组的 Z 方向（与 biome 一致），并复制到独立 buffer
          // （applyTerrainShading 会多次访问，避免每次都从 HEAPF32 读取）
          const heights = new Float32Array(hw * hh)
          for (let z = 0; z < hh; z++) {
            const srcZ = hh - 1 - z
            heights.set(heightData.subarray(srcZ * hw, (srcZ + 1) * hw), z * hw)
          }
          applyTerrainShading(rgba, TILE_SIZE, TILE_SIZE, heights, hw, hh, heightCellPx, {
            scale,
            pixelsPerCell: TILE_SIZE / sx,
            doContour: msg.doContour ?? false,
            ymax: msg.ymax ?? Infinity,
          })
        }
      }
    }

    // ===== 输出 ImageBitmap =====
    const imageData = new ImageData(rgba, TILE_SIZE, TILE_SIZE)
    const bitmap = await createImageBitmap(imageData)
    post({ type: 'tile_result', jobId: msg.jobId, imageBitmap: bitmap }, [bitmap])
  } finally {
    Module._free(seedPtr)
  }
}

// ===== Find Structures =====

/**
 * 史莱姆区块遍历上限：可视范围 chunk 数超过此值时跳过，避免卡死 Worker。
 * 10000 chunks ≈ 160×160 区块 ≈ 2560×2560 方块，正常缩放下足够覆盖。
 */
const SLIME_CHUNK_LIMIT = 10000

/**
 * 峡谷/化石查找的 chunk 范围硬上限。
 * - 非 mega（ravine/underwater_ravine/nether_fossil/fossil/fossil_diamond）：64x64=4096 chunks
 * - mega（mega_ravine/mega_underwater_ravine）：32x32=1024 chunks（carveCanyon 较慢）
 * 超过此值时跳过该结构类型查找，避免卡死 Worker。
 */
const CHUNK_FIND_LIMIT_NON_MEGA = 64
const CHUNK_FIND_LIMIT_MEGA = 32

/** carveCanyon poses.size 阈值：超过此值视为 mega 峡谷（cubiomes 默认 1024 上限，经验值 200） */
const MEGA_RAVINE_POSE_THRESHOLD = 200

/**
 * 调用 cubiomes chunk 范围查找函数（ravines/nether_fossils/fossils 共用模式）
 *
 * 统一封装：chunk 范围计算 → buffer 分配 → WASM 调用 → 结果读取 → buffer 释放。
 * WASM 函数签名：(startCX, startCZ, numX, numZ, bufPtr, bufLen) → count
 *
 * @param fn         WASM 调用闭包（接收 chunk 范围 + buffer 指针/长度，返回结果数量）
 * @param minX/minZ/maxX/maxZ  可视范围方块坐标
 * @param sizeLimit  X/Z 方向 chunk 数上限（超过则跳过，返回 null）
 * @returns 坐标数组（每项 {x, z}），或 null（范围过大跳过 / 分配失败）
 */
function callChunkFinder(
  fn: (startCX: number, startCZ: number, numX: number, numZ: number,
       bufPtr: number, bufLen: number) => number,
  minX: number, minZ: number, maxX: number, maxZ: number,
  sizeLimit: number,
): { x: number; z: number }[] | null {
  const startCX = Math.floor(minX / 16)
  const endCX = Math.floor(maxX / 16)
  const startCZ = Math.floor(minZ / 16)
  const endCZ = Math.floor(maxZ / 16)
  const numX = endCX - startCX + 1
  const numZ = endCZ - startCZ + 1
  if (numX > sizeLimit || numZ > sizeLimit) return null

  /* 每结果占 2 个 int (x, z)，buffer 大小 = numX * numZ * 2（上限情况） */
  const bufLen = numX * numZ * 2
  const bufPtr = Module._malloc(bufLen * 4)
  if (!bufPtr) return null

  try {
    const count = fn(startCX, startCZ, numX, numZ, bufPtr, bufLen)
    if (count <= 0) return []
    const heap = ensureHeap()
    const buf = new Int32Array(heap.buffer, bufPtr, count * 2)
    const result: { x: number; z: number }[] = []
    for (let i = 0; i < count; i++) {
      result.push({ x: buf[i * 2], z: buf[i * 2 + 1] })
    }
    return result
  } finally {
    Module._free(bufPtr)
  }
}

async function handleFindStructures(msg: FindStructuresMsg) {
  checkModule()

  const { seed, mcVersion, dimension, largeBiomes, minX, minZ, maxX, maxZ } = msg
  const structs: WorkerStructure[] = []
  const types = getStructuresByDimension(dimension)

  const seedPtr = writeSeedString(seed)
  try {
    for (const tconf of types) {
      // queryMode='stronghold' 走 specials 流程（cubiomes_find_strongholds 多座迭代），
      // 此处跳过避免与 specials 重复绘制。
      if (tconf.queryMode === 'stronghold') continue

      // queryMode='slime'：遍历可视范围 chunk 调 isSlimeChunk 判断
      if (tconf.queryMode === 'slime') {
        try {
          // chunk 范围（block / 16，向下取整 / 向上取整确保覆盖）
          const startCX = Math.floor(minX / 16)
          const endCX = Math.floor(maxX / 16)
          const startCZ = Math.floor(minZ / 16)
          const endCZ = Math.floor(maxZ / 16)
          const chunkCount = (endCX - startCX + 1) * (endCZ - startCZ + 1)
          // 可视范围过大时跳过，避免卡死
          if (chunkCount > SLIME_CHUNK_LIMIT) continue

          for (let cx = startCX; cx <= endCX; cx++) {
            for (let cz = startCZ; cz <= endCZ; cz++) {
              // _cubiomes_is_slime_chunk(seed, chunk_x, chunk_z) 返回 1/0
              const isSlime = Module._cubiomes_is_slime_chunk(seedPtr, cx, cz)
              if (isSlime) {
                // chunk 中心方块坐标 = chunk*16 + 8
                structs.push({
                  stype: tconf.name,
                  x: cx * 16 + 8,
                  z: cz * 16 + 8,
                  viable: true,
                })
              }
            }
          }
        } catch { /* isSlimeChunk 调用失败，跳过 */ }
        continue
      }

      // queryMode='ravine' / 'mega_ravine' / 'underwater_ravine' / 'mega_underwater_ravine'
      // → 调 _cubiomes_find_ravines（canyonType: 0=CANYON_CARVER, 1=UNDERWATER_CANYON_CARVER；
      //   megaThreshold: 0=不区分, >0=carveCanyon poses.size 阈值）
      const qm = tconf.queryMode
      if (qm === 'ravine' || qm === 'mega_ravine'
          || qm === 'underwater_ravine' || qm === 'mega_underwater_ravine') {
        const isUnderwater = (qm === 'underwater_ravine' || qm === 'mega_underwater_ravine')
        const isMega = (qm === 'mega_ravine' || qm === 'mega_underwater_ravine')
        // canyonType: 0=CANYON_CARVER, 1=UNDERWATER_CANYON_CARVER（finders.h:617）
        const canyonType = isUnderwater ? 1 : 0
        // megaThreshold: 0=不区分（ravine/underwater_ravine）; MEGA_RAVINE_POSE_THRESHOLD=mega（需 carveCanyon）
        const megaThreshold = isMega ? MEGA_RAVINE_POSE_THRESHOLD : 0
        // mega 模式 carveCanyon 较慢，限制 32x32；非 mega 仅 checkCanyonStart，限制 64x64
        const sizeLimit = isMega ? CHUNK_FIND_LIMIT_MEGA : CHUNK_FIND_LIMIT_NON_MEGA

        try {
          const results = callChunkFinder(
            (sx, sz, nx, nz, bp, bl) => Module._cubiomes_find_ravines(
              seedPtr, mcVersion, dimension, sx, sz, nx, nz,
              canyonType, megaThreshold, bp, bl,
            ),
            minX, minZ, maxX, maxZ, sizeLimit,
          )
          if (results) {
            for (const r of results) {
              structs.push({ stype: tconf.name, x: r.x, z: r.z, viable: true })
            }
          } else {
            console.warn(`[cubiomes] ${qm} 范围过大，跳过`)
          }
        } catch { /* ravine 查找失败，跳过 */ }
        continue
      }

      // queryMode='nether_fossil' → 调 _cubiomes_find_nether_fossils（soul_sand_valley 中心启发式）
      if (qm === 'nether_fossil') {
        try {
          const results = callChunkFinder(
            (sx, sz, nx, nz, bp, bl) => Module._cubiomes_find_nether_fossils(
              seedPtr, mcVersion, sx, sz, nx, nz, bp, bl,
            ),
            minX, minZ, maxX, maxZ, CHUNK_FIND_LIMIT_NON_MEGA,
          )
          if (results) {
            for (const r of results) {
              structs.push({ stype: tconf.name, x: r.x, z: r.z, viable: true })
            }
          } else {
            console.warn(`[cubiomes] nether_fossil 范围过大，跳过`)
          }
        } catch { /* nether_fossil 查找失败，跳过 */ }
        continue
      }

      // queryMode='fossil' / 'fossil_diamond' → 调 _cubiomes_find_fossils（biome 中心启发式）
      if (qm === 'fossil' || qm === 'fossil_diamond') {
        const diamondMode = qm === 'fossil_diamond' ? 1 : 0
        try {
          const results = callChunkFinder(
            (sx, sz, nx, nz, bp, bl) => Module._cubiomes_find_fossils(
              seedPtr, mcVersion, dimension, sx, sz, nx, nz,
              diamondMode, bp, bl,
            ),
            minX, minZ, maxX, maxZ, CHUNK_FIND_LIMIT_NON_MEGA,
          )
          if (results) {
            for (const r of results) {
              structs.push({ stype: tconf.name, x: r.x, z: r.z, viable: true })
            }
          } else {
            console.warn(`[cubiomes] ${qm} 范围过大，跳过`)
          }
        } catch { /* fossil 查找失败，跳过 */ }
        continue
      }

      try {
        // 获取该结构类型的 regionSize
        // cubiomes getStructurePos 内部按 StructureConfig.regionSize 处理：
        //   常规结构按 region 查找，Mineshaft 按 chunk 查找（queryMode='mineshaft'）
        //   都通过同一个 API 调用，无需分支
        const regionSize = Module._cubiomes_get_region_size(tconf.id, mcVersion)
        if (!regionSize) continue

        // 遍历覆盖可视范围的所有 region
        const startRegX = Math.floor(minX / regionSize)
        const endRegX = Math.floor(maxX / regionSize)
        const startRegZ = Math.floor(minZ / regionSize)
        const endRegZ = Math.floor(maxZ / regionSize)

        for (let rx = startRegX; rx <= endRegX; rx++) {
          for (let rz = startRegZ; rz <= endRegZ; rz++) {
            // 查询该 region 是否有结构
            // _cubiomes_get_structure_pos(stype, seed, mc, reg_x, reg_z, out_x, out_z)
            // 返回 1=有结构，0=无结构
            const outXPtr = Module._malloc(4)
            const outZPtr = Module._malloc(4)
            if (!outXPtr || !outZPtr) {
              if (outXPtr) Module._free(outXPtr)
              if (outZPtr) Module._free(outZPtr)
              continue
            }
            try {
              const found = Module._cubiomes_get_structure_pos(
                tconf.id, seedPtr, mcVersion, rx, rz, outXPtr, outZPtr,
              )
              if (found) {
                const heap = ensureHeap()
                const x = new Int32Array(heap.buffer, outXPtr, 1)[0]
                const z = new Int32Array(heap.buffer, outZPtr, 1)[0]
                // 过滤可视范围外
                if (x >= minX && x <= maxX && z >= minZ && z <= maxZ) {
                  // 可行性检查
                  let viable = true
                  try {
                    viable = Module._cubiomes_is_viable(
                      tconf.id, seedPtr, mcVersion, dimension, largeBiomes ? 1 : 0, x, z,
                    ) === 1
                  } catch { /* skip viability check */ }
                  structs.push({ stype: tconf.name, x, z, viable })
                }
              }
            } finally {
              Module._free(outXPtr)
              Module._free(outZPtr)
            }
          }
        }
      } catch { /* 该结构类型可能不支持，跳过 */ }
    }
  } finally {
    Module._free(seedPtr)
  }

  post({ type: 'structure_result', jobId: msg.jobId, structures: structs })
}

// ===== Specials (Spawn + Strongholds) =====
//
// 要塞遍历：cubiomes nextStronghold 迭代器最多返回 128 座要塞（MC_1_9+）。
// 旧版仅返回首座（cubiomes_first_stronghold），无法显示完整要塞分布；
// 现改为 cubiomes_find_strongholds 一次性返回 max_count 座，OL 自动渲染所有 Feature。

/** 要塞查询上限（cubiomes 默认要塞总数） */
const MAX_STRONGHOLDS = 128

function handleSpecials(msg: SpecialsMsg) {
  checkModule()

  const { seed, mcVersion, largeBiomes } = msg
  const seedPtr = writeSeedString(seed)

  let spawn: { x: number; z: number } | null = null
  const strongholds: { x: number; z: number }[] = []

  try {
    // 出生点（固定主世界）
    const spawnXPtr = Module._malloc(4)
    const spawnZPtr = Module._malloc(4)
    if (spawnXPtr && spawnZPtr) {
      try {
        Module._cubiomes_estimate_spawn(seedPtr, mcVersion, largeBiomes ? 1 : 0, spawnXPtr, spawnZPtr)
        const heap = ensureHeap()
        spawn = {
          x: new Int32Array(heap.buffer, spawnXPtr, 1)[0],
          z: new Int32Array(heap.buffer, spawnZPtr, 1)[0],
        }
      } finally {
        Module._free(spawnXPtr)
        Module._free(spawnZPtr)
      }
    }

    // 多座要塞（仅主世界）：_cubiomes_find_strongholds(seed, mc, max_count, out_buffer, out_len)
    // out_buffer 每座要塞占 2 个 int（x, z 交替），返回实际找到的数量
    const bufLen = MAX_STRONGHOLDS * 2
    const shBufPtr = Module._malloc(bufLen * 4)
    if (shBufPtr) {
      try {
        const count = Module._cubiomes_find_strongholds(
          seedPtr, mcVersion, MAX_STRONGHOLDS, shBufPtr, bufLen,
        )
        if (count > 0) {
          const heap = ensureHeap()
          const buf = new Int32Array(heap.buffer, shBufPtr, count * 2)
          for (let i = 0; i < count; i++) {
            strongholds.push({ x: buf[i * 2], z: buf[i * 2 + 1] })
          }
        }
      } catch { /* 要塞查找失败 */ } finally {
        Module._free(shBufPtr)
      }
    }
  } finally {
    Module._free(seedPtr)
  }

  post({ type: 'specials_result', jobId: msg.jobId, spawn, strongholds })
}

// ===== Biome At Point（鼠标悬停查询群系名） =====

function handleBiomeAtPoint(msg: BiomeAtPointMsg) {
  checkModule()
  const seedPtr = writeSeedString(msg.seed)
  try {
    const biomeId = Module._cubiomes_get_biome_at_point(
      seedPtr, msg.mcVersion, msg.dimension, msg.largeBiomes ? 1 : 0,
      msg.scale, msg.x, msg.y, msg.z,
    )
    post({ type: 'biome_at_point_result', jobId: msg.jobId, biomeId })
  } finally {
    Module._free(seedPtr)
  }
}
