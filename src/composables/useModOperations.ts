/**
 * Mod 管理操作 composable（从 ModTab.vue 抽出）
 *
 * 封装 ModTab 子页的全部业务逻辑：
 * - Mod 列表加载 / 过滤搜索
 * - 启用/禁用、删除、安装
 * - 打开 mods 目录 / 打开单个 mod 文件位置
 * - 预加载事件监听（useModsPreload）
 * - 详情查询桥接（useModDetailQuery）
 * - 版本上下文预取（gameVersion / modsDir / disableModUpdate）
 *
 * 设计原则：
 * - 接收所需的 ref/computed 作为参数（selectedId / isModable / modLocalNameStyle）
 * - 返回 handler 函数和状态
 * - handler 内部的 toast/modal 调用保持原 ModTab.vue 行为不变
 * - 模板中的事件绑定保持不变
 */
import { ref, computed, onUnmounted, type Ref, type ComputedRef } from 'vue'
import * as tauri from '@/utils/tauri'
import { toastSuccess, toastError } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { useModsPreload } from '@/composables/useModsPreload'
import { useModDetailQuery } from '@/composables/useModDetailQuery'
import { useMultiSelect } from '@/composables/useMultiSelect'
import { onImageCached } from '@/composables/useImageCache'
import { useTauriEvent } from '@/composables/useTauriEvent'
import { modTitle } from '@/utils/mod-display'
import type { ModInfo } from '@/utils/tauri'

interface UseModOperationsOptions {
  /** 当前选中的版本 ID（来自 useVersionSettings） */
  selectedId: ComputedRef<string | null>
  /** 版本是否可安装 Mod（来自 useVersionSettings 的 isModable computed） */
  isModable: ComputedRef<boolean>
  /** Mod 本地名称显示风格（0=文件名 1=译名 2=译名+文件名，由父组件持有以便其他子组件共用） */
  modLocalNameStyle: Ref<number>
}

/**
 * Mod 管理操作 composable
 *
 * 使用方式：
 * ```ts
 * const { selectedId, isModable } = useVersionSettings()
 * const modLocalNameStyle = ref(0)
 * const { mods, filteredMods, handleToggleMod, init, ... } = useModOperations({
 *   selectedId, isModable, modLocalNameStyle,
 * })
 * onMounted(init)
 * onUnmounted(stopPreloadListener)
 * ```
 */
