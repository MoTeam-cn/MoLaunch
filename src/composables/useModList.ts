/**
 * Mod 列表管理 composable（聚合入口，切片组装）
 *
 * 持有列表状态 + 加载/预加载/详情桥接/生命周期逻辑；
 * 过滤计数在 useModList/query.ts，单项操作在 useModList/item-ops.ts。
 */
import { ref, onUnmounted, type Ref, type ComputedRef } from 'vue'
import * as tauri from '@/utils/tauri'
import { toastSuccess, toastError } from '@/utils/toast'
import { useModsPreload } from '@/composables/useModsPreload'
import { useModDetailQuery } from '@/composables/useModDetailQuery'
import { onImageCached } from '@/composables/useImageCache'
import { onGlobalEvent } from '@/composables/useGlobalTauriEvent'
import type { ModInfo } from '@/utils/tauri'
import { useModListQuery } from './useModList/query'
import { useModListItemOps } from './useModList/item-ops'

export interface UseModListOptions {
  /** 当前选中的版本 ID（来自 useVersionSettings） */
  selectedId: ComputedRef<string | null>
  /** 版本是否可安装 Mod（来自 useVersionSettings 的 isModable computed） */
  isModable: ComputedRef<boolean>
  /** Mod 本地名称显示风格（0=文件名 1=译名 2=译名+文件名，由父组件持有以便其他子组件共用） */
  modLocalNameStyle: Ref<number>
}

