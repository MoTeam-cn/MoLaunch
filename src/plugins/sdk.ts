/**
 * 插件 SDK
 *
 * 提供给插件调用的有限后端 API 包装。
 * 插件通过 `import { pluginSdk } from '@/plugins/sdk'` 获取 SDK 实例。
 *
 * 接口定义见 sdk/sdk-types，实现按域拆分于 sdk/config、sdk/window、sdk/system 等模块。
 */
export type {
  CacheStatEntry,
  CacheStatsResult,
  ProcessResult,
  SpawnProcessOptions,
  CreateWindowOptions,
  PluginSdk,
} from './sdk/sdk-types'
export { pluginSdk } from './sdk/instance'