/**
 * 种子地图 WorkerPool 调度层（复刻 minecraftsearch.com，docs/Map/map.md §4.1）
 *
 * 多 Worker 并行生成 tile / 查找结构；prepareSeed 广播给全部 Worker 确认后才返回；
 * 任务经 jobId 关联 Promise。Worker 数 = clamp(4, 0.75*hardwareConcurrency, 16)，低配降到 2。
 * 与原站差异：每 Worker 独立实例化 WASM（不共享 Module），且无 seedEpoch 缓存。
 */
import type {
  GenerateTileParams, FindStructuresParams, SpecialsParams, SpecialsResult,
  BiomeAtPointParams,
  WorkerStructure, WorkerToMainMsg,
} from './types'
import type { WasmBundle } from '@/utils/wasm-loader'

interface JobResolver<T = unknown> {
  resolve: (value?: T) => void
  reject: (err: Error) => void
}

interface WorkerHandle {
  worker: Worker
  healthy: boolean
  /** Worker 已 terminate，不可再派发任务 */
  terminated: boolean
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

// 单个 Worker 最大错误次数：超过后 terminate。
// 设为 50（而非 5）避免 tile 生成偶发失败（WASM 内存增长导致 HEAPU8 detach 等）
// 累积触发 Worker 终止，进而导致所有 Worker 终止后地图全黑无法恢复。
const MAX_ERRORS_PER_WORKER = 50

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
   * @param bundle 主线程缓存的胶水 JS + WASM 二进制（各 Worker 通过 postMessage 共享，不再各自 fetch）
   */
  async init(bundle: WasmBundle): Promise<void> {
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
        terminated: false,
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
      worker.postMessage({
        type: 'init',
        jobId,
        wasmJsUrl: bundle.wasmJsUrl,
        wasmUrl: bundle.wasmUrl,
        wasmJsCode: bundle.jsCode,
        wasmBytes: bundle.wasmBytes,
        seedEpoch: 0,
      })
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
    if (!worker) {
      const terminatedCount = this.workers.filter(w => w.terminated).length
      console.error(`[WorkerPool] 所有 ${this.workers.length} 个 Worker 已终止（terminated=${terminatedCount}），无法派发 ${type} 任务`)
      return Promise.reject(new Error('所有 Worker 已终止，无法派发任务'))
    }
    const jobId = `${type}_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 8)}`
    return new Promise<T>((resolve, reject) => {
      worker.pending.set(jobId, { resolve: resolve as (v: unknown) => void, reject })
      worker.worker.postMessage({ type, jobId, ...(params as object) })
    })
  }

  /** 选一个 Worker 派发任务：优先 healthy && idle，否则 pending 最少。所有 Worker terminated 时返回 null */
  private pickWorker(): WorkerHandle | null {
    let best: WorkerHandle | null = null
    let bestScore = -Infinity
    // 所有 worker 都不健康时的 fallback：选 pending 最少的非 terminated worker
    let fallback: WorkerHandle | null = null
    let fallbackScore = -Infinity
    for (let i = 0; i < this.workers.length; i++) {
      const idx = (this.nextWorkerIdx + i) % this.workers.length
      const w = this.workers[idx]
      if (w.terminated) continue  // 已终止的 worker 不可用
      const score = w.pending.size === 0 ? 1000 : 100 - w.pending.size
      if (w.healthy) {
        if (score > bestScore) {
          bestScore = score
          best = w
          if (score === 1000) {
            this.nextWorkerIdx = (idx + 1) % this.workers.length
            break
          }
        }
      } else if (score > fallbackScore) {
        // 记录最空闲的 unhealthy worker（可能已从瞬时错误恢复）
        fallbackScore = score
        fallback = w
      }
    }
    // 没有 healthy worker 时用最空闲的 unhealthy（worker 可能已恢复但尚未收到消息重置标志）
    // 所有 worker 都 terminated 时返回 null，enqueue 会 reject 而非向死 Worker postMessage
    return best ?? fallback ?? null
  }

  private onMessage(worker: WorkerHandle, msg: WorkerToMainMsg) {
    // 成功收到任何非 error 消息都说明 Worker 恢复了健康（从瞬时错误中恢复）
    // 同时重置 errorCount，避免偶发错误随时间累积触发 terminate
    if (msg.type !== 'error') {
      worker.healthy = true
      worker.errorCount = 0
    }
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
    console.error(`[WorkerPool] Worker#${this.workers.indexOf(worker)} 错误 (#${worker.errorCount}):`, errMsg)
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
      // 重试次数超限：terminate 该 Worker，标记为 terminated 避免再派发任务
      worker.terminated = true
      try { worker.worker.terminate() } catch { /* ignore */ }
    }
  }
}
