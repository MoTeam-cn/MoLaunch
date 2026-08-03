/**
 * SDK 进程域
 *
 * spawnProcess 为高级权限，仅外部插件可通过沙箱桥接注入 pluginId 后调用。
 */
import type { ProcessResult, SpawnProcessOptions } from './sdk-types'

/** 执行子进程命令（高级权限，内置插件直接调用会抛出错误） */
export async function spawnProcess(
  _command: string,
  _args: string[],
  _options?: SpawnProcessOptions,
): Promise<ProcessResult> {
  // 外部插件的 spawnProcess 请求由 PluginSandbox 特殊拦截处理（注入 pluginId）
  throw new Error('spawnProcess 仅外部插件可用（需通过沙箱桥接注入 pluginId 上下文）')
}