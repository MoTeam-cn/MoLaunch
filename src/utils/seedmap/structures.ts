/**
 * cubiomes 结构类型配置 + 版本/维度过滤
 *
 * cubiomes finders.h 的 StructureType 枚举值与维度、Java 版引入版本等元数据。
 * 用于 Worker 内 findStructures 遍历可视范围内的 region 查找结构，
 * 以及前端按 MC 版本 + 维度动态过滤可选结构列表。
 *
 * 枚举值来源：src-tauri/cubiomes/finders.h:14-44
 * 版本值来源：src-tauri/cubiomes/biomes.h MCVersion 枚举（与 useSeedMap.ts 的 SEEDMAP_MC_VERSIONS 对齐）
 *
 * queryMode 说明（参考 docs/Map/prompt-structures.md §结构定义）：
 * - 'region'     常规区域结构，由 cubiomes_get_structure_pos 按 regionSize 遍历 region 查找
 * - 'mineshaft'  废弃矿井，cubiomes getStructurePos 内部已统一处理（按 chunk 而非 region），
 *                后端无需特殊分支，前端语义保留以便未来切换到 getMineshafts 批量 API
 * - 'stronghold' 要塞，由 cubiomes_find_strongholds 走 specials 流程统一返回多座
 * - 'slime'      史莱姆区块，cubiomes isSlimeChunk 按 chunk 逐个判断（id=-3 文档约定特殊值，
 *                不影响 cubiomes 调用），handleFindStructures 内遍历可视范围 chunk
 * - 'ravine' / 'mega_ravine' / 'underwater_ravine' / 'mega_underwater_ravine'
 *                峡谷系列，cubiomes checkCanyonStart 原生精确（mega 需 carveCanyon 验证规模）
 * - 'nether_fossil'  下界化石，biome 检查启发式（soul_sand_valley 中心标记）
 * - 'fossil' / 'fossil_diamond'
 *                    化石，biome 检查启发式（desert/swamp/mangrove 中心标记；
 *                    diamond 额外要求深层 deep_dark）
 */
import type { Dimension } from './types'

/** 结构查找模式 */
export type StructureQueryMode =
  | 'region'
  | 'mineshaft'
  | 'stronghold'
  | 'slime'
  | 'ravine'
  | 'mega_ravine'
  | 'underwater_ravine'
  | 'mega_underwater_ravine'
  | 'nether_fossil'
  | 'fossil'
  | 'fossil_diamond'

export interface StructureTypeConfig {
  /** cubiomes StructureType 枚举值（finders.h） */
  id: number
  /** 与 STRUCTURE_ICONS 的 key 对应（src/utils/seedmap/constants.ts） */
  name: string
  /** 该结构所在维度 */
  dimension: Dimension
  /**
   * Java 版引入版本对应的 cubiomes MC 枚举值。
   * 若当前选中版本 < javaSinceValue，则该结构不出现在筛选栏（也无法查找）。
   * 取值参考 cubiomes/biomes.h MCVersion 枚举：
   *   MC_1_7=10, MC_1_8=11, MC_1_9=12, MC_1_10=13, MC_1_11=14,
   *   MC_1_13=16, MC_1_14=17, MC_1_16=20, MC_1_17=21, MC_1_18=22,
   *   MC_1_19=23, MC_1_20=25, MC_1_21=26
   */
  javaSinceValue: number
  /**
   * 查找模式：默认 'region'。
   * - 'region'     → 调 cubiomes_get_structure_pos 遍历 region
   * - 'mineshaft'  → 同 region（cubiomes 内部统一处理），前端语义保留
   * - 'stronghold' → 走 specials 流程（cubiomes_find_strongholds 多座迭代）
   * - 'slime'      → 调 cubiomes_is_slime_chunk 遍历可视范围 chunk
   * - 'ravine' / 'mega_ravine' / 'underwater_ravine' / 'mega_underwater_ravine'
   *                → 调 cubiomes_find_ravines（checkCanyonStart + carveCanyon for mega）
   * - 'nether_fossil' → 调 cubiomes_find_nether_fossils（soul_sand_valley 中心启发式）
   * - 'fossil' / 'fossil_diamond'
   *                   → 调 cubiomes_find_fossils（desert/swamp/mangrove 中心启发式；
   *                      diamond 额外要求深层 deep_dark）
   */
  queryMode?: StructureQueryMode
}

