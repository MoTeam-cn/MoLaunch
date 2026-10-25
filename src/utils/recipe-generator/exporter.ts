/**
 * 合成配方生成器 - 导出
 *
 * - 1.13+：数据包 zip（前端生成文件清单，后端 tools_manager 打包）
 * - 1.12：无数据包，导出单个配方 JSON
 */
import { toolsManager, TOOLS_ACTIONS } from '@/utils/api/tools/core'
import { pickSavePath } from '@/utils/fileDialog'
import { writeTextFile } from '@/utils/api/system'
import {
  createDatapackFiles,
  sanitizeRecipeName,
  supportsDatapackExport,
  type DatapackRecipe,
  type DatapackTag,
  type PackFile,
} from './datapack'
import type { JavaVersionId } from './types'

export type RecipeExportOptions = {
  version: JavaVersionId
  recipes: DatapackRecipe[]
  tags: DatapackTag[]
  defaultFileName: string
}

export type RecipeExportResult =
  | { ok: true; path: string }
  | { ok: false; reason: 'cancelled' | 'unsupported' | string }

/** 1.12 等无数据包版本：导出单个配方 JSON */
async function exportSingleRecipe(recipe: DatapackRecipe): Promise<RecipeExportResult> {
  const path = await pickSavePath({
    title: '导出配方 JSON',
    defaultPath: `${sanitizeRecipeName(recipe.name)}.json`,
    filters: [{ name: 'Minecraft 配方 JSON', extensions: ['json'] }],
  })
  if (!path) return { ok: false, reason: 'cancelled' }
  try {
    await writeTextFile(path, JSON.stringify(recipe.json, null, 2))
  } catch (error) {
    return { ok: false, reason: String(error) }
  }
  return { ok: true, path }
}

export async function exportRecipePack(options: RecipeExportOptions): Promise<RecipeExportResult> {
  const { version, recipes, tags, defaultFileName } = options

  if (!supportsDatapackExport(version)) {
    if (recipes.length !== 1) return { ok: false, reason: 'unsupported' }
    return exportSingleRecipe(recipes[0])
  }

  let files: PackFile[]
  try {
    files = createDatapackFiles(version, recipes, tags)
  } catch (error) {
    return { ok: false, reason: String(error) }
  }

  const path = await pickSavePath({
    title: '导出数据包',
    defaultPath: `${defaultFileName}.zip`,
    filters: [{ name: 'Minecraft 数据包', extensions: ['zip'] }],
  })
  if (!path) return { ok: false, reason: 'cancelled' }

  try {
    await toolsManager(TOOLS_ACTIONS.RECIPE_GENERATOR_EXPORT, { path, files })
  } catch (error) {
    return { ok: false, reason: String(error) }
  }
  return { ok: true, path }
}