export function useModList(options: UseModListOptions) {
  const { selectedId, isModable, modLocalNameStyle } = options

  const mods = ref<ModInfo[]>([])
  const modsLoading = ref(false)
  const modFilter = ref<'all' | 'enabled' | 'disabled'>('all')
  const modSearch = ref('')
  const isModableVersion = ref(false)
  const checkingModable = ref(false)
  /** 组件是否仍挂载（init() 异步链中检查，卸载后不再触发 fire-and-forget invoke） */
  let isMounted = true

  // 版本上下文（详情弹窗 + ResourceDetail 使用）
  const versionGameVersion = ref<string | null>(null)
  const versionModsDir = ref<string | null>(null)
  const disableModUpdate = ref(false)

  // 预加载事件监听
  const { startListener: startPreloadListener, stopListener: stopPreloadListener, isPreloadDone } = useModsPreload(mods)

  /**
   * 图片缓存完成事件监听
   *
   * 后端 `image_cache::get_image_url` 在缓存未命中时返回远程 URL，并 spawn 异步下载任务。
   * 下载完成后 emit `image-cached` 事件，payload 为 `{ remote_url, local_url }`。
   * 本监听器在 mods 数组中查找 `cached_logo_url === remote_url` 的 mod，原地替换为 local_url。
   */
  onImageCached((remoteUrl, localUrl) => {
    for (let i = 0; i < mods.value.length; i++) {
      if (mods.value[i].cached_logo_url === remoteUrl) {
        mods.value[i] = { ...mods.value[i], cached_logo_url: localUrl }
      }
    }
  })

  /**
   * Mods 目录文件变化监听
   *
   * 后端 `watch_mods_dir` 在 mods 目录文件变化时 emit `mods-dir-changed` 事件（500ms 防抖），
   * 本监听器收到事件后重新加载 mod 列表（loadMods 内部会合并保留预加载数据），
   * 并重新触发后台预加载（为新加入的 mod 查询 CF/MR 工程详情）。
   */
  onGlobalEvent('mods-dir-changed', () => {
    loadMods(true)
    if (selectedId.value) {
      tauri.preloadModsDetail(selectedId.value).catch(e => {
        console.debug('[ModTab] 文件变化后预加载启动失败:', e)
      })
    }
  })

  // 详情查询桥接
  const { detailVisible, detailProject, detailLoadingFor, handleShowInfo, handleOpenWiki } = useModDetailQuery()

  // 过滤 / 搜索 / 计数切片
  const { filteredMods, filterOptions } = useModListQuery({ mods, modFilter, modSearch })

  async function checkModable() {
    if (!selectedId.value) { isModableVersion.value = false; return }
    checkingModable.value = true
    try {
      isModableVersion.value = await tauri.isVersionModable(selectedId.value)
    } catch {
      isModableVersion.value = isModable.value
    } finally {
      checkingModable.value = false
    }
  }

  /**
   * 加载 Mod 列表（合并预加载数据）
   *
   * `list_mods` 返回的 mod 元数据字段（project / cached_logo_url / translated_name 等）全为空，
   * 由 `preload_mods_detail` 后台异步补全。重新加载时按 `enabled_name` 合并已加载的预加载数据，
   * 避免刷新后丢失工程信息。
   *
   * @param silent 静默模式（文件监听触发时使用）：不设置 modsLoading，避免 spinner 闪烁
   */
  async function loadMods(silent = false) {
    if (!selectedId.value) return
    const savedData = new Map<string, Partial<ModInfo>>()
    for (const mod of mods.value) {
      savedData.set(mod.enabled_name, {
        project: mod.project,
        cached_logo_url: mod.cached_logo_url,
        translated_name: mod.translated_name,
        description: mod.description,
        version: mod.version,
        slug: mod.slug,
      })
    }
    if (!silent) modsLoading.value = true
    try {
      const freshMods = await tauri.listMods(selectedId.value)
      mods.value = freshMods.map(mod => {
        const saved = savedData.get(mod.enabled_name)
        return saved ? { ...mod, ...saved } : mod
      })
      if (!silent) toastSuccess('Mod 列表已刷新')
    } catch (e) {
      toastError(`加载 Mod 列表失败：${String(e)}`)
      mods.value = []
    } finally {
      if (!silent) modsLoading.value = false
    }
  }

  /** 预取整合包的 MC 版本号和 mods 目录（不阻塞 UI） */
  async function prefetchVersionContext() {
    if (!selectedId.value) return
    try {
      versionGameVersion.value = await tauri.getVersionGameVersion(selectedId.value)
    } catch (e) {
      console.debug('[ModTab] 获取版本号失败:', e)
      versionGameVersion.value = null
    }
    try {
      versionModsDir.value = await tauri.getVersionModsDir(selectedId.value)
    } catch (e) {
      console.debug('[ModTab] 获取 mods 目录失败:', e)
      versionModsDir.value = null
    }
    try {
      const p = await tauri.getVersionPersonalization(selectedId.value)
      disableModUpdate.value = p.advanceDisableModUpdate
    } catch (e) {
      console.debug('[ModTab] 获取禁止更新 Mod 配置失败:', e)
      disableModUpdate.value = false
    }
  }

  // 单项操作切片（启用/禁用、删除、安装、打开目录、打开文件位置）
  const { handleToggleMod, handleDeleteMod, handleInstallMod, handleOpenModsDir, handleOpenFile } = useModListItemOps({
    selectedId,
    modLocalNameStyle,
    mods,
    loadMods,
  })

  /** 详情按钮事件桥接：把 mods/isPreloadDone refs 传给 composable（模板中 ref 会自动解包，需在脚本中转发） */
  function onShowInfo(mod: ModInfo) {
    handleShowInfo(mod, mods, isPreloadDone)
  }

  /**
   * onMounted 初始化逻辑：
   * 1. 读取全局配置中的 Mod 本地名称风格
   * 2. 启动预加载事件监听（必须在 loadMods 之前启动，避免错过早期事件）
   * 3. 检查版本是否可安装 Mod
   * 4. 加载 Mod 列表
   * 5. 启动 mods 目录文件监听
   * 6. 预取版本上下文（不阻塞 UI）
   * 7. 触发后台预加载
   *
   * 每个 await 后检查 isMounted：组件卸载后不再继续触发 fire-and-forget invoke。
   */
  async function init() {
    try {
      const cfg = await tauri.getConfigMap()
      if (!isMounted) return
      modLocalNameStyle.value = cfg.communityModLocalNameStyle
    } catch { /* 默认 0 */ }
    startPreloadListener()
    await checkModable()
    if (!isMounted) return
    if (isModableVersion.value) {
      await loadMods()
      if (!isMounted) return
      if (selectedId.value) {
        tauri.watchModsDir(selectedId.value).catch(e => {
          console.debug('[ModTab] 启动文件监听失败:', e)
        })
      }
      prefetchVersionContext()
      if (selectedId.value) {
        tauri.preloadModsDetail(selectedId.value).catch(e => {
          console.debug('[ModTab] 预加载启动失败:', e)
        })
      }
    }
  }

  /**
   * 组件卸载时停止 mods 目录文件监听 + 取消后台预加载 task
   *
   * `onGlobalEvent` 的 handler 由 onUnmounted 自动从 Set 中移除（Tauri listener 永不 unlisten）。
   * 后端的 `notify` watcher 需要显式调用 `unwatchModsDir` 停止，避免资源泄漏。
   * 同时设置 isMounted = false，让 init() 异步链不再继续触发 fire-and-forget invoke。
   */
  onUnmounted(() => {
    isMounted = false
    tauri.unwatchModsDir().catch(e => {
      console.debug('[ModTab] 停止文件监听失败:', e)
    })
    tauri.cancelPreloadModsDetail().catch(e => {
      console.debug('[ModTab] 取消预加载失败:', e)
    })
  })

  return {
    // 状态
    mods,
    modsLoading,
    modFilter,
    modSearch,
    isModableVersion,
    checkingModable,
    versionGameVersion,
    versionModsDir,
    disableModUpdate,
    isPreloadDone,
    // 详情弹窗
    detailVisible,
    detailProject,
    detailLoadingFor,
    handleOpenWiki,
    // computed
    filteredMods,
    filterOptions,
    // 生命周期
    startPreloadListener,
    stopPreloadListener,
    init,
    // handler
    checkModable,
    loadMods,
    prefetchVersionContext,
    handleToggleMod,
    handleDeleteMod,
    handleInstallMod,
    handleOpenModsDir,
    handleOpenFile,
    onShowInfo,
  }
}
