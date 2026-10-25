/**
 * 合成配方生成器 - 版本与配方类型
 */

/** 工具支持的 Java 版本（1.12 ~ 26.2，共 19 个） */
export const JAVA_VERSIONS = [
  '1.12',
  '1.13',
  '1.14',
  '1.15',
  '1.16',
  '1.17',
  '1.18',
  '1.19',
  '1.20',
  '1.21',
  '1.21.2',
  '1.21.4',
  '1.21.5',
  '1.21.6',
  '1.21.7',
  '1.21.9',
  '1.21.11',
  '26.1',
  '26.2',
] as const

export type JavaVersionId = (typeof JAVA_VERSIONS)[number]

/** 配方类型（对应数据包 recipe 目录下的类型名） */
export const RECIPE_TYPES = [
  'crafting',
  'smelting',
  'blasting',
  'smoking',
  'campfire_cooking',
  'stonecutter',
  'smithing',
  'smithing_trim',
  'smithing_transform',
] as const

export type RecipeType = (typeof RECIPE_TYPES)[number]

/** pack.mcmeta 的 pack_format：单值，或 [最小, 最大] 区间 */
export type PackFormatVersion = number | [number, number]
