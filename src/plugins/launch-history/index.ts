/**
 * 内置插件：启动历史
 *
 * 在主页右侧内容区显示最近启动过的版本列表，含时间、版本 ID、用户名、退出状态。
 * 监听 plugin:game-launch / plugin:game-exit 事件实时刷新。
 */

import type { PluginManifest } from '@/types/plugin'
import LaunchHistoryPanel from './LaunchHistoryPanel.vue'

export const launchHistoryPlugin: PluginManifest = {
  id: 'launch-history',
  name: '启动历史',
  description: '显示最近启动过的版本记录，含启动时间与退出状态',
  version: '1.0.0',
  author: 'MoLaunch',
  capabilities: () => ({
    homePanel: LaunchHistoryPanel,
  }),
  builtin: true,
}