/**
 * 主世界结构清单（cubiomes finders.h 枚举值）
 *
 * 注：Ruined_Portal(11) 在主世界，Ruined_Portal_N(12) 在下界。
 * End_Island(22) 不加入查询（非玩家可访问结构）。
 * Stronghold(25) 走 specials 流程（cubiomes_find_strongholds 多座迭代），queryMode='stronghold'。
 * Mineshaft(15) cubiomes 内部按 chunk 查找，前端语义标注 queryMode='mineshaft'。
 * Slime_Chunks id=-3（文档约定特殊值，不影响 cubiomes 调用），queryMode='slime'，
 *   由 handleFindStructures 遍历可视范围 chunk 调 isSlimeChunk 判断。
 *
 * 扩展结构（prompt-structures.md §结构定义 id 201-223，方案 A）：
 *   212 Ravine, 213 Mega_Ravine, 214 Underwater_Ravine, 215 Mega_Underwater_Ravine
 *     → cubiomes checkCanyonStart 原生精确（mega 通过 carveCanyon poses.size 阈值区分）
 *   221 Fossil, 222 Fossil_Diamond
 *     → biome 检查启发式（desert/swamp/mangrove_swamp 中心；diamond 额外要求深层 deep_dark）
 *   ID 211-215 + 221-222 由前端自定义（不与 cubiomes finders.h 枚举冲突，221/222 在 finders.h
 *   已有 End_Island=22 / End_Gateway=21 等占用，故使用 200+ 段避免冲突）。
 */
export const OVERWORLD_STRUCTURES: StructureTypeConfig[] = [
  { id: 1,  name: 'Desert_Pyramid', dimension: 0, javaSinceValue: 10 }, // 1.3（早于 1.7，所有可选版本均支持）
  { id: 2,  name: 'Jungle_Temple',  dimension: 0, javaSinceValue: 10 }, // 1.3
  { id: 3,  name: 'Swamp_Hut',      dimension: 0, javaSinceValue: 10 }, // 1.4
  { id: 5,  name: 'Village',        dimension: 0, javaSinceValue: 10 }, // 1.0
  { id: 15, name: 'Mineshaft',      dimension: 0, javaSinceValue: 10, queryMode: 'mineshaft' }, // 1.0
  { id: 4,  name: 'Igloo',          dimension: 0, javaSinceValue: 12 }, // 1.9
  { id: 8,  name: 'Monument',       dimension: 0, javaSinceValue: 11 }, // 1.8
  { id: 9,  name: 'Mansion',        dimension: 0, javaSinceValue: 14 }, // 1.11
  { id: 6,  name: 'Ocean_Ruin',     dimension: 0, javaSinceValue: 16 }, // 1.13
  { id: 7,  name: 'Shipwreck',      dimension: 0, javaSinceValue: 16 }, // 1.13
  { id: 14, name: 'Treasure',       dimension: 0, javaSinceValue: 16 }, // 1.13
  { id: 16, name: 'Desert_Well',    dimension: 0, javaSinceValue: 16 }, // 1.13
  { id: 10, name: 'Outpost',        dimension: 0, javaSinceValue: 17 }, // 1.14
  { id: 11, name: 'Ruined_Portal',  dimension: 0, javaSinceValue: 20 }, // 1.16
  { id: 17, name: 'Geode',          dimension: 0, javaSinceValue: 21 }, // 1.17
  { id: 13, name: 'Ancient_City',   dimension: 0, javaSinceValue: 23 }, // 1.19
  { id: 23, name: 'Trail_Ruins',    dimension: 0, javaSinceValue: 25 }, // 1.20
  { id: 24, name: 'Trial_Chambers', dimension: 0, javaSinceValue: 26 }, // 1.21
  { id: 25, name: 'Stronghold',     dimension: 0, javaSinceValue: 10, queryMode: 'stronghold' }, // 1.0（要塞，特殊查找）
  { id: -3, name: 'Slime_Chunks',   dimension: 0, javaSinceValue: 10, queryMode: 'slime' }, // 1.0（史莱姆区块，按 chunk 查找）
  // 扩展结构（方案 A，cubiomes 原生精确 + biome 启发式）
  { id: 212, name: 'Ravine',                 dimension: 0, javaSinceValue: 10, queryMode: 'ravine' }, // 1.0（峡谷，CANYON_CARVER）
  { id: 213, name: 'Mega_Ravine',            dimension: 0, javaSinceValue: 10, queryMode: 'mega_ravine' }, // 1.0（巨型峡谷，carveCanyon poses.size 阈值）
  { id: 214, name: 'Underwater_Ravine',      dimension: 0, javaSinceValue: 20, queryMode: 'underwater_ravine' }, // 1.16（水下峡谷，UNDERWATER_CANYON_CARVER）
  { id: 215, name: 'Mega_Underwater_Ravine', dimension: 0, javaSinceValue: 20, queryMode: 'mega_underwater_ravine' }, // 1.16
  { id: 221, name: 'Fossil',                 dimension: 0, javaSinceValue: 22, queryMode: 'fossil' }, // 1.18（desert/swamp/mangrove 中心启发式）
  { id: 222, name: 'Fossil_Diamond',         dimension: 0, javaSinceValue: 22, queryMode: 'fossil_diamond' }, // 1.18（额外需深层 deep_dark）
]

