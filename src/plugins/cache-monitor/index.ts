/**
 * 内置插件：缓存监控
 *
 * 在主页右侧内容区显示各缓存目录的占用情况：
 * - 总占用 / 总文件数概览
 * - 按分类（运行缓存 / 临时缓存 / AppData）分组展示子目录明细
 * - 可手动刷新，不轮询（缓存数据变化不频繁，避免 IPC 重复读取）
 *
 * 与 system-monitor 的区别：system-monitor 综合展示内存/进程/SDK 状态等系统信息，
 * 本插件专注于缓存磁盘占用展示，提供更详细的子目录明细。
 */

import type { PluginManifest } from '@/types/plugin'
import CacheMonitorPanel from './CacheMonitorPanel.vue'

export const cacheMonitorPlugin: PluginManifest = {
  id: 'cache-monitor',
  name: '缓存监控',
  description: '在主页右侧显示各缓存目录的占用情况与文件数量',
  version: '1.0.0',
  author: 'MoLaunch',
  capabilities: () => ({
    homePanel: CacheMonitorPanel,
  }),
  builtin: true,
}
