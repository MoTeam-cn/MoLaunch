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
import { ref, computed, type Ref, type ComputedRef } from 'vue'
import * as tauri from '@/utils/tauri'
import { toastSuccess, toastError } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { useModsPreload } from '@/composables/useModsPreload'
import { useModDetailQuery } from '@/composables/useModDetailQuery'
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

  async function loadMods() {
    if (!selectedId.value) return
    modsLoading.value = true
    try {
      mods.value = await tauri.listMods(selectedId.value)
    } catch (e) {
      toastError('加载 Mod 列表失败', String(e))
      mods.value = []
    } finally {
      modsLoading.value = false
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

  /**
   * onMounted 初始化逻辑：
   * 1. 读取全局配置中的 Mod 本地名称风格
   * 2. 启动预加载事件监听（必须在 loadMods 之前启动，避免错过早期事件）
   * 3. 检查版本是否可安装 Mod
   * 4. 加载 Mod 列表
   * 5. 预取版本上下文（不阻塞 UI）
   * 6. 触发后台预加载（批量查询每个 mod 的 CF/MR 工程详情）
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
  }
}
