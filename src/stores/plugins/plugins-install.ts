/**
 * 插件 store - 外部插件安装 / 状态管理切片
 *
 * 外部插件清单加载、安装/卸载、后端同步、启用状态与钩子触发。
 * 共享状态 ref 由主 store 创建并注入，本切片不新建重复状态。
 */
import { computed } from 'vue'
import type { Ref } from 'vue'
import type {
  PluginManifest,
  PluginRuntimeState,
  HomePanelMode,
  CustomLayoutConfig,
} from '@/types/plugin'
import { builtinPlugins } from '@/plugins'
import {
  listExternalPlugins,
  installExternalPluginFromDir,
  installExternalPluginFromZip,
  uninstallExternalPlugin,
  type ExternalPluginEntry,
} from '@/utils/api/plugins'
import {
  DEFAULT_CUSTOM_LAYOUT,
  externalManifestToPluginManifest,
  loadPersonalizationData,
  fetchCustomLayoutContent,
  isValidHomePanelMode,
} from '@/utils/pluginInstaller'
import { safeCall } from '@/utils/async'

export interface PluginsInstallDeps {
  /** 已注册的插件清单（内置 + 外部） */
  manifests: Ref<PluginManifest[]>
  /** 插件运行时状态（id → state） */
  runtimeStates: Ref<Record<string, PluginRuntimeState>>
  /** 外部插件原始清单（保留 entry / permissions 供沙箱使用） */
  externalPluginsRaw: Ref<ExternalPluginEntry[]>
  /** 是否已从后端同步 */
  backendSynced: Ref<boolean>
  /** 主页右侧内容区显示模式 */
  homePanelMode: Ref<HomePanelMode>
  /** 自定义布局配置 */
  customLayoutConfig: Ref<CustomLayoutConfig>
  /** 持久化到后端 AppData（由主 store 提供） */
  persistToBackend: () => Promise<void>
}

