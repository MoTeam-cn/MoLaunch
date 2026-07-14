/**
 * 皮肤与披风管理 API
 */

import { invoke } from '@tauri-apps/api/core'

export interface SkinInfo {
  id: string
  state: string
  url: string
  variant: string
  alias: string | null
}

export interface CapeInfo {
  id: string
  state: string
  alias: string
  display_name: string
  url: string | null
}

export interface SkinCapeInfo {
  skins: SkinInfo[]
  capes: CapeInfo[]
}

/**
 * 获取当前账号的皮肤/披风信息
 */
export async function getSkinCapeInfo(): Promise<SkinCapeInfo> {
  return await invoke<SkinCapeInfo>('get_skin_cape_info')
}

/**
 * 获取皮肤 PNG 下载 URL
 */
export async function getSkinUrl(): Promise<string | null> {
  return await invoke<string | null>('get_skin_url')
}

/**
 * 下载皮肤 PNG，返回 data:image/png;base64,... 格式
 *
 * 前端收到后用 canvas 裁剪 (8,8,8,8) 区域作为头像（PCL2 的方式）
 */
export async function downloadSkinPng(uuid?: string): Promise<string> {
  return await invoke<string>('download_skin_png', { uuid: uuid ?? null })
}

/**
 * 下载当前已装备披风的 PNG，返回 data:image/png;base64,... 格式
 *
 * 无披风时返回 null
 */
export async function downloadCapePng(): Promise<string | null> {
  return await invoke<string | null>('download_cape_png')
}

/**
 * 将 data URL（如 data:image/png;base64,xxxx）保存到本地文件
 *
 * 用于"下载当前皮肤到本地"：前端已有 dataURL，用户选择保存位置后调用此命令写入
 */
export async function saveDataUrlToFile(dataUrl: string, path: string): Promise<void> {
  return await invoke<void>('save_data_url_to_file', { dataUrl, path })
}

/**
 * 上传/修改皮肤
 * @param filePath PNG 文件本地路径
 * @param variant 'classic' (Steve) 或 'slim' (Alex)
 */
export async function uploadSkin(filePath: string, variant: 'classic' | 'slim'): Promise<void> {
  return await invoke<void>('upload_skin', { filePath, variant })
}

/**
 * 装备披风
 */
export async function equipCape(capeId: string): Promise<void> {
  return await invoke<void>('equip_cape', { capeId })
}

/**
 * 取消披风
 */
export async function unequipCape(): Promise<void> {
  return await invoke<void>('unequip_cape')
}
