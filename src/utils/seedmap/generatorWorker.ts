/**
 * 种子地图生成 Worker 入口
 *
 * 消息串行化（单队列防并发 WASM 内存操作）+ 任务分发。
 * WASM 绑定/渲染/结构查找分别由 wasm-bindings/tile-render/structure-search 承载。
 */
import type {
  MainToWorkerMsg, WorkerToMainMsg, GenerateTileMsg, FindStructuresMsg,
  SpecialsMsg, BiomeAtPointMsg, InitMsg,
} from './types'
import { initModule, checkModule, writeSeedString, wasm } from './wasm-bindings'
import { renderTile } from './tile-render'
import { findStructures, findSpecials, findBiomeAtPoint } from './structure-search'

const queue: MainToWorkerMsg[] = []
let draining = false

function post(msg: WorkerToMainMsg, transfer: Transferable[] = []) {
  ;(self as any).postMessage(msg, transfer)
}

function postError(jobId: string, err: unknown) {
  post({ type: 'error', jobId, error: err instanceof Error ? err.message : String(err) })
}

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
      } else if (wasm.ready) {
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

async function handleInit(msg: InitMsg) {
  if (wasm.module) {
    wasm.ready = true
    post({ type: 'init_complete', jobId: msg.jobId, seedEpoch: msg.seedEpoch })
    return
  }
  await initModule(msg)
  post({ type: 'init_complete', jobId: msg.jobId, seedEpoch: msg.seedEpoch })
}

// 上游 cubiomes API 无状态化（每次调用都 setupGenerator + applySeed），
// prepare_seed 仅作协议兼容，无实际操作。
function handlePrepareSeed(msg: { seed: string; seedEpoch: number }) {
  checkModule()
  post({ type: 'seed_prepared', seed: msg.seed, seedEpoch: msg.seedEpoch })
}

async function handleGenerate(msg: GenerateTileMsg) {
  checkModule()
  const bitmap = await renderTileWithSeed(msg)
  post({ type: 'tile_result', jobId: msg.jobId, imageBitmap: bitmap }, [bitmap])
}

async function renderTileWithSeed(msg: GenerateTileMsg): Promise<ImageBitmap> {
  const { seed, mcVersion, dimension, largeBiomes, blockX, blockZ, sx, sz, scale, y, doContour, ymax } = msg
  const seedPtr = writeSeedString(seed)
  try {
    return await renderTile({
      seedPtr,
      mcVersion, dimension, largeBiomes,
      scale, blockX, blockZ, sx, sz, y, doContour, ymax,
    })
  } finally {
    wasm.module._free(seedPtr)
  }
}

async function handleFindStructures(msg: FindStructuresMsg) {
  checkModule()
  const structures = findStructures(msg)
  post({ type: 'structure_result', jobId: msg.jobId, structures })
}

function handleSpecials(msg: SpecialsMsg) {
  checkModule()
  const { spawn, strongholds } = findSpecials(msg)
  post({ type: 'specials_result', jobId: msg.jobId, spawn, strongholds })
}

function handleBiomeAtPoint(msg: BiomeAtPointMsg) {
  checkModule()
  const biomeId = findBiomeAtPoint(msg)
  post({ type: 'biome_at_point_result', jobId: msg.jobId, biomeId })
}
