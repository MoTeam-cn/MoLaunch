/**
 * 种子地图 WorkerPool 调度层
 *
 * 复刻 minecraftsearch.com 的 WorkerPool 架构（docs/Map/map.md §4.1）：
 * - 多 Worker 并行处理 tile 生成与结构查找
 * - prepareSeed 广播给所有 Worker，等所有 Worker 确认后才返回
 * - 任务通过 jobId 关联 Promise，主线程无需关心 Worker 选择
 * - Worker 数量：clamp(4, floor(0.75 * hardwareConcurrency), 16)，低配降级到 2
 *
 * 与原站的差异：
 * - 不共享 WebAssembly.Module（每个 Worker 独立实例化，简化实现）
 * - 不维护 seedEpoch 缓存（cubiomes_wrapper.c 每次 setupGenerator，无需预热）
 */
import type {
  GenerateTileParams, FindStructuresParams, SpecialsParams, SpecialsResult,
  BiomeAtPointParams,
  WorkerStructure, WorkerToMainMsg,
} from './types'

interface JobResolver<T = unknown> {
  resolve: (value?: T) => void
  reject: (err: Error) => void
}

interface WorkerHandle {
  worker: Worker
  healthy: boolean
  seedReady: boolean
  /** pending 任务 Map<jobId, resolver> */
  pending: Map<string, JobResolver<any>>
  /** prepare_seed 任务的 resolver/rejecter（无 jobId，单独存放） */
  seedResolve: (() => void) | null
  seedReject: ((err: Error) => void) | null
  /** init 任务的 resolver */
  initResolve: (() => void) | null
  errorCount: number
}

export interface WorkerPoolOptions {
  /** 自定义 Worker 数量（不传则按 hardwareConcurrency 计算） */
  workerCount?: number
}

const MAX_ERRORS_PER_WORKER = 5

export class WorkerPool {
  private workers: WorkerHandle[] = []
  private nextWorkerIdx = 0
  private disposed = false
  /** 计划创建的 Worker 数量（init 后归零） */
  private plannedCount: number

  constructor(opts: WorkerPoolOptions = {}) {
    this.plannedCount = opts.workerCount ?? this.computeWorkerCount()
  }

  /** 计算合理的 Worker 数量 */
  private computeWorkerCount(): number {
    const hwc = typeof navigator !== 'undefined' ? navigator.hardwareConcurrency : 4
    if (hwc <= 2) return Math.min(2, hwc)
    return Math.min(16, Math.max(4, Math.floor(0.75 * hwc)))
  }

  /**
   * 初始化 WorkerPool：创建 N 个 Worker 并并行 init
   * @param wasmJsUrl Emscripten cubiomes.js 的 URL
   * @param wasmUrl cubiomes.wasm 的 URL
   */
  async init(wasmJsUrl: string, wasmUrl: string): Promise<void> {
    if (this.disposed) throw new Error('WorkerPool 已 dispose')
    const count = this.plannedCount
    const initPromises: Promise<void>[] = []
    for (let i = 0; i < count; i++) {
      // Vite 自动打包：new URL('./generatorWorker.ts', import.meta.url) + { type: 'module' } → ESM worker bundle
      // 必须用 module worker，因为 generatorWorker.ts 使用了 ES import 语法
      const worker = new Worker(new URL('./generatorWorker.ts', import.meta.url), { type: 'module' })
      const jobId = `init_${i}_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 8)}`
      const handle: WorkerHandle = {
        worker,
        healthy: false,
        seedReady: false,
        pending: new Map(),
        seedResolve: null,
        seedReject: null,
        initResolve: null,
        errorCount: 0,
      }
      this.workers.push(handle)
      worker.onmessage = (e: MessageEvent<WorkerToMainMsg>) => this.onMessage(handle, e.data)
      worker.onerror = (e) => this.onError(handle, e)
      const initPromise = new Promise<void>((resolve, reject) => {
        handle.initResolve = resolve
        // 用 jobId 关联：init 失败时 Worker 通过 error 消息回传 jobId，触发 reject
        // （之前用 '__init__' 固定 key，但 init 消息无 jobId，Worker 内 catch 不会 postError，导致失败被吞）
        handle.pending.set(jobId, { resolve: () => resolve(), reject })
      })
      worker.postMessage({ type: 'init', jobId, wasmJsUrl, wasmUrl, seedEpoch: 0 })
      initPromises.push(initPromise)
    }
    await Promise.all(initPromises)
  }

  /** 广播 prepare_seed 到所有 Worker，等所有 Worker 确认 */
  async prepareSeed(seed: string, mcVersion: number, dimension: number, largeBiomes: boolean): Promise<void> {
    if (this.disposed) throw new Error('WorkerPool 已 dispose')
    const promises = this.workers.map(w => new Promise<void>((resolve, reject) => {
      w.seedResolve = resolve
      w.seedReject = reject
      w.seedReady = false
      w.worker.postMessage({
        type: 'prepare_seed',
        seed,
        seedEpoch: 1, // 单一 epoch（不递增），cubiomes_wrapper.c 不依赖 epoch
        mcVersion,
        dimension,
        largeBiomes,
      })
    }))
    await Promise.all(promises)
  }

