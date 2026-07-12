/**
 * 默认皮肤工具
 *
 * 离线账号使用 src/assets/Skins 目录下的官方默认皮肤：
 * Alex, Ari, Efe, Kai, Makena, Noor, Steve, Sunny, Zuri
 *
 * 皮肤选择持久化到注册表（通过后端 set_offline_skin 命令），
 * 前端用内存 Map 缓存，启动时从后端同步。
 */

import { ref } from 'vue'
import { setOfflineSkin } from '@/utils/tauri'

// 导入所有默认皮肤（Vite 会处理为 URL）
import AlexSkin from '@/assets/Skins/Alex.png'
import AriSkin from '@/assets/Skins/Ari.png'
import EfeSkin from '@/assets/Skins/Efe.png'
import KaiSkin from '@/assets/Skins/Kai.png'
import MakenaSkin from '@/assets/Skins/Makena.png'
import NoorSkin from '@/assets/Skins/Noor.png'
import SteveSkin from '@/assets/Skins/Steve.png'
import SunnySkin from '@/assets/Skins/Sunny.png'
import ZuriSkin from '@/assets/Skins/Zuri.png'

export interface DefaultSkinEntry {
  name: string
  url: string
  variant: 'classic' | 'slim'
}

/** 所有可选的默认皮肤 */
export const defaultSkins: DefaultSkinEntry[] = [
  { name: 'Steve', url: SteveSkin, variant: 'classic' },
  { name: 'Alex', url: AlexSkin, variant: 'slim' },
  { name: 'Ari', url: AriSkin, variant: 'slim' },
  { name: 'Efe', url: EfeSkin, variant: 'slim' },
  { name: 'Kai', url: KaiSkin, variant: 'classic' },
  { name: 'Makena', url: MakenaSkin, variant: 'slim' },
  { name: 'Noor', url: NoorSkin, variant: 'slim' },
  { name: 'Sunny', url: SunnySkin, variant: 'slim' },
  { name: 'Zuri', url: ZuriSkin, variant: 'slim' },
]

/**
 * 内存缓存：uuid → 皮肤名称
 *
 * 由 auth store 在 loadOfflineAccounts 后调用 syncOfflineSkins 填充。
 */
const offlineSkinMap = ref<Map<string, string>>(new Map())

/** 从后端同步离线账号皮肤选择到内存 */
export function syncOfflineSkins(accounts: { uuid: string; skin: string | null }[]): void {
  offlineSkinMap.value.clear()
  for (const acc of accounts) {
    if (acc.skin) {
      offlineSkinMap.value.set(acc.uuid, acc.skin)
    }
  }
}

/** 读取离线账号当前选择的皮肤名称（内存缓存），未选择返回 null */
export function getLocalSkinName(uuid: string): string | null {
  return offlineSkinMap.value.get(uuid) ?? null
}

/**
 * 保存离线账号的皮肤选择
 *
 * 同时更新内存缓存和后端注册表。
 */
export async function setLocalSkinName(uuid: string, skinName: string): Promise<void> {
  offlineSkinMap.value.set(uuid, skinName)
  try {
    await setOfflineSkin(uuid, skinName)
  } catch (e) {
    console.error('Failed to persist offline skin:', e)
  }
}

/**
 * 获取离线账号当前应使用的皮肤
 *
 * 优先级：内存缓存（已同步自注册表）> uuid hash 默认
 */
export function getDefaultSkin(uuid: string): string {
  const chosen = offlineSkinMap.value.get(uuid)
  if (chosen) {
    const found = defaultSkins.find(s => s.name === chosen)
    if (found) return found.url
  }
  let hash = 0
  for (const c of uuid) hash = (hash * 31 + c.charCodeAt(0)) | 0
  return defaultSkins[Math.abs(hash) % defaultSkins.length].url
}

/**
 * 获取离线账号当前应使用的皮肤条目（含 variant 信息）
 *
 * 优先级：内存缓存（已同步自注册表）> uuid hash 默认
 */
export function getDefaultSkinEntry(uuid: string): DefaultSkinEntry {
  const chosen = offlineSkinMap.value.get(uuid)
  if (chosen) {
    const found = defaultSkins.find(s => s.name === chosen)
    if (found) return found
  }
  let hash = 0
  for (const c of uuid) hash = (hash * 31 + c.charCodeAt(0)) | 0
  return defaultSkins[Math.abs(hash) % defaultSkins.length]
}

/**
 * 全局皮肤版本号（用于触发 SkinAvatar 重新加载）
 *
 * 当用户在皮肤管理中切换/上传皮肤后递增，SkinAvatar 监听此值变化后自动重新加载。
 */
export const skinVersion = ref(0)

/** 递增全局皮肤版本号，触发所有 SkinAvatar 重新加载 */
export function bumpSkinVersion(): void {
  skinVersion.value++
}
