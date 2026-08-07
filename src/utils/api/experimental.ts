/**
 * 实验性功能 API 统一入口。具体 action 域实现位于相邻模块，保持既有导出兼容。
 */

import { invoke } from '@tauri-apps/api/core'

export async function experimentalManager<T = unknown>(action: string, params?: unknown): Promise<T> {
  return invoke<T>('experimental_manager', { req: { action, params: params ?? null } })
}

export { EXPERIMENTAL_ACTIONS, type ExperimentalAction } from './experimental-actions'
export * from './experimental-conversation'
export * from './experimental-chat'
export * from './experimental-context'
export * from './experimental-analyze'
export * from './experimental-events'
