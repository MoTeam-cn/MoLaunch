/**
 * 插件安装与个性化配置辅助模块（纯函数，不依赖 Pinia store 状态）
 *
 * 提供 PersonalizationData 结构、外部清单转换、个性化配置读写与自定义布局获取；
 * 底层经 plugins_manager 单一 IPC 按 action 分发。
 */
import { markRaw, h, defineComponent } from 'vue'
import type {
  PluginManifest,
  HomePanelMode,
  ExternalPluginManifest,
  CustomLayoutConfig,
} from '@/types/plugin'
import PluginSandbox from '@/plugins/sandbox/PluginSandbox.vue'
import { PLUGINS_ACTIONS, pluginsManager } from '@/utils/api/plugins-manager'

/** 个性化配置结构（与后端 personalization.json 对应） */
export interface PersonalizationData {
  /** 插件 ID → 是否启用 */
  enabledMap: Record<string, boolean>
  /** 主页右侧内容区显示模式 */
  homePanelMode: HomePanelMode
  /** 自定义布局配置（homePanelMode = 'custom' 时有效） */
  customLayoutConfig: CustomLayoutConfig
}

/** 默认自定义布局配置 */
export const DEFAULT_CUSTOM_LAYOUT: CustomLayoutConfig = {
  format: 'json',
  source: 'inline',
  inlineContent: '',
  url: '',
  cachedContent: '',
  cachedAt: 0,
}

/**
 * 将外部插件清单转换为内置 PluginManifest 结构
 *
 * 外部插件没有真实的 Vue 组件作为 homePanel，
 * 通过包装一个渲染函数返回 PluginSandbox 组件来代理渲染。
 *
 * 使用 markRaw 避免 Vue 对组件对象做不必要的响应式包装。
 */
export function externalManifestToPluginManifest(
  external: ExternalPluginManifest,
): PluginManifest {
  const sandboxComponent = markRaw(
    defineComponent({
      name: `PluginSandboxWrapper_${external.id}`,
      setup() {
        // 渲染 PluginSandbox 并透传 props
        return () =>
          h(PluginSandbox, {
            pluginId: external.id,
            entry: external.entry,
            permissions: external.permissions ?? [],
          })
      },
    }),
  )

  return {
    id: external.id,
    name: external.name,
    description: external.description,
    version: external.version,
    author: external.author,
    builtin: false,
    capabilities: () => ({
      homePanel: sandboxComponent,
    }),
    // 外部插件 manifest 声明的权限（用于插件管理页展示）
    permissions: external.permissions ?? [],
  }
}

/**
 * 从后端 AppData 读取个性化配置
 *
 * 包装 pluginsManager(READ_PERSONALIZATION)，返回 Partial 类型供调用方合并默认值。
 * 文件位置：%APPDATA%/.Molaunch/personalization.json
 */
export async function loadPersonalizationData(): Promise<Partial<PersonalizationData>> {
  return await pluginsManager<Partial<PersonalizationData>>(PLUGINS_ACTIONS.READ_PERSONALIZATION)
}

/**
 * 将个性化配置持久化到后端 AppData
 *
 * 包装 pluginsManager(WRITE_PERSONALIZATION)，全量覆盖写入。
 * 文件位置：%APPDATA%/.Molaunch/personalization.json
 */
export async function savePersonalizationData(data: PersonalizationData): Promise<void> {
  await pluginsManager<void>(PLUGINS_ACTIONS.WRITE_PERSONALIZATION, { data })
}

/**
 * 从后端加载 URL 自定义布局内容
 *
 * 包装 pluginsManager(LOAD_CUSTOM_LAYOUT)，命中本地缓存或强制刷新。
 * 后端按 url 的 sha256 哈希作为缓存文件名，缓存目录 .Molaunch/cache/custom_layout/。
 *
 * @param url 布局文件的 URL
 * @param forceRefresh true 时强制忽略本地缓存重新下载
 */
export async function fetchCustomLayoutContent(
  url: string,
  forceRefresh = false,
): Promise<string> {
  return await pluginsManager<string>(PLUGINS_ACTIONS.LOAD_CUSTOM_LAYOUT, {
    url,
    forceRefresh,
  })
}

/**
 * 校验 homePanelMode 字符串是否为合法值
 *
 * 合法值：'default' | 'custom' | 'plugin:<id>'
 */
export function isValidHomePanelMode(value: unknown): value is HomePanelMode {
  if (typeof value !== 'string') return false
  return value === 'default' || value === 'custom' || value.startsWith('plugin:')
}