export function useModOperations(options: UseModOperationsOptions) {
  const { selectedId, isModable, modLocalNameStyle } = options

  const mods = ref<ModInfo[]>([])
  const modsLoading = ref(false)
  const modFilter = ref<'all' | 'enabled' | 'disabled'>('all')
  const modSearch = ref('')
  const isModableVersion = ref(false)
  const checkingModable = ref(false)

  // ===== Mod 更新对话框状态 =====
  /** Mod 更新对话框状态 */
  const updateDialogVisible = ref(false)
  /** 当前要更新/更改的 mod */
  const updateTargetMod = ref<ModInfo | null>(null)
  /**
   * 当前整合包对应的 MC 版本号和 mods 目录路径
   *
   * 在 onMounted 时预取，避免用户点击「详情」按钮后才请求导致卡顿。
   * - gameVersion：传给 ResourceDetail，自动选中顶部筛选 tag
   * - modsDir：传给 ResourceDetail，下载按钮默认保存到此目录
   */
  const versionGameVersion = ref<string | null>(null)
  const versionModsDir = ref<string | null>(null)
  /** 此版本是否禁止更新 Mod（advance_disable_mod_update），开启后 ResourceDetail 下载已存在文件时拦截 */
  const disableModUpdate = ref(false)

  /**
   * 预加载事件监听：后端 `preload_mods_detail_cmd` 批量查询 CF/MR 后，
   * 通过 `mods-preload-update` 事件推送每个 mod 的 project，本 composable 自动更新 mods 数组。
   */
  const { startListener: startPreloadListener, stopListener: stopPreloadListener, isPreloadDone } = useModsPreload(mods)

  /**
   * 图片缓存完成事件监听（参考皮肤/披风 cached_url 刷新机制）
   *
   * 后端 `image_cache::get_image_url` 在缓存未命中时返回远程 URL，并 spawn 异步下载任务。
   * 下载完成后 emit `image-cached` 事件，payload 为 `{ remote_url, local_url }`。
   *
   * 本监听器在 mods 数组中查找 `cached_logo_url === remote_url` 的 mod，
   * 原地替换为 `local_url`（`cache-image://{hash}.png`），触发 Vue 响应式更新，
   * 实现「几秒后图标自动加载出来」的体验（与 PCL2 行为一致）。
   *
   * 注意：`onImageCached` 内部使用 `useTauriEvent`，必须在 setup 同步上下文中调用，
   * 因此放在 composable 顶层（非 init 异步函数内），onUnmounted 自动清理。
   */
  onImageCached((remoteUrl, localUrl) => {
    for (let i = 0; i < mods.value.length; i++) {
      if (mods.value[i].cached_logo_url === remoteUrl) {
        mods.value[i] = { ...mods.value[i], cached_logo_url: localUrl }
      }
    }
  })

  /**
   * Mods 目录文件变化监听（参考 PCL2 PageInstanceMod FileSystemWatcher）
   *
   * 后端 `watch_mods_dir` 在 mods 目录文件变化时 emit `mods-dir-changed` 事件（500ms 防抖），
   * 本监听器收到事件后重新加载 mod 列表（loadMods 内部会合并保留预加载数据），
   * 并重新触发后台预加载（为新加入的 mod 查询 CF/MR 工程详情）。
   *
   * 实现「拖入新 mod → 几秒后自动出现在列表中并加载图标」的体验（与 PCL2 一致）。
   *
   * 注意：`useTauriEvent` 必须在 setup 同步上下文中调用，onUnmounted 自动清理。
   */
  const { start: startModsDirListener } = useTauriEvent('mods-dir-changed', () => {
    // 静默重载：不显示 loading spinner，避免用户操作（toggle/delete/install）后 spinner 闪烁
    loadMods(true)
    // 重新触发预加载（持久化缓存命中的 mod 不会重复联网，新 mod 会查询 CF/MR）
    if (selectedId.value) {
      tauri.preloadModsDetail(selectedId.value).catch(e => {
        console.debug('[ModTab] 文件变化后预加载启动失败:', e)
      })
    }
  })
  startModsDirListener()

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
   * **合并设计**（参考 PCL2 ModList 刷新时保留已加载的工程信息）：
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
    // 保存当前预加载数据（按 enabled_name 匹配，enabled_name 在启用/禁用切换时保持不变）
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
      // 合并：对每个新加载的 mod，如果之前有预加载数据则保留
      mods.value = freshMods.map(mod => {
        const saved = savedData.get(mod.enabled_name)
        return saved ? { ...mod, ...saved } : mod
      })
    } catch (e) {
      toastError('加载 Mod 列表失败', String(e))
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
    // 读取版本独立设置：是否禁止更新 Mod
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

  // ===== 通用多选（使用 useMultiSelect composable，参考 PCL2 PageInstanceMod） =====
  const {
    selectedIds, batchProcessing,
    hasSelection, selectedCount,
    toggle: toggleSelect, selectAll, invertSelection,
    clearSelection, checkSelected, getSelectedItems,
    handleKeydown: handleMultiSelectKeydown,
  } = useMultiSelect<ModInfo>({
    items: filteredMods,
    getId: (mod) => mod.file_name,
  })

  // ===== 按钮可用性判断（参考 PCL2 PageInstanceMod.xaml.vb 第 202-216 行） =====
  // 选中项中是否有已启用的 mod（控制"禁用"按钮）
  const hasEnabledSelected = computed(() =>
    getSelectedItems().some(m => m.is_enabled),
  )
  // 选中项中是否有已禁用的 mod（控制"启用"按钮）
  const hasDisabledSelected = computed(() =>
    getSelectedItems().some(m => !m.is_enabled),
  )
  // 选中项中是否有可更新的 mod（有关联平台工程信息）
  const hasUpdatableSelected = computed(() =>
    getSelectedItems().some(m => m.project),
  )

  /**
   * 启用/禁用 Mod（参考 PCL2 MyLocalModItem.Enable_Click）
   *
   * 核心设计：**原地更新 mod 字段，不重新加载列表**。
   *
   * 原设计（`await loadMods()`）的问题：
   * 1. 列表视觉闪烁刷新
   * 2. 后端排序规则「启用的排前面 + 文件名升序」会导致禁用的 mod 从启用区跳到禁用区末尾，
   *    用户看到的 mod 突然窜到列表最后，体验差
   * 3. 预加载的 `project` 字段全部丢失（list_mods 返回时 project 为空），用户点详情按钮又要等预加载
   *
   * 现设计：后端 toggle_mod 返回新文件名，前端按 file_name 找到对应 mod 原地更新三个字段：
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
      // 原地更新：按 file_name 找到对应 mod，更新字段（用整对象替换确保 Vue 响应式触发）
      const idx = mods.value.findIndex(m => m.file_name === mod.file_name)
      if (idx !== -1) {
        mods.value[idx] = {
          ...mods.value[idx],
          file_name: newFileName,
          is_enabled: enable,
        }
      }
      toastSuccess(enable ? '已启用' : '已禁用', mod.enabled_name)
    } catch (e) {
      toastError('操作失败', String(e))
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
          toastSuccess('Mod 已删除', mod.enabled_name)
          await loadMods()
        } catch (e) {
          toastError('删除失败', String(e))
        }
      },
    )
  }

  async function handleInstallMod() {
    if (!selectedId.value) return
    try {
      const files = await tauri.selectFile('选择要安装的 Mod', [
        { name: 'Mod 文件', extensions: ['jar', 'litemod', 'disabled', 'old'] },
      ])
      if (!files) return
      await tauri.installMod(selectedId.value, files)
      toastSuccess('Mod 安装成功')
      await loadMods()
    } catch (e) {
      toastError('安装失败', String(e))
    }
  }

  async function handleOpenModsDir() {
    if (!selectedId.value) return
    try {
      await tauri.openModsDir(selectedId.value)
    } catch (e) {
      toastError('打开文件夹失败', String(e))
    }
  }

  /** 打开单个 Mod 的文件位置（参考 PCL2 Open_Click） */
  async function handleOpenFile(mod: ModInfo) {
    if (!selectedId.value) return
    try {
      await tauri.revealModFile(selectedId.value, mod.file_name)
    } catch (e) {
      toastError('打开文件位置失败', String(e))
    }
  }

  /** 详情按钮事件桥接：把 mods/isPreloadDone refs 传给 composable（模板中 ref 会自动解包，需在脚本中转发） */
  function onShowInfo(mod: ModInfo) {
    handleShowInfo(mod, mods, isPreloadDone)
  }

  // ===== 批量操作 handler（业务逻辑，使用 useMultiSelect 的选中状态） =====

  /** 批量启用/禁用 */
  async function batchToggle(enable: boolean) {
    if (!selectedId.value || selectedIds.value.size === 0) return
    batchProcessing.value = true
    try {
      const selected = getSelectedItems()
      const toToggle = selected.filter(m => m.is_enabled !== enable)
      let success = 0
      let failed = 0
      for (const mod of toToggle) {
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
          // 同步更新选中集合的 file_name（toggleMod 会重命名文件）
          selectedIds.value.delete(mod.file_name)
          selectedIds.value.add(newFileName)
          success++
        } catch {
          failed++
        }
      }
      selectedIds.value = new Set(selectedIds.value)
      if (failed === 0) {
        toastSuccess(`已${enable ? '启用' : '禁用'} ${success} 个 Mod`)
      } else {
        toastError(`${enable ? '启用' : '禁用'}完成，成功 ${success} 个，失败 ${failed} 个`)
      }
      // 操作完成后自动清空选中（参考 PCL2 PageInstanceMod.xaml.vb 第 465 行 ChangeAllSelected(False)）
      clearSelection()
    } finally {
      batchProcessing.value = false
    }
  }

  /** 批量删除 */
  function batchDelete() {
    if (!selectedId.value || selectedIds.value.size === 0) return
    const count = selectedIds.value.size
    showConfirm(
      '批量删除 Mod',
      `确定要删除选中的 ${count} 个 Mod 吗？此操作不可恢复。`,
      async () => {
        batchProcessing.value = true
        try {
          let success = 0
          let failed = 0
          const toDelete = getSelectedItems()
          for (const mod of toDelete) {
            try {
              await tauri.deleteMod(selectedId.value!, mod.file_name)
              selectedIds.value.delete(mod.file_name)
              success++
            } catch {
              failed++
            }
          }
          await loadMods()
          if (failed === 0) {
            toastSuccess(`已删除 ${success} 个 Mod`)
          } else {
            toastError(`删除完成，成功 ${success} 个，失败 ${failed} 个`)
          }
          // 操作完成后自动清空选中（参考 PCL2 PageInstanceMod.xaml.vb 第 678 行 ChangeAllSelected(False)）
          clearSelection()
        } finally {
          batchProcessing.value = false
        }
      },
    )
  }

  /** 批量更新（打开更新对话框逐个处理） */
  function batchUpdate() {
    if (selectedIds.value.size === 0) return
    const selected = getSelectedItems()
    // 找第一个有 project 信息的可更新 mod
    const updatable = selected.find(m => m.project)
    if (!updatable) {
      toastError('选中的 Mod 没有关联平台信息，无法更新')
      return
    }
    // 打开更新对话框
    updateTargetMod.value = updatable
    updateDialogVisible.value = true
  }

  /** 打开单个 mod 的更新/更改对话框 */
  function openUpdateDialog(mod: ModInfo) {
    if (!mod.project) {
      toastError('此 Mod 没有关联平台信息，无法更新')
      return
    }
    updateTargetMod.value = mod
    updateDialogVisible.value = true
  }

  /** Mod 更新对话框安装完成后的回调 */
  async function onModUpdated() {
    await loadMods()
    // 移除已更新 mod 的选中状态
    if (updateTargetMod.value) {
      selectedIds.value.delete(updateTargetMod.value.file_name)
      selectedIds.value = new Set(selectedIds.value)
    }
  }

  /**
   * onMounted 初始化逻辑：
   * 1. 读取全局配置中的 Mod 本地名称风格
   * 2. 启动预加载事件监听（必须在 loadMods 之前启动，避免错过早期事件）
   * 3. 检查版本是否可安装 Mod
   * 4. 加载 Mod 列表
   * 5. 启动 mods 目录文件监听（拖入新 mod 自动刷新列表）
   * 6. 预取版本上下文（不阻塞 UI）
   * 7. 触发后台预加载（批量查询每个 mod 的 CF/MR 工程详情）
   */
  async function init() {
    try {
      const cfg = await tauri.getConfigMap()
      modLocalNameStyle.value = cfg.communityModLocalNameStyle
    } catch { /* 默认 0 */ }
    // 启动预加载事件监听（必须在 loadMods 之前启动，避免错过早期事件）
    startPreloadListener()
    await checkModable()
    if (isModableVersion.value) {
      await loadMods()
      // 启动 mods 目录文件监听（参考 PCL2 PageInstanceMod FileSystemWatcher）
      // 拖入新 mod / 删除 mod 时自动刷新列表，无需手动按刷新按钮
      if (selectedId.value) {
        tauri.watchModsDir(selectedId.value).catch(e => {
          console.debug('[ModTab] 启动文件监听失败:', e)
        })
      }
      // 预取整合包的 MC 版本号和 mods 目录路径，避免用户点击详情按钮时才请求造成卡顿
      prefetchVersionContext()
      // 触发后台预加载：批量查询每个 mod 的 CF/MR 工程详情
      // 后台异步执行，不阻塞 UI；结果通过 mods-preload-update 事件推送
      if (selectedId.value) {
        tauri.preloadModsDetail(selectedId.value).catch(e => {
          console.debug('[ModTab] 预加载启动失败:', e)
        })
      }
    }
  }

  /**
   * 组件卸载时停止 mods 目录文件监听
   *
   * `useTauriEvent` 的 `mods-dir-changed` 监听器由 onUnmounted 自动清理，
   * 但后端的 `notify` watcher 需要显式调用 `unwatchModsDir` 停止，避免资源泄漏。
   */
  onUnmounted(() => {
    tauri.unwatchModsDir().catch(e => {
      console.debug('[ModTab] 停止文件监听失败:', e)
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
    // 多选状态（来自 useMultiSelect）
    selectedIds,
    batchProcessing,
    hasSelection,
    selectedCount,
    // 按钮可用性判断（参考 PCL2 第 202-216 行）
    hasEnabledSelected,
    hasDisabledSelected,
    hasUpdatableSelected,
    // Mod 更新对话框状态
    updateDialogVisible,
    updateTargetMod,
    // 详情弹窗（来自 useModDetailQuery）
    detailVisible,
    detailProject,
    detailLoadingFor,
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
    handleOpenWiki,
    // 多选操作（来自 useMultiSelect）
    toggleSelect,
    selectAll,
    invertSelection,
    clearSelection,
    checkSelected,
    getSelectedItems,
    handleMultiSelectKeydown,
    // 批量业务 handler
    batchToggle,
    batchDelete,
    batchUpdate,
    openUpdateDialog,
    onModUpdated,
  }
}
