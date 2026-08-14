/**
 * 合成配方生成器 - 导出
 *
 * - 1.13+：数据包 zip（生成文件清单 -> 后端 tools_manager 打包写入）
 * - 1.12：数据包未引入（1.13 起才有），导出单个配方 JSON（前端直接写文件）
 */
import { toolsManager, TOOLS_ACTIONS } from '@/utils/api/tools/core'
import { pickSavePath } from '@/utils/fileDialog'
import { writeTextFile } from '@/utils/api/system'
import type { JavaVersionId } from './types'
import { getJavaVersionMeta } from './versions'
import {
  createDatapackFiles,
  sanitizeRecipeName,
  type DatapackRecipe,
  type DatapackTag,
  type PackFile,
} from './datapack'

export type RecipeExportOptions = {
  version: JavaVersionId
  recipes: DatapackRecipe[]
  tags: DatapackTag[]
  defaultFileName: string
}

export type RecipeExportResult =
  | { ok: true; path: string }
  | { ok: false; reason: 'cancelled' | 'unsupported' | string }

function isDatapackVersion(version: JavaVersionId): boolean {
  const meta = getJavaVersionMeta(version)
  return meta.packFormat !== null && meta.recipeDir !== null && meta.tagDir !== null
}

export async function exportRecipePack(options: RecipeExportOptions): Promise<RecipeExportResult> {
  const { version, recipes, tags, defaultFileName } = options

  if (!isDatapackVersion(version)) {
    if (recipes.length !== 1) {
      return { ok: false, reason: 'unsupported' }
    }
    const path = await pickSavePath({
      title: '导出配方 JSON',
      defaultPath: `${sanitizeRecipeName(recipes[0].name, 'recipe')}.json`,
      filters: [{ name: 'Minecraft 配方 JSON', extensions: ['json'] }],
    })
    if (!path) return { ok: false, reason: 'cancelled' }
    try {
      await writeTextFile(path, JSON.stringify(recipes[0].json, null, 2))
    } catch (err) {
      return { ok: false, reason: String(err) }
    }
    return { ok: true, path }
  }

  let files: PackFile[]
  try {
    files = createDatapackFiles(version, recipes, tags)
  } catch (err) {
    return { ok: false, reason: String(err) }
  }
  const path = await pickSavePath({
    title: '导出数据包',
    defaultPath: `${defaultFileName}.zip`,
    filters: [{ name: 'Minecraft 数据包', extensions: ['zip'] }],
  })
  if (!path) return { ok: false, reason: 'cancelled' }
  try {
    await toolsManager(TOOLS_ACTIONS.RECIPE_GENERATOR_EXPORT, { path, files })
  } catch (err) {
    return { ok: false, reason: String(err) }
  }
  return { ok: true, path }
}
