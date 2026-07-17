/**
 * 加载器版本数据 composable
 *
 * 负责：
 * - 从后端获取 5 种加载器（Forge/NeoForge/Fabric/OptiFine/LiteLoader）的版本列表
 * - 缓存到 versionStore（避免重复请求）
 * - 提供 computed 版本项列表（供 LoaderCard 组件直接使用）
 *
 * 用法：
 *   const { forgeItems, loadingForge, fetchAll, ... } = useLoaderData(mcVersion, showFlags)
 */
import { ref, computed, type Ref, type ComputedRef } from 'vue'
import * as tauri from '@/utils/tauri'
import { useVersionStore } from '@/stores/version'
import { showError } from '@/utils/modal'

export interface ForgeVersion { version: string; is_recommended: boolean; release_time: string }
export interface NeoforgeVersion { version: string; recommended: boolean }
export interface FabricVersion { version: string; stable: boolean }
export interface OptifineVersion { display_name: string; is_preview: boolean }

export interface LoaderItem {
  key: string
  label: string
  tags: string[]
}

interface ShowFlags {
  forge: ComputedRef<boolean>
  neoforge: ComputedRef<boolean>
  fabric: ComputedRef<boolean>
  optifine: ComputedRef<boolean>
  liteloader: ComputedRef<boolean>
}

/** 通用加载器获取：fetch → 赋值 → 错误提示 → loading=false */
function fetchLoader<T>(
  name: string,
  promise: Promise<T[]>,
  target: Ref<T[]>,
  loading: Ref<boolean>,
) {
  promise.then(v => { target.value = v })
    .catch((e) => {
      console.error(`Failed to load ${name} versions:`, e)
      showError('加载失败', `无法获取 ${name} 版本列表：${e}`)
    })
    .finally(() => { loading.value = false })
}

export function useLoaderData(mcVersion: Ref<string>, flags: ShowFlags) {
  const versionStore = useVersionStore()

  // 原始版本数据
  const forgeVersions = ref<ForgeVersion[]>([])
  const neoforgeVersions = ref<NeoforgeVersion[]>([])
  const fabricVersions = ref<FabricVersion[]>([])
  const optifineVersions = ref<OptifineVersion[]>([])
  const liteloaderVersions = ref<string[]>([])

  // 加载状态
  const loadingForge = ref(true)
  const loadingNeoforge = ref(true)
  const loadingFabric = ref(true)
  const loadingOptifine = ref(true)
  const loadingLiteloader = ref(true)

  // —— 版本项 computed（供 LoaderCard :versions 直接使用） ——

  // OptiFine 先过滤出与当前 MC 版本匹配的
  const filteredOptifine = computed(() =>
    optifineVersions.value.filter(v => {
      const match = v.display_name.match(/^([\d.]+)\s/)
      return match ? match[1] === mcVersion.value : false
    })
  )

  const forgeItems = computed<LoaderItem[]>(() =>
    forgeVersions.value.map((v, i) => ({
      key: v.version,
      label: v.version,
      tags: [
        i === 0 ? '最新版' : null,
        v.is_recommended ? '推荐' : null,
        v.release_time ? `发布于 ${v.release_time}` : null,
      ].filter(Boolean) as string[],
    }))
  )

  const neoforgeItems = computed<LoaderItem[]>(() =>
    [...neoforgeVersions.value].reverse().map(v => ({
      key: v.version,
      label: v.version.split('-')[0],
      tags: v.recommended ? ['推荐'] : v.version.includes('beta') ? ['测试版'] : v.version.includes('alpha') ? ['内测版'] : [],
    }))
  )

  const fabricItems = computed<LoaderItem[]>(() =>
    fabricVersions.value.map((v, i) => ({
      key: v.version,
      label: v.version.split('+')[0],
      tags: i === 0 ? ['最新版'] : [v.stable ? '稳定版' : '测试版'],
    }))
  )

  const optifineItems = computed<LoaderItem[]>(() =>
    filteredOptifine.value.map(v => ({
      key: v.display_name,
      label: v.display_name,
      tags: [v.is_preview ? '测试版' : '正式版'],
    }))
  )

  const liteloaderItems = computed<LoaderItem[]>(() =>
    liteloaderVersions.value.map(v => ({ key: v, label: v, tags: ['稳定版'] }))
  )

  /** 获取所有加载器版本（带缓存） */
  async function fetchAll() {
    // 1. 检查缓存
    const cached = versionStore.getLoaderCache(mcVersion.value)
    if (cached) {
      forgeVersions.value = cached.forge
      neoforgeVersions.value = cached.neoforge
      fabricVersions.value = cached.fabric
      optifineVersions.value = cached.optifine
      liteloaderVersions.value = cached.liteloader
      loadingForge.value = loadingNeoforge.value = loadingFabric.value = loadingOptifine.value = loadingLiteloader.value = false
      return
    }

    // 2. 无缓存，独立请求每个加载器
    if (flags.forge.value) {
      fetchLoader('Forge', tauri.listForgeVersions(mcVersion.value), forgeVersions, loadingForge)
    } else { loadingForge.value = false }

    if (flags.neoforge.value) {
      fetchLoader('NeoForge', tauri.listNeoforgeVersions(mcVersion.value), neoforgeVersions, loadingNeoforge)
    } else { loadingNeoforge.value = false }

    if (flags.fabric.value) {
      fetchLoader('Fabric', tauri.listFabricVersions(), fabricVersions, loadingFabric)
    } else { loadingFabric.value = false }

    if (flags.liteloader.value) {
      fetchLoader('LiteLoader', tauri.listLiteloaderVersions(mcVersion.value), liteloaderVersions, loadingLiteloader)
    } else { loadingLiteloader.value = false }

    if (flags.optifine.value) {
      fetchLoader('OptiFine', tauri.listOptifineVersions(), optifineVersions, loadingOptifine)
    } else { loadingOptifine.value = false }

    // 3. 所有请求完成后缓存
    const checkAllDone = (): Promise<void> => new Promise(resolve => {
      const check = () => {
        if (!loadingForge.value && !loadingNeoforge.value && !loadingFabric.value &&
            !loadingOptifine.value && !loadingLiteloader.value) {
          resolve()
        } else {
          setTimeout(check, 100)
        }
      }
      check()
    })
    checkAllDone().then(() => {
      versionStore.setLoaderCache(mcVersion.value, {
        forge: forgeVersions.value,
        neoforge: neoforgeVersions.value,
        fabric: fabricVersions.value,
        optifine: optifineVersions.value,
        liteloader: liteloaderVersions.value,
      })
    })
  }

  return {
    // 版本项列表
    forgeItems, neoforgeItems, fabricItems, optifineItems, liteloaderItems,
    // 加载状态
    loadingForge, loadingNeoforge, loadingFabric, loadingOptifine, loadingLiteloader,
    // 获取函数
    fetchAll,
  }
}
