/**
 * 合成配方生成器 - 数据包文件清单（对应 specs/cleanroom/recipe-generator.md §3.7 §3.8 §4）
 *
 * 目录名与 pack_format 全部取自版本能力表；1.12 无数据包，调用方需走单文件导出。
 */
import { getJavaVersionMeta } from './versions'
import type { JavaVersionId, PackFormatVersion } from './types'

export type PackFile = {
  path: string
  content: string
}

export type DatapackRecipe = {
  name: string
  json: Record<string, unknown>
}

export type DatapackTag = {
  namespace: string
  id: string
  values: string[]
}

const PACK_DESCRIPTION = 'Generated with MoLaunch Recipe Generator'
const DEFAULT_NAMESPACE = 'crafting'
const FALLBACK_NAME = 'recipe'

export function createPackMcmeta(packFormat: PackFormatVersion): string {
  const range = Array.isArray(packFormat)
    ? { min_format: packFormat[0], max_format: packFormat[1] }
    : { pack_format: packFormat }
  return JSON.stringify({ pack: { description: PACK_DESCRIPTION, ...range } }, null, 2)
}

/** 配方文件名安全化：小写、只留 `[a-z0-9._-]`，首尾标点去掉 */
export function sanitizeRecipeName(name: string, fallback: string = FALLBACK_NAME): string {
  const cleaned = name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/g, '_')
    .replace(/^[._]+|[._]+$/g, '')
  return cleaned || fallback
}

export function supportsDatapackExport(version: JavaVersionId): boolean {
  const meta = getJavaVersionMeta(version)
  return meta.packFormat !== null && meta.recipeDir !== null && meta.tagDir !== null
}

/** 生成数据包文件清单（pack.mcmeta + 配方 + 自定义标签） */
export function createDatapackFiles(
  version: JavaVersionId,
  recipes: DatapackRecipe[],
  tags: DatapackTag[],
): PackFile[] {
  const meta = getJavaVersionMeta(version)
  if (meta.packFormat === null || meta.recipeDir === null || meta.tagDir === null) {
    throw new Error(`该版本（${version}）不支持数据包导出，请导出单个配方 JSON`)
  }

  const files: PackFile[] = [{ path: 'pack.mcmeta', content: createPackMcmeta(meta.packFormat) }]

  for (const recipe of recipes) {
    files.push({
      path: `data/${DEFAULT_NAMESPACE}/${meta.recipeDir}/${sanitizeRecipeName(recipe.name)}.json`,
      content: JSON.stringify(recipe.json, null, 2),
    })
  }

  for (const tag of tags) {
    files.push({
      path: `data/${tag.namespace}/${meta.tagDir}/${tag.id}.json`,
      content: JSON.stringify({ replace: false, values: tag.values }, null, 2),
    })
  }

  return files
}
