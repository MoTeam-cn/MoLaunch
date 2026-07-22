/**
 * 内置插件：快速统计
 *
 * 在主页右侧内容区显示已安装版本数量等统计信息。
 */

import type { PluginManifest } from '@/types/plugin'
import QuickStatsPanel from './QuickStatsPanel.vue'

export const quickStatsPlugin: PluginManifest = {
  id: 'quick-stats',
  name: '快速统计',
  description: '在主页右侧显示已安装版本数量等统计信息',
  version: '1.0.0',
  author: 'MoLaunch',
  capabilities: () => ({
    homePanel: QuickStatsPanel,
  }),
  builtin: true,
}
