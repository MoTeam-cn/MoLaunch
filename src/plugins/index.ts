/**
 * 插件注册中心
 *
 * 所有内置插件在此处注册。启动器启动时会从此处加载插件清单。
 *
 * 未来扩展点：
 * - 支持从 `src-tauri/resources/plugins/` 目录动态加载外部插件
 * - 外部插件需通过沙箱（iframe 或 Web Worker）隔离执行
 */

import type { PluginManifest } from '@/types/plugin'
import { quickStatsPlugin } from './quick-stats'
import { launchHistoryPlugin } from './launch-history'
import { systemMonitorPlugin } from './system-monitor'
import { versionStatsPlugin } from './version-stats'
import { cacheMonitorPlugin } from './cache-monitor'

/** 已注册的内置插件清单列表 */
export const builtinPlugins: PluginManifest[] = [
  quickStatsPlugin,
  launchHistoryPlugin,
  systemMonitorPlugin,
  versionStatsPlugin,
  cacheMonitorPlugin,
]

/**
 * 根据 ID 查找插件清单
 */
export function findPlugin(id: string): PluginManifest | undefined {
  return builtinPlugins.find((p) => p.id === id)
}
