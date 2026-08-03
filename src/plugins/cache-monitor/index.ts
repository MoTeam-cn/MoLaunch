/**
 * 内置插件：缓存监控。在主页右侧展示各缓存目录占用（总占用/总文件数概览 + 按分类分组子目录明细）。
 * 手动刷新不轮询（避免 IPC 重复读取）；区别于 system-monitor 的系统综合信息，本插件专注缓存磁盘占用。
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
