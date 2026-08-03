/**
 * 种子地图配置常量 + MC 版本映射（从 useSeedMap.ts 抽取，无闭包依赖）
 *
 * Zoom 体系（与原站对齐）：0~10（11 级），RESOLUTIONS=[256..0.25]；tile 64×64px；
 * cubiomes scale ∈ {1,4,16,64,256}，每级取 ≤bpp 的最大 scale 生成 biome。
 */

// ===== Zoom level 配置（与原站对齐） =====
// MIN_ZOOM=3 防止过度缩小：zoom 0~2 时单 tile 覆盖 4K~16K 方块，viewport tile 极少，
// 已加载区块外的区域为空 bitmap 导致观感"黑屏"。zoom 3（bpp=32）下 tile 生成快且覆盖合理。
export const MIN_ZOOM = 3
// MAX_ZOOM=10 对应 resolution 0.25（4 像素/方块），与原站 minecraftsearch.com 对齐。
export const MAX_ZOOM = 10
export const TILE_SIZE = 64  // 原站用 64，不是 256
// 11 级 resolution：从 256 bpp（最远）到 0.25 bpp（最近，4 像素一个方块）
export const RESOLUTIONS = [256, 128, 64, 32, 16, 8, 4, 2, 1, 0.5, 0.25]

// 投影 extent：约 ±3e7 方块
// EXTENT_HALF 必须是最大 blocksPerTile（64×256=16384=2^14）的整数倍，
// 确保所有 zoom 级别的 tile 边界与 extent 完全对齐，避免相邻 tile 内容不连续
export const EXTENT_HALF = 29_999_104  // 16384 × 1831 = 2^14 × 1831
export const EXTENT = [-EXTENT_HALF, -EXTENT_HALF, EXTENT_HALF, EXTENT_HALF]

// 低 zoom（< 4）时可视范围过大，findStructures 遍历 region 数可达数百万，
// 会长时间阻塞 Worker 串行队列导致 tile 生成饿死（黑屏/无法继续加载）。
// 低 zoom 时隐藏结构图层并跳过查找，高 zoom 时恢复。
export const STRUCT_MIN_ZOOM = 4

/**
 * MC 版本列表（与 cubiomes MC_* 枚举值映射；见 cubiomes/biomes.h:5-46）
 *
 * 版本支持说明：
 * - 使用 fork 仓库 https://github.com/MoTeam-cn/cubiomes，原生支持 MC_26_2 (=34)
 * - 枚举值（biomes.h）：
 *   MC_1_21_WD=28 (1.21.4), MC_1_21_5=29, MC_1_21_6=30, MC_1_21_9=31,
 *   MC_1_21_11=32 (=MC_1_21), MC_26_1=33, MC_26_2=34 (=MC_NEWEST)
 * - 1.21.9/1.21.11 共用 31（cubiomes 未单独定义 1.21.11 枚举，1.21.9=31 已涵盖）
 */
export const SEEDMAP_MC_VERSIONS = [
  // Latest（fork cubiomes 原生支持 MC_26_2）
  { label: '26.2', value: 28 },
  { label: '26.1', value: 27 },
  { label: '1.21.9', value: 25 },
  { label: '1.21.6', value: 24 },
  { label: '1.21.5', value: 23 },
  { label: '1.21.4', value: 22 },
  { label: '1.21.3', value: 21 },
  { label: '1.21.1', value: 20 },
  { label: '1.20', value: 19 },
  { label: '1.19.4', value: 18 },
  { label: '1.19.2', value: 17 },
  { label: '1.18', value: 16 },
  // Old
  { label: '1.17', value: 15 }, { label: '1.16', value: 14 },
  { label: '1.15', value: 12 }, { label: '1.14', value: 11 },
  { label: '1.13', value: 10 }, { label: '1.12', value: 9 },
  { label: '1.11', value: 8 }, { label: '1.10', value: 7 },
  { label: '1.9', value: 6 }, { label: '1.8', value: 5 },
  { label: '1.7', value: 4 },
] as const

/**
 * 将 MC 版本号字符串（如 "1.21.5"、"1.20"、"26.2"）映射到 seedmap 支持的最近 cubiomes 枚举值。
 *
 * 规则：在所有 ≤ 目标版本的 seedmap 版本中取最大；若全大于目标（如 1.5），取最老版本。
 * 版本比较按 "." 分段转数字逐段比较（"1.21.5" → [1,21,5]）。
 *
 * 用于"从存档加载"功能：存档版本可能不在 seedmap 支持列表中（如 1.21.7），
 * 自动降级到最近的受支持版本。
 *
 * @returns 匹配的 cubiomes 枚举值；无法解析时返回 null
 */
export function mapMcVersionToCubiomes(mcVersion: string): number | null {
  const parseVer = (s: string): number[] => {
    const parts = s.split('.').map((p) => parseInt(p, 10))
    return parts.some((n) => Number.isNaN(n)) ? [] : parts
  }
  const cmp = (a: number[], b: number[]): number => {
    const len = Math.max(a.length, b.length)
    for (let i = 0; i < len; i++) {
      const av = a[i] ?? 0
      const bv = b[i] ?? 0
      if (av !== bv) return av - bv
    }
    return 0
  }
  const target = parseVer(mcVersion)
  if (target.length === 0) return null

  // 优先精确匹配 label
  const exact = SEEDMAP_MC_VERSIONS.find((v) => cmp(parseVer(v.label), target) === 0)
  if (exact) return exact.value

  // 降级：取 ≤ target 中最大的；若无则取最老版本
  const le = SEEDMAP_MC_VERSIONS
    .map((v) => ({ v, parts: parseVer(v.label) }))
    .filter((x) => cmp(x.parts, target) <= 0)
    .sort((x, y) => cmp(x.parts, y.parts))
  if (le.length > 0) return le[le.length - 1].v.value
  return SEEDMAP_MC_VERSIONS[SEEDMAP_MC_VERSIONS.length - 1].value
}
