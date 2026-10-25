/**
 * JVM 内存与参数建议（纯计算，便于测试）
 *
 * GC 由后端按 Java 版本自动选择（21+ ZGC / 15+ ZGC / 更低 G1GC），
 * 这里只给堆内存建议与与 GC 无关的通用参数，避免和自动选择冲突。
 */

/** 推荐结果 */
export interface JvmRecommendation {
  /** 推荐最大堆（MB，512 的整数倍） */
  memoryMB: number
  /** 追加参数（不含 -Xmx） */
  extraArgs: string[]
  /** 展示用理由 */
  reason: string
}

const MIN_MEMORY_MB = 2048
const MAX_MEMORY_MB = 8192
const STEP_MB = 512

/** 按物理内存给出推荐堆大小与通用参数 */
export function recommendJvm(totalMemoryMB: number): JvmRecommendation {
  const total = Number.isFinite(totalMemoryMB) && totalMemoryMB > 0 ? totalMemoryMB : 0
  if (total === 0) {
    return {
      memoryMB: MIN_MEMORY_MB,
      extraArgs: ['-XX:+AlwaysPreTouch'],
      reason: '未获取到物理内存，先按 2 GB 起，可按实际 Mod 数量微调',
    }
  }

  // 取物理内存的 40%，向 512MB 取整，再夹在 [2G, 8G]
  const raw = Math.floor((total * 0.4) / STEP_MB) * STEP_MB
  const memoryMB = Math.min(MAX_MEMORY_MB, Math.max(MIN_MEMORY_MB, raw))
  const gb = (memoryMB / 1024).toFixed(memoryMB % 1024 === 0 ? 0 : 1)
  return {
    memoryMB,
    extraArgs: ['-XX:+AlwaysPreTouch'],
    reason: `物理内存 ${Math.round(total / 1024)} GB，建议分配 ${gb} GB；GC 已按 Java 版本自动选择`,
  }
}
