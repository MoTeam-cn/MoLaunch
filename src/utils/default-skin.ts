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
import { convertFileSrc } from '@tauri-apps/api/core'
import { setOfflineSkin } from '@/utils/tauri'
import { safeCall } from '@/utils/async'

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
  await safeCall(() => setOfflineSkin(uuid, skinName), 'persist offline skin')
}

/**
 * 获取离线账号当前应使用的皮肤
 *
 * 优先级：内存缓存（已同步自注册表）> uuid hash 默认
 */
export function getDefaultSkin(uuid: string): string {
  const chosen = offlineSkinMap.value.get(uuid)
  if (chosen) {
    // 自定义皮肤：通过 convertFileSrc 转换路径
    if (chosen.startsWith('custom:')) {
      const url = parseSkinUrl(chosen)
      if (url) return url
    }
    // 默认皮肤
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
 *
 * 注意：自定义皮肤返回的条目中 url 已通过 convertFileSrc 转换
 */
export function getDefaultSkinEntry(uuid: string): DefaultSkinEntry {
  const chosen = offlineSkinMap.value.get(uuid)
  if (chosen) {
    if (chosen.startsWith('custom:')) {
      const url = parseSkinUrl(chosen)
      if (url) {
        return {
          name: 'custom',
          url,
          variant: parseSkinVariant(chosen),
        }
      }
    }
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

/**
 * 判断给定 MC 版本是否为 1.19.3+
 *
 * 1.19.3+ 有 9 个默认角色（Steve/Alex/Ari/Efe/Kai/Makena/Noor/Sunny/Zuri），
 * 1.19.2 及以前只有 Steve/Alex 两个角色。
 */
export function isVersion1193Plus(version: string): boolean {
  const match = version.match(/^(\d+)\.(\d+)\.(\d+)/)
  if (!match) return false
  const [, major, minor, patch] = match.map(Number)
  return major > 1 || (major === 1 && (minor > 19 || (minor === 19 && patch >= 3)))
}

/**
 * 根据版本获取可选的默认皮肤列表
 *
 * 1.19.3+ 返回全部 9 个皮肤，1.19.2 及以前只返回 Steve/Alex
 */
export function getDefaultSkinsForVersion(version: string): DefaultSkinEntry[] {
  if (isVersion1193Plus(version)) {
    return defaultSkins
  }
  return defaultSkins.filter(s => s.name === 'Steve' || s.name === 'Alex')
}

/**
 * 解析 skin 字段为皮肤 URL（用于 SkinAvatar 显示）
 *
 * - 默认皮肤（"Steve"/"Alex" 等）：返回 defaultSkins 中对应的 URL
 * - 自定义皮肤（"custom:/path|variant"）：返回文件路径（convertFileSrc 由调用方处理）
 * - null/空：返回 null
 */
export function parseSkinUrl(skin: string | null): string | null {
  if (!skin) return null
  if (skin.startsWith('custom:')) {
    // 格式：custom:/path/to/file.png|variant
    const path = skin.slice('custom:'.length).split('|')[0]
    // 使用 Tauri 的 convertFileSrc 将本地文件路径转为可加载的 URL
    return convertFileSrc(path)
  }
  const entry = defaultSkins.find(s => s.name === skin)
  return entry?.url ?? null
}

/**
 * 判断 skin 字段是否为自定义皮肤
 */
export function isCustomSkin(skin: string | null | undefined): boolean {
  return !!skin && skin.startsWith('custom:')
}

/**
 * 从 skin 字段提取变体（classic/slim）
 */
export function parseSkinVariant(skin: string | null): 'classic' | 'slim' {
  if (!skin) return 'classic'
  if (skin.startsWith('custom:')) {
    return skin.includes('|slim') ? 'slim' : 'classic'
  }
  const entry = defaultSkins.find(s => s.name === skin)
  return entry?.variant ?? 'classic'
}