  /** 生成单个 tile，返回 ImageBitmap */
  generateTile(params: GenerateTileParams): Promise<ImageBitmap> {
    return this.enqueue<ImageBitmap>('generate', params)
  }

  /** 在指定范围内查找结构 */
  findStructures(params: FindStructuresParams): Promise<WorkerStructure[]> {
    return this.enqueue<WorkerStructure[]>('find_structures', params)
  }

  /** 查询出生点 + 多座要塞（cubiomes nextStronghold 迭代，上限 128） */
  getSpecials(params: SpecialsParams): Promise<SpecialsResult> {
    return this.enqueue<SpecialsResult>('specials', params)
  }

  /** 查询单点 biome ID（供鼠标悬停显示群系名） */
  getBiomeAtPoint(params: BiomeAtPointParams): Promise<number> {
    return this.enqueue<number>('biome_at_point', params)
  }

  /** 销毁所有 Worker */
  dispose(): void {
    this.disposed = true
    for (const w of this.workers) {
      w.worker.terminate()
    }
    this.workers = []
  }

  // ===== 内部方法 =====

  private enqueue<T>(type: 'generate' | 'find_structures' | 'specials' | 'biome_at_point', params: unknown): Promise<T> {
    if (this.disposed) return Promise.reject(new Error('WorkerPool 已 dispose'))
    if (this.workers.length === 0) return Promise.reject(new Error('WorkerPool 未 init'))
    const worker = this.pickWorker()
    const jobId = `${type}_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 8)}`
    return new Promise<T>((resolve, reject) => {
      worker.pending.set(jobId, { resolve: resolve as (v: unknown) => void, reject })
      worker.worker.postMessage({ type, jobId, ...(params as object) })
    })
  }

  /** 选一个 Worker 派发任务：优先 healthy && seedReady && idle，否则 pending 最少 */
  private pickWorker(): WorkerHandle {
    let best = this.workers[0]
    let bestScore = -Infinity
    for (let i = 0; i < this.workers.length; i++) {
      const idx = (this.nextWorkerIdx + i) % this.workers.length
      const w = this.workers[idx]
      if (!w.healthy) continue
      // 评分：idle 最高，pending 越少越优先
      const score = w.pending.size === 0 ? 1000 : 100 - w.pending.size
      if (score > bestScore) {
        bestScore = score
        best = w
        if (score === 1000) {
          this.nextWorkerIdx = (idx + 1) % this.workers.length
          break
        }
      }
    }
    return best
  }

  private onMessage(worker: WorkerHandle, msg: WorkerToMainMsg) {
    switch (msg.type) {
      case 'init_complete': {
        worker.healthy = true
        const initEntry = worker.pending.get(msg.jobId)
        if (initEntry) {
          worker.pending.delete(msg.jobId)
          initEntry.resolve()
        }
        worker.initResolve?.()
        worker.initResolve = null
        return
      }
      case 'seed_prepared': {
        worker.seedReady = true
        worker.seedResolve?.()
        worker.seedResolve = null
        worker.seedReject = null
        return
      }
      case 'tile_result': {
        const entry = worker.pending.get(msg.jobId)
        if (entry) {
          worker.pending.delete(msg.jobId)
          entry.resolve(msg.imageBitmap)
        }
        return
      }
      case 'structure_result': {
        const entry = worker.pending.get(msg.jobId)
        if (entry) {
          worker.pending.delete(msg.jobId)
          entry.resolve(msg.structures)
        }
        return
      }
      case 'specials_result': {
        const entry = worker.pending.get(msg.jobId)
        if (entry) {
          worker.pending.delete(msg.jobId)
          entry.resolve({ spawn: msg.spawn, strongholds: msg.strongholds })
        }
        return
      }
      case 'biome_at_point_result': {
        const entry = worker.pending.get(msg.jobId)
        if (entry) {
          worker.pending.delete(msg.jobId)
          entry.resolve(msg.biomeId)
        }
        return
      }
      case 'error': {
        const entry = worker.pending.get(msg.jobId)
        if (entry) {
          worker.pending.delete(msg.jobId)
          entry.reject(new Error(msg.error))
        }
        return
      }
    }
  }

  private onError(worker: WorkerHandle, e: ErrorEvent) {
    worker.healthy = false
    worker.errorCount++
    const errMsg = e.message || 'Worker 内部错误'
    // reject 所有 pending 任务
    for (const [id, entry] of worker.pending) {
      entry.reject(new Error('Worker 错误: ' + errMsg))
      worker.pending.delete(id)
    }
    if (worker.seedReject) {
      worker.seedReject(new Error('Worker 错误: ' + errMsg))
      worker.seedResolve = null
      worker.seedReject = null
    }
    if (worker.errorCount > MAX_ERRORS_PER_WORKER) {
      // 重试次数超限：terminate 该 Worker
      try { worker.worker.terminate() } catch { /* ignore */ }
    }
  }
}
