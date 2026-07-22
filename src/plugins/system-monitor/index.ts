/**
 * 内置插件：系统状态监控
 *
 * 显示系统内存占用、游戏进程运行状态、SDK 初始化状态。
 * 每 3 秒轮询刷新内存与进程状态。
 */

import type { PluginManifest } from '@/types/plugin'
import SystemMonitorPanel from './SystemMonitorPanel.vue'

export const systemMonitorPlugin: PluginManifest = {
  id: 'system-monitor',
  name: '系统状态',
  description: '显示内存占用、游戏进程、SDK 状态等系统监控信息',
  version: '1.0.0',
  author: 'MoLaunch',
  capabilities: () => ({
    homePanel: SystemMonitorPanel,
  }),
  builtin: true,
}
