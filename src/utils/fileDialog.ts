/**
 * 文件 / 文件夹选择对话框封装
 *
 * 基于 @tauri-apps/plugin-dialog，提供类型安全的打开 / 保存对话框。
 * 用户取消选择时返回 null（不抛错），业务侧判空即可。
 */

import { open, save } from '@tauri-apps/plugin-dialog'

/** 打开文件选择对话框，返回选中文件路径（取消返回 null） */
export async function pickFile(options?: {
  /** 文件扩展名过滤器，如 [{ name: 'ZIP', extensions: ['zip'] }] */
  filters?: { name: string; extensions: string[] }[]
  /** 默认打开目录 */
  defaultPath?: string
  /** 对话框标题 */
  title?: string
}): Promise<string | null> {
  const result = await open({
    multiple: false,
    directory: false,
    title: options?.title,
    defaultPath: options?.defaultPath,
    filters: options?.filters,
  })
  // open 在 multiple:false 时返回 string | null
  return typeof result === 'string' ? result : null
}

/** 打开文件夹选择对话框，返回选中文件夹路径（取消返回 null） */
export async function pickDirectory(options?: {
  defaultPath?: string
  title?: string
}): Promise<string | null> {
  const result = await open({
    multiple: false,
    directory: true,
    title: options?.title,
    defaultPath: options?.defaultPath,
  })
  return typeof result === 'string' ? result : null
}

/** 打开保存对话框，返回保存路径（取消返回 null） */
export async function pickSavePath(options?: {
  /** 文件扩展名过滤器 */
  filters?: { name: string; extensions: string[] }[]
  /** 默认文件名 */
  defaultPath?: string
  /** 对话框标题 */
  title?: string
}): Promise<string | null> {
  return await save({
    title: options?.title,
    defaultPath: options?.defaultPath,
    filters: options?.filters,
  })
}
