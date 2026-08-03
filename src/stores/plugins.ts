/**
 * 插件状态管理（聚合入口，切片组装）
 *
 * 内置插件静态加载 + 外部插件异步扫描；启用状态 / 主页模式 / 自定义布局
 * 统一持久化到 `%APPDATA%/.Molaunch/personalization.json`。
 * 外部插件安装与同步在 plugins/plugins-install.ts，
 * 主页模式与自定义布局在 plugins/plugins-layout.ts。
 */
import { defineStore } from 'pinia'
import { ref } from 'vue'
import type {
  PluginManifest,
  PluginRuntimeState,
  HomePanelMode,
  CustomLayoutConfig,
} from '@/types/plugin'
import { builtinPlugins } from '@/plugins'
import type { ExternalPluginEntry } from '@/utils/api/plugins'
import {
  DEFAULT_CUSTOM_LAYOUT,
  savePersonalizationData,
  type PersonalizationData,
} from '@/utils/pluginInstaller'
import { safeCall } from '@/utils/async'
import { toastError } from '@/utils/toast'
import { usePluginsInstallSlice } from './plugins/plugins-install'
import { usePluginsLayoutSlice } from './plugins/plugins-layout'

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

  /**
   * 将当前状态持久化到后端 AppData（`%APPDATA%/.Molaunch/personalization.json`）
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
    }, '[Plugins] persist to backend', (e) => toastError('保存设置失败：' + String(e)))
  }

  // 切片组装：外部插件安装/状态 + 主页模式/自定义布局
  const install = usePluginsInstallSlice({
    manifests,
    runtimeStates,
    externalPluginsRaw,
    backendSynced,
    homePanelMode,
    customLayoutConfig,
    persistToBackend,
  })
  const layout = usePluginsLayoutSlice({
    manifests,
    homePanelMode,
    customLayoutConfig,
    persistToBackend,
  })

  // 初始化运行时状态（启用状态由 syncFromBackend 异步从 AppData 加载）
  install.initRuntimeStates()

  return {
    manifests,
    runtimeStates,
    homePanelMode,
    customLayoutConfig,
    backendSynced,
    externalPluginsRaw,
    ...install,
    ...layout,
  }
})
