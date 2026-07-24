/**
 * 插件状态管理
 *
 * - 内置插件清单：从 `src/plugins/index.ts` 静态加载
 * - 外部插件清单：通过 `loadExternalPlugins()` 异步扫描后端 `<base_dir>/plugins/`
 * - 插件启用状态 / 主页模式 / 自定义布局配置：统一存储在 `%APPDATA%/.MolaLaunch/personalization.json`
 *   作为全系统共享存储，独立于游戏目录，确保不同 game_dir 的启动器实例加载同一份配置
 *
 * 纯函数和数据结构已抽离到 `@/utils/pluginInstaller`：
 * - PersonalizationData 接口 + DEFAULT_CUSTOM_LAYOUT 默认值
 * - externalManifestToPluginManifest 清单转换
 * - loadPersonalizationData / savePersonalizationData / fetchCustomLayoutContent 后端封装
 * - isValidHomePanelMode 字符串校验
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
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
  type PersonalizationData,
  DEFAULT_CUSTOM_LAYOUT,
  externalManifestToPluginManifest,
  loadPersonalizationData,
  savePersonalizationData,
  fetchCustomLayoutContent,
  isValidHomePanelMode,
} from '@/utils/pluginInstaller'
import { safeCall } from '@/utils/async'

export const usePluginStore = defineStore('plugins', () => {
  /** 已注册的插件清单（内置 + 外部） */
  const manifests = ref<PluginManifest[]>([...builtinPlugins])

  /** 插件运行时状态（id → state） */
  const runtimeStates = ref<Record<string, PluginRuntimeState>>({})

  /** 主页右侧内容区显示模式 */
  const homePanelMode = ref<HomePanelMode>('default')

  /** 自定义布局配置（homePanelMode = 'custom' 时有效） */
  const customLayoutConfig = ref<CustomLayoutConfig>({ ...DEFAULT_CUSTOM_LAYOUT })

  /** 是否已从后端同步 */
  const backendSynced = ref(false)

  /** 外部插件原始清单（保留 entry / permissions 供沙箱使用） */
  const externalPluginsRaw = ref<ExternalPluginEntry[]>([])

  /** 已启用的插件清单（计算属性） */
  const enabledPlugins = computed(() =>
    manifests.value.filter((m) => runtimeStates.value[m.id]?.enabled),
  )

  /** 提供主页内容区的插件列表（供个性化页选择） */
  const homePanelPlugins = computed(() =>
    manifests.value.filter((m) => m.capabilities?.()?.homePanel != null),
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
   * 将当前状态持久化到后端 AppData（`%APPDATA%/.MolaLaunch/personalization.json`）
   *
   * 全量覆盖写入，简单可靠。所有状态变更（启用/禁用插件、切换主页模式、更新布局配置）
   * 都调用此函数统一持久化，确保跨游戏目录共享。
   */
  async function persistToBackend() {
    await safeCall(async () => {
      const data: PersonalizationData = {
        enabledMap: {},
        homePanelMode: homePanelMode.value,
        customLayoutConfig: customLayoutConfig.value,
      }
      for (const [id, state] of Object.entries(runtimeStates.value)) {
        data.enabledMap[id] = state.enabled
      }
      await savePersonalizationData(data)
    }, '[Plugins] persist to backend')
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
   * 从 `%APPDATA%/.MolaLaunch/personalization.json` 读取全部配置：
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
   * 设置主页右侧内容区模式
   */
  async function setHomePanelMode(mode: HomePanelMode) {
    homePanelMode.value = mode
    await persistToBackend()
  }

  /**
   * 更新自定义布局配置
   *
   * 合并传入的字段到现有配置，持久化到后端 AppData。
   * URL 内容通过独立的 load_custom_layout 命令单独缓存到本地文件。
   */
  async function setCustomLayoutConfig(partial: Partial<CustomLayoutConfig>) {
    customLayoutConfig.value = { ...customLayoutConfig.value, ...partial }
    await persistToBackend()
  }

  /**
   * 刷新 URL 自定义布局缓存
   *
   * 调用后端命令重新下载 URL 内容并更新 cachedContent。
   */
  async function refreshCustomLayoutCache(): Promise<void> {
    const cfg = customLayoutConfig.value
    if (cfg.source !== 'url' || !cfg.url) {
      throw new Error('当前布局来源不是 URL 或 URL 为空')
    }

    try {
      // forceRefresh=true 强制忽略本地缓存重新下载
      const content = await fetchCustomLayoutContent(cfg.url, true)
      customLayoutConfig.value = {
        ...cfg,
        cachedContent: content,
        cachedAt: Date.now(),
      }
      await persistToBackend()
    } catch (e) {
      throw new Error(`刷新布局缓存失败：${e}`)
    }
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

  // 初始化运行时状态（启用状态由 syncFromBackend 异步从 AppData 加载）
  initRuntimeStates()

  return {
    manifests,
    runtimeStates,
    homePanelMode,
    customLayoutConfig,
    backendSynced,
    externalPluginsRaw,
    enabledPlugins,
    homePanelPlugins,
    builtinPluginList,
    externalPluginList,
    setPluginEnabled,
    setHomePanelMode,
    setCustomLayoutConfig,
    refreshCustomLayoutCache,
    syncFromBackend,
    loadExternalPlugins,
    installFromDir,
    installFromZip,
    uninstallExternal,
    notifyGameLaunch,
    notifyGameExit,
  }
})
