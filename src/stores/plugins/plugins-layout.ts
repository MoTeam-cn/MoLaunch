/**
 * 插件 store - 主页模式 / 自定义布局切片
 *
 * homePanelMode + customLayoutConfig 的读写与持久化；homePanelPlugins 计算属性。
 * 共享状态 ref 由主 store 创建并注入。
 */
import { computed } from 'vue'
import type { Ref } from 'vue'
import type {
  PluginManifest,
  HomePanelMode,
  CustomLayoutConfig,
} from '@/types/plugin'
import { fetchCustomLayoutContent } from '@/utils/pluginInstaller'
import { isUrlAllowed } from '@/config/picker-templates'
import { toastError } from '@/utils/toast'

/** 自定义布局 URL 域名白名单（支持 *.example.com 通配符） */
export const CUSTOM_LAYOUT_ALLOWED_DOMAINS = [
  'moteam.top',
  '*.moteam.top',
  '*.molaunch.moiu.cn',
]

/**
 * 校验自定义布局 URL 是否允许加载
 *
 * 仅允许 https scheme，且域名必须在白名单内（复用 picker-templates 的 isUrlAllowed 匹配逻辑）。
 */
export function isCustomLayoutUrlAllowed(url: string): boolean {
  try {
    if (new URL(url).protocol !== 'https:') return false
    return isUrlAllowed(url, CUSTOM_LAYOUT_ALLOWED_DOMAINS)
  } catch {
    return false
  }
}

export interface PluginsLayoutDeps {
  /** 已注册的插件清单（内置 + 外部） */
  manifests: Ref<PluginManifest[]>
  /** 主页右侧内容区显示模式 */
  homePanelMode: Ref<HomePanelMode>
  /** 自定义布局配置 */
  customLayoutConfig: Ref<CustomLayoutConfig>
  /** 持久化到后端 AppData（由主 store 提供） */
  persistToBackend: () => Promise<void>
}

export function usePluginsLayoutSlice(deps: PluginsLayoutDeps) {
  const { manifests, homePanelMode, customLayoutConfig, persistToBackend } = deps

  /** 提供主页内容区的插件列表（供个性化页选择） */
  const homePanelPlugins = computed(() =>
    manifests.value.filter((m) => m.capabilities?.()?.homePanel != null),
  )

  /**
   * 设置主页右侧内容区模式
   */
  async function setHomePanelMode(mode: HomePanelMode) {
    homePanelMode.value = mode
    try {
      await persistToBackend()
    } catch (e) {
      toastError('保存设置失败：' + String(e))
    }
  }

  /**
   * 更新自定义布局配置
   *
   * 合并传入的字段到现有配置，持久化到后端 AppData。
   * URL 内容通过独立的 load_custom_layout 命令单独缓存到本地文件。
   */
  async function setCustomLayoutConfig(partial: Partial<CustomLayoutConfig>) {
    customLayoutConfig.value = { ...customLayoutConfig.value, ...partial }
    try {
      await persistToBackend()
    } catch (e) {
      toastError('保存设置失败：' + String(e))
    }
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
    if (!isCustomLayoutUrlAllowed(cfg.url)) {
      throw new Error(`布局 URL 不在白名单内（仅允许 https 且域名需在允许列表）：${cfg.url}`)
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

  return {
    homePanelPlugins,
    setHomePanelMode,
    setCustomLayoutConfig,
    refreshCustomLayoutCache,
  }
}
