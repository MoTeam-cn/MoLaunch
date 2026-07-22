/**
 * 内置插件：版本统计
 *
 * 显示已安装版本总数、按加载器分类的横向条形图、按主版本号分布的统计图。
 */

import type { PluginManifest } from '@/types/plugin'
import VersionStatsPanel from './VersionStatsPanel.vue'

export const versionStatsPlugin: PluginManifest = {
  id: 'version-stats',
  name: '版本统计',
  description: '以图表展示已安装版本按加载器与主版本号的分布',
  version: '1.0.0',
  author: 'MoLaunch',
  capabilities: () => ({
    homePanel: VersionStatsPanel,
  }),
  builtin: true,
}
