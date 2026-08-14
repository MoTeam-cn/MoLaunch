/**
 * 合成配方生成器 - 数据包结构生成
 *
 * 生成数据包文件清单（pack.mcmeta + 配方 + 自定义标签），打包由后端完成。
 * 1.12 不支持数据包（1.13 起引入），其导出走单文件 JSON。
 */
import type { JavaVersionId, PackFormatVersion } from './types'
import { getJavaVersionMeta } from './versions'

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

export function createPackMcmeta(packFormat: PackFormatVersion): string {
  const pack = Array.isArray(packFormat)
    ? { min_format: packFormat[0], max_format: packFormat[1] }
    : { pack_format: packFormat }
  return JSON.stringify({ pack: { description: PACK_DESCRIPTION, ...pack } }, null, 2)
}

/** 配方文件名安全化（小写 + 仅 [a-z0-9._-]） */
export function sanitizeRecipeName(name: string, fallback: string): string {
  const cleaned = name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/g, '_')
    .replace(/^[._]+|[._]+$/g, '')
  return cleaned || fallback
}

/**
 * 生成数据包文件清单
 * @throws 1.12 等不支持数据包的版本抛错
 */
export function createDatapackFiles(
  version: JavaVersionId,
  recipes: DatapackRecipe[],
  tags: DatapackTag[],
): PackFile[] {
  const meta = getJavaVersionMeta(version)
  if (!meta.packFormat || !meta.recipeDir || !meta.tagDir) {
    throw new Error(`该版本（${version}）不支持数据包导出，请导出单个配方 JSON`)
  }

  const files: PackFile[] = [{ path: 'pack.mcmeta', content: createPackMcmeta(meta.packFormat) }]
  for (const recipe of recipes) {
    files.push({
      path: `data/crafting/${meta.recipeDir}/${recipe.name}.json`,
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
