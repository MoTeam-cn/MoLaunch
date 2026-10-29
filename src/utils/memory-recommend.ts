/**
 * 游戏内存推荐
 *
 * 原先的规则是「可用内存的 75%，最多 8GB」，两个方向都不对：
 * 32GB 以上机器照样被 8GB 卡住（200 模组的整合包根本不够），
 * 而 8GB 机器和 64GB 机器拿到同一个上限，档位形同虚设。
 *
 * 这里改成「按总内存分档 + 模组数量加成 + 给系统留出余量」，
 * 并把每一档的理由写出来——用户看到 6144 时应该能知道为什么。
 */

/** 总内存档位对应的基线（MB）：最高档要明显高于旧实现的 8GB 硬顶，否则等于没改 */
const BASE_TIERS: { upToMB: number; baseMB: number }[] = [
  { upToMB: 4096, baseMB: 1024 },
  { upToMB: 8192, baseMB: 2048 },
  { upToMB: 16384, baseMB: 4096 },
  { upToMB: 32768, baseMB: 6144 },
  { upToMB: Number.MAX_SAFE_INTEGER, baseMB: 12288 },
]

/** 每 50 个模组加多少（MB） */
const PER_MODS_STEP_MB = 512
const MODS_PER_STEP = 50
/** 模组加成的上限：再多也该先怀疑整合包本身有问题 */
const MODS_BONUS_CAP_MB = 2048
/** 留给操作系统的下限（MB） */
const SYSTEM_RESERVE_MB = 2048
/** 现代版本的可用下限：低于它游戏起不来 */
const HARD_FLOOR_MB = 1024

export interface MemoryRecommendInput {
  /** 物理内存总量（MB） */
  totalMB: number
  /** 当前可用内存（MB） */
  availableMB: number
  /** 实例模组数量，未知按 0 */
  modCount?: number
}

export interface MemoryRecommendation {
  /** 建议的最大堆（MB） */
  maxMB: number
  /** 建议的最小堆（MB） */
  minMB: number
  /** 为什么是这个数 */
  reason: string
}

/** 按总内存挑基线档位 */
function baseFor(totalMB: number): number {
  const tier = BASE_TIERS.find((item) => totalMB <= item.upToMB)
  return tier ? tier.baseMB : BASE_TIERS[BASE_TIERS.length - 1].baseMB
}

/** 模组数量带来的加成 */
function modsBonus(modCount: number): number {
  if (modCount <= 0) return 0
  const steps = Math.floor(modCount / MODS_PER_STEP)
  return Math.min(steps * PER_MODS_STEP_MB, MODS_BONUS_CAP_MB)
}

/**
 * 算出推荐内存
 *
 * 上限受三重约束：可用内存的 70%、总内存减去系统预留、以及基线加模组加成。
 * 取三者最小，但不会低于硬下限。
 */
export function recommendMemory(input: MemoryRecommendInput): MemoryRecommendation {
  const totalMB = Math.max(0, Math.round(input.totalMB))
  const availableMB = Math.max(0, Math.round(input.availableMB))
  const modCount = Math.max(0, Math.round(input.modCount ?? 0))

  const base = baseFor(totalMB)
  const bonus = modsBonus(modCount)
  const wanted = base + bonus
  const byAvailable = Math.floor(availableMB * 0.7)
  const byReserve = totalMB - SYSTEM_RESERVE_MB

  const candidates = [wanted, byAvailable, byReserve].filter((value) => value > 0)
  const limited = candidates.length > 0 ? Math.min(...candidates) : wanted
  const maxMB = Math.max(HARD_FLOOR_MB, limited)
  const minMB = Math.max(512, Math.round(maxMB / 2))

  const reasonParts = [`按总内存 ${totalMB} MB 取基线 ${base} MB`]
  if (bonus > 0) reasonParts.push(`模组 ${modCount} 个加 ${bonus} MB`)
  if (byAvailable < wanted) reasonParts.push(`当前可用只允许 ${byAvailable} MB`)
  if (byReserve < wanted && byReserve < byAvailable) {
    reasonParts.push(`给系统预留 ${SYSTEM_RESERVE_MB} MB`)
  }
  if (maxMB === HARD_FLOOR_MB && limited < HARD_FLOOR_MB) {
    reasonParts.push(`按硬下限兜底 ${HARD_FLOOR_MB} MB`)
  }

  return { maxMB, minMB, reason: reasonParts.join('，') }
}
