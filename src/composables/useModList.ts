/**
 * Mod 列表管理 composable（从 useModOperations 拆出）
 *
 * 负责：
 * - Mod 列表加载 / 过滤搜索 / 计数
 * - 单个 Mod 操作：启用/禁用、删除、安装、打开目录、打开文件位置、显示详情
 * - 预加载事件监听（useModsPreload）
 * - 图片缓存事件监听（onImageCached）
 * - mods 目录文件变化监听（onGlobalEvent 'mods-dir-changed'）
 * - 详情查询桥接（useModDetailQuery）
 * - 版本上下文预取（gameVersion / modsDir / disableModUpdate）
 * - onMounted 初始化 + onUnmounted 清理
 *
 * 不负责：
 * - 多选批量操作（见 useModBatchOps）
 * - 更新对话框状态（见 useModUpdateDialog）
 */
import { ref, computed, onUnmounted, type Ref, type ComputedRef } from 'vue'
import * as tauri from '@/utils/tauri'
import { pickFile } from '@/utils/fileDialog'
import { toastSuccess, toastError, toastInfo } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { useModsPreload } from '@/composables/useModsPreload'
import { useModDetailQuery } from '@/composables/useModDetailQuery'
import { onImageCached } from '@/composables/useImageCache'
import { onGlobalEvent } from '@/composables/useGlobalTauriEvent'
import { modTitle } from '@/utils/mod-display'
import type { ModInfo } from '@/utils/tauri'

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
   *
   * 使用全局单例 listener（`onGlobalEvent`），避免 Tauri 2.x `unlisten` 竞态
   * 导致的 "Couldn't find callback id xxx" 警告。
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
   * **合并设计**（刷新时保留已加载的工程信息）：
   * `list_mods` 返回的 mod 元数据字段（project / cached_logo_url / translated_name 等）全为空，
   * 由 `preload_mods_detail` 后台异步补全。当 mods 目录文件变化触发重新加载时，
   * 如果直接覆盖会丢失已加载的预加载数据，导致用户点详情按钮又要等预加载。
   *
   * 解决方案：按 `enabled_name`（在启用/禁用切换时保持不变）保存当前预加载数据，
   * 重新加载后合并回去。新增的 mod 没有保存数据，字段为空，由后续 preload 补全。
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

  const filteredMods = computed(() => {
    let list = mods.value
    if (modFilter.value === 'enabled') list = list.filter(m => m.is_enabled)
    else if (modFilter.value === 'disabled') list = list.filter(m => !m.is_enabled)
    if (modSearch.value.trim()) {
      const q = modSearch.value.toLowerCase()
      list = list.filter(m =>
        m.enabled_name.toLowerCase().includes(q) ||
        m.translated_name.toLowerCase().includes(q),
      )
    }
    return list
  })

  const enabledCount = computed(() => mods.value.filter(m => m.is_enabled).length)
  const disabledCount = computed(() => mods.value.filter(m => !m.is_enabled).length)

  const filterOptions = computed(() => [
    { v: 'all' as const, l: '全部', count: mods.value.length },
    { v: 'enabled' as const, l: '已启用', count: enabledCount.value },
    { v: 'disabled' as const, l: '已禁用', count: disabledCount.value },
  ])

  /**
   * 启用/禁用 Mod
   *
   * 核心设计：**原地更新 mod 字段，不重新加载列表**。
   * 后端 toggle_mod 返回新文件名，前端按 file_name 找到对应 mod 原地更新三个字段：
   * - `file_name`：禁用后变 `xxx.jar.disabled`，启用后变回 `xxx.jar`
   * - `is_enabled`：取反
   * - `enabled_name`：保持不变（永远是去后缀的名称）
   *
   * 这样 mod 在列表中的位置完全不动，project 字段也保留。
   */
  async function handleToggleMod(mod: ModInfo) {
    if (!selectedId.value) return
    const enable = !mod.is_enabled
    try {
      const newFileName = await tauri.toggleMod(selectedId.value, mod.file_name, enable)
      const idx = mods.value.findIndex(m => m.file_name === mod.file_name)
      if (idx !== -1) {
        mods.value[idx] = {
          ...mods.value[idx],
          file_name: newFileName,
          is_enabled: enable,
        }
      }
      toastSuccess(`${enable ? '已启用' : '已禁用'}：${mod.enabled_name}`)
    } catch (e) {
      toastError(`操作失败：${String(e)}`)
    }
  }

  function handleDeleteMod(mod: ModInfo) {
    if (!selectedId.value) return
    showConfirm(
      '删除 Mod',
      `确定要删除 "${modTitle(mod, modLocalNameStyle.value)}" 吗？此操作不可恢复。`,
      async () => {
        try {
          await tauri.deleteMod(selectedId.value!, mod.file_name)
          toastSuccess(`Mod 已删除：${mod.enabled_name}`)
          await loadMods()
        } catch (e) {
          toastError(`删除失败：${String(e)}`)
        }
      },
    )
  }

  async function handleInstallMod() {
    if (!selectedId.value) return
    try {
      const files = await pickFile({
        title: '选择要安装的 Mod',
        filters: [
          { name: 'Mod 文件', extensions: ['jar', 'litemod', 'disabled', 'old'] },
        ],
      })
      if (!files) { toastInfo('已取消安装'); return }
      await tauri.installMod(selectedId.value, files)
      toastSuccess('Mod 安装成功')
      await loadMods()
    } catch (e) {
      toastError(`安装失败：${String(e)}`)
    }
  }

  async function handleOpenModsDir() {
    if (!selectedId.value) return
    try {
      await tauri.openModsDir(selectedId.value)
    } catch (e) {
      toastError(`打开文件夹失败：${String(e)}`)
    }
  }

  /** 打开单个 Mod 的文件位置 */
  async function handleOpenFile(mod: ModInfo) {
    if (!selectedId.value) return
    try {
      await tauri.revealModFile(selectedId.value, mod.file_name)
    } catch (e) {
      toastError(`打开文件位置失败：${String(e)}`)
    }
  }

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
   * 每个 await 后检查 isMounted：组件卸载后不再继续触发 fire-and-forget invoke
   * （特别是 preloadModsDetail 会 tokio::spawn 后台 task 持续 emit 事件，
   * 如果在组件卸载后才触发，会导致 listener 已被清理但 Rust 仍在 emit → callback 丢失警告）
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
   * `onGlobalEvent` 的 handler 由 onUnmounted 自动从 Set 中移除（Tauri listener 永不 unlisten，
   * 避免 `unlisten` 竞态导致的 "Couldn't find callback id xxx" 警告）。
   * 后端的 `notify` watcher 需要显式调用 `unwatchModsDir` 停止，避免资源泄漏。
   * 同时设置 isMounted = false，让 init() 异步链不再继续触发 fire-and-forget invoke。
   *
   * **关键**：调用 `cancelPreloadModsDetail` abort 后台 spawn 的预加载 task，
   * 避免无意义的后台计算（注意：image-cached 事件的 emit 由全局 listener 兜底，
   * 不受此 abort 影响，仍需全局 listener 处理）。
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
