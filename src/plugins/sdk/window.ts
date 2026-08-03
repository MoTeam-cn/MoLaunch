/**
 * SDK 窗口域
 *
 * createWindow 为高级权限，仅外部插件可通过沙箱桥接注入 pluginId 后调用。
 */
import type { CreateWindowOptions } from './sdk-types'

/** 创建子窗口（高级权限，内置插件直接调用会抛出错误） */
export async function createWindow(_options: CreateWindowOptions): Promise<void> {
  // 外部插件的 createWindow 请求由 PluginSandbox 特殊拦截处理（注入 pluginId）
  throw new Error('createWindow 仅外部插件可用（需通过沙箱桥接注入 pluginId 上下文）')
}