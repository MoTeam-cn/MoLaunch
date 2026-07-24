/**
 * 皮肤与披风管理 API
 */

import { invoke } from '@tauri-apps/api/core'
import type { CachedImage } from './image-cache'

export interface SkinInfo {
  id: string
  state: string
  url: string
  variant: string
  alias: string | null
  /** 缓存 URL（命中缓存时为 cache-image:// 本地 URL，未命中时为远程 URL） */
  cached_url?: string
  /** 是否命中缓存 */
  cached?: boolean
}

export interface CapeInfo {
  id: string
  state: string
  alias: string
  display_name: string
  url: string | null
  /** 缓存 URL（命中缓存时为 cache-image:// 本地 URL，未命中时为远程 URL） */
  cached_url?: string
  /** 是否命中缓存 */
  cached?: boolean
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
 * 获取皮肤 PNG URL（带本地缓存）
 *
 * 可传入 uuid 指定账号（用于预加载非当前账号的皮肤）；
 * 不传则使用当前登录用户。
 *
 * 返回 CachedImage：
 * - cached: true 表示本地缓存命中，URL 为 asset protocol
 * - cached: false 表示远程 URL，后端会异步下载，完成后 emit 'image-cached' 事件
 */
export async function getSkinUrl(uuid?: string): Promise<CachedImage | null> {
  return await invoke<CachedImage | null>('get_skin_url', { uuid: uuid ?? null })
}

/**
 * 获取当前已装备披风的下载 URL（带本地缓存）
 *
 * 返回 CachedImage，同 getSkinUrl
 */
export async function getCapeUrl(): Promise<CachedImage | null> {
  return await invoke<CachedImage | null>('get_cape_url')
}

/**
 * 下载指定 URL 的图片到本地文件
 *
 * 用于"下载当前皮肤到本地"功能：前端已有皮肤 URL（来自 getSkinUrl），
 * 用户选择保存位置后，后端直接从 URL 下载并写入文件。
 */
export async function downloadUrlToFile(url: string, path: string): Promise<void> {
  return await invoke<void>('download_url_to_file', { url, path })
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
