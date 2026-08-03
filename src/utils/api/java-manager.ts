/**
 * Java 管理模块统一 API 入口
 *
 * 后端 java_manager IPC 按 action 分发；params 字段一律 camelCase。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 java_manager IPC
 * @param action 操作名称（取自 JAVA_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase）
 */
export async function javaManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('java_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::java_manager::DISPATCHER` 注册的 action 一一对应。
 * 业务代码应优先使用此常量而非裸字符串，避免拼写错误。
 */
export const JAVA_ACTIONS = {
  DETECT_JAVA: 'detect_java',
  LIST_JAVA: 'list_java',
  SELECT_JAVA_FOR_MC: 'select_java_for_mc',
  GET_JAVA_REQUIREMENTS: 'get_java_requirements',
  CHECK_JAVA_COMPATIBLE: 'check_java_compatible',
  DOWNLOAD_JAVA: 'download_java',
} as const

/** action 名称类型 */
export type JavaAction = typeof JAVA_ACTIONS[keyof typeof JAVA_ACTIONS]