export function usePluginsInstallSlice(deps: PluginsInstallDeps) {
  const { manifests, runtimeStates, externalPluginsRaw, backendSynced, homePanelMode, customLayoutConfig, persistToBackend } = deps

  /** 已启用的插件清单（计算属性） */
  const enabledPlugins = computed(() =>
    manifests.value.filter((m) => runtimeStates.value[m.id]?.enabled),
  )

  /** 内置插件列表 */
  const builtinPluginList = computed(() => manifests.value.filter((m) => m.builtin))

  /** 外部插件列表 */
  const externalPluginList = computed(() => manifests.value.filter((m) => !m.builtin))

  /**
   * 初始化运行时状态
   *
   * 为每个已注册插件创建默认运行时状态（内置默认启用，外部默认禁用）。
   * 已存在的状态保留（避免外部插件加载后丢失 AppData 偏好）。
   */
  function initRuntimeStates() {
    const states: Record<string, PluginRuntimeState> = {}
    for (const m of manifests.value) {
      if (runtimeStates.value[m.id]) {
        states[m.id] = runtimeStates.value[m.id]
        continue
      }
      states[m.id] = {
        id: m.id,
        enabled: m.builtin, // 内置默认启用，外部默认禁用
        builtin: m.builtin,
        lastError: null,
      }
    }
    runtimeStates.value = states
  }

  /**
   * 加载外部插件（启动后异步调用）
   *
   * 扫描 `<base_dir>/plugins/` 目录，读取每个插件的 manifest.json，
   * 转换为 PluginManifest 后合并到 manifests 数组。
   */
  async function loadExternalPlugins() {
    try {
      const entries = await listExternalPlugins()
      externalPluginsRaw.value = entries

      // 合并内置 + 外部插件清单
      const externalManifests = entries.map(externalManifestToPluginManifest)
      manifests.value = [...builtinPlugins, ...externalManifests]

      // 为新加载的外部插件初始化运行时状态（保留已有状态）
      initRuntimeStates()
    } catch (e) {
      console.warn('[Plugins] Failed to load external plugins:', e)
    }
  }

  /**
   * 从源目录安装外部插件
   *
   * 安装后重新加载外部插件清单。返回安装后的插件 ID。
   */
  async function installFromDir(sourceDir: string): Promise<string> {
    const pluginId = await installExternalPluginFromDir(sourceDir)
    await loadExternalPlugins()
    return pluginId
  }

  /**
   * 从 ZIP 文件路径安装外部插件
   *
   * 安装后重新加载外部插件清单。返回安装后的插件 ID。
   */
  async function installFromZip(zipPath: string): Promise<string> {
    const pluginId = await installExternalPluginFromZip(zipPath)
    await loadExternalPlugins()
    return pluginId
  }

  /**
   * 卸载外部插件
   *
   * 卸载后重新加载外部插件清单，并清理运行时状态。
   */
  async function uninstallExternal(pluginId: string): Promise<void> {
    // 若卸载的是当前 homePanelMode 对应的插件，回退到 default
    if (homePanelMode.value === `plugin:${pluginId}`) {
      homePanelMode.value = 'default'
    }
    await uninstallExternalPlugin(pluginId)
    await loadExternalPlugins()
    await persistToBackend()
  }

  /**
   * 从后端 AppData 同步个性化配置（启动后异步调用一次）
   *
   * - enabledMap：插件启用状态
   * - homePanelMode：主页右侧内容区模式
   * - customLayoutConfig：自定义布局配置
   *
   * URL 来源的自定义布局若无缓存内容，通过 load_custom_layout 命令单独获取。
   */
  async function syncFromBackend() {
    try {
      // 先加载外部插件清单（确保后端同步能覆盖外部插件）
      await loadExternalPlugins()

      // 从 AppData 读取个性化配置
      const data = await loadPersonalizationData()

      if (isValidHomePanelMode(data.homePanelMode)) {
        homePanelMode.value = data.homePanelMode as HomePanelMode
      }

      if (data.customLayoutConfig) {
        customLayoutConfig.value = { ...DEFAULT_CUSTOM_LAYOUT, ...data.customLayoutConfig }
      }

      if (data.enabledMap) {
        for (const [id, enabled] of Object.entries(data.enabledMap)) {
          if (runtimeStates.value[id]) {
            runtimeStates.value[id].enabled = enabled
          }
        }
      }

      // 若是 URL 来源且无缓存内容，通过 load_custom_layout 单独获取（命中本地缓存文件）
      if (
        homePanelMode.value === 'custom' &&
        customLayoutConfig.value.source === 'url' &&
        customLayoutConfig.value.url &&
        !customLayoutConfig.value.cachedContent
      ) {
        try {
          const content = await fetchCustomLayoutContent(customLayoutConfig.value.url)
          customLayoutConfig.value.cachedContent = content
          customLayoutConfig.value.cachedAt = Date.now()
          await persistToBackend()
        } catch (e) {
          console.warn('[Plugins] Failed to load custom layout from URL:', e)
        }
      }

      backendSynced.value = true
    } catch (e) {
      console.warn('[Plugins] Backend sync skipped:', e)
    }
  }

  /**
   * 启用/禁用插件
   *
   * - 更新运行时状态
   * - 调用插件钩子（onEnable / onDisable）
   * - 持久化到后端 AppData
   */
  async function setPluginEnabled(id: string, enabled: boolean) {
    const state = runtimeStates.value[id]
    if (!state) return

    const manifest = manifests.value.find((m) => m.id === id)
    if (!manifest) return

    // 如果禁用的是当前 homePanelMode 对应的插件，回退到 default
    if (!enabled && homePanelMode.value === `plugin:${id}`) {
      homePanelMode.value = 'default'
    }

    state.enabled = enabled

    // 调用钩子（失败仅记录，不阻塞）
    try {
      if (enabled && manifest.hooks?.onEnable) {
        await manifest.hooks.onEnable()
      } else if (!enabled && manifest.hooks?.onDisable) {
        await manifest.hooks.onDisable()
      }
      state.lastError = null
    } catch (e) {
      state.lastError = String(e)
      console.error(`[Plugins] Plugin ${id} hook failed:`, e)
    }

    await persistToBackend()
  }

  /**
   * 触发游戏启动事件（供 Home.vue 调用）
   *
   * 内置插件调用 hooks，外部插件通过 window 自定义事件桥接到沙箱。
   */
  async function notifyGameLaunch(versionId: string) {
    for (const plugin of enabledPlugins.value) {
      if (plugin.hooks?.onGameLaunch) {
        await safeCall(() => Promise.resolve(plugin.hooks!.onGameLaunch!(versionId)), `[Plugins] ${plugin.id} onGameLaunch`)
      }
    }
    window.dispatchEvent(new CustomEvent('plugin:game-launch', { detail: { versionId } }))
  }

  /**
   * 触发游戏退出事件（供 Home.vue 调用）
   */
  async function notifyGameExit(versionId: string, exitCode: number | null) {
    for (const plugin of enabledPlugins.value) {
      if (plugin.hooks?.onGameExit) {
        await safeCall(() => Promise.resolve(plugin.hooks!.onGameExit!(versionId, exitCode)), `[Plugins] ${plugin.id} onGameExit`)
      }
    }
    window.dispatchEvent(
      new CustomEvent('plugin:game-exit', { detail: { versionId, exitCode } }),
    )
  }

  return {
    enabledPlugins,
    builtinPluginList,
    externalPluginList,
    initRuntimeStates,
    loadExternalPlugins,
    installFromDir,
    installFromZip,
    uninstallExternal,
    syncFromBackend,
    setPluginEnabled,
    notifyGameLaunch,
    notifyGameExit,
  }
}
