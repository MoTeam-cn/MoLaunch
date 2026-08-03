/**
 * SDK 单例
 *
 * 将各域实现组合为 PluginSdk 实例。
 */
import { fetchSdkConfig } from './config'
import {
  listInstalledVersions,
  listInstalledVersionsWithType,
  listLaunchHistory,
  getRunningGamePid,
} from './version'
import { getSystemMemory, getCacheStats } from './system'
import { spawnProcess } from './process'
import { createWindow } from './window'
import { emitEvent, logMessage } from './events'
import type { PluginSdk } from './sdk-types'

/**
 * 插件 SDK 实现
 */
class PluginSdkImpl implements PluginSdk {
  getConfig = fetchSdkConfig
  listInstalledVersions = listInstalledVersions
  listInstalledVersionsWithType = listInstalledVersionsWithType
  listLaunchHistory = listLaunchHistory
  getSystemMemory = getSystemMemory
  getRunningGamePid = getRunningGamePid
  getCacheStats = getCacheStats
  spawnProcess = spawnProcess
  createWindow = createWindow
  emit = emitEvent
  log = logMessage
}

/** 插件 SDK 单例 */
export const pluginSdk: PluginSdk = new PluginSdkImpl()