/** 下界结构 */
export const NETHER_STRUCTURES: StructureTypeConfig[] = [
  { id: 18, name: 'Fortress',       dimension: -1, javaSinceValue: 10 }, // 1.0（下界要塞自 1.0 存在）
  { id: 12, name: 'Ruined_Portal_N',dimension: -1, javaSinceValue: 20 }, // 1.16
  { id: 19, name: 'Bastion',        dimension: -1, javaSinceValue: 20 }, // 1.16
  // 扩展结构（方案 A，biome 启发式）
  { id: 204, name: 'Nether_Fossil', dimension: -1, javaSinceValue: 20, queryMode: 'nether_fossil' }, // 1.16（soul_sand_valley 中心启发式）
]

/** 末地结构 */
export const END_STRUCTURES: StructureTypeConfig[] = [
  { id: 20, name: 'End_City',       dimension: 1, javaSinceValue: 12 }, // 1.9
  { id: 21, name: 'End_Gateway',    dimension: 1, javaSinceValue: 16 }, // 1.13（末地折跃门 1.9 生成，但外岛门 1.13+）
]

/** 按维度获取全部结构清单（不做版本过滤，供 Worker 遍历用） */
export function getStructuresByDimension(dim: Dimension): StructureTypeConfig[] {
  switch (dim) {
    case 0: return OVERWORLD_STRUCTURES
    case -1: return NETHER_STRUCTURES
    case 1: return END_STRUCTURES
  }
}

/**
 * 按 MC 版本 + 维度过滤结构清单（供前端筛选栏显示）
 *
 * Worker 的 findStructures 仍用 getStructuresByDimension（全量遍历），
 * 因为 cubiomes_get_structure_pos 对旧版本调用会返回 0（无结构），安全跳过。
 * 此函数仅控制前端 UI 哪些结构按钮可见，避免显示不可能存在的结构。
 */
export function getStructuresForVersion(mcVersion: number, dim: Dimension): StructureTypeConfig[] {
  return getStructuresByDimension(dim).filter(s => mcVersion >= s.javaSinceValue)
}
