/**
 * Java 检测、校验、下载 API
 *
 * 注：底层已聚合为 `java_manager` 单一 IPC 入口，通过 `action` 字段分发。
 * `getDeviceId` 例外：底层属于 SDK 命令，已聚合到 `sdk_manager` 入口。
 * 类型定义与常量保留在此文件，业务调用点保持类型安全。
 */

import type { JavaRuntime, JavaRequirements, JavaCompatResult } from '@/types/java'
import { JAVA_ACTIONS, javaManager } from './java-manager'
import { SDK_ACTIONS, sdkManager } from './sdk-manager'

/**
 * 获取设备 ID
 *
 * 注：`get_device_id` 属于 SDK 命令，已聚合到 `sdk_manager` IPC 入口，通过 `action` 字段分发。
 */
export async function getDeviceId(): Promise<string> {
  return sdkManager<string>(SDK_ACTIONS.GET_DEVICE_ID)
}

/**
 * 检测 Java
 */
export async function detectJava(): Promise<JavaRuntime> {
  return javaManager<JavaRuntime>(JAVA_ACTIONS.DETECT_JAVA)
}

/**
 * 列出所有 Java
 */
export async function listJava(): Promise<JavaRuntime[]> {
  return javaManager<JavaRuntime[]>(JAVA_ACTIONS.LIST_JAVA)
}

/**
 * 获取 MC 版本的 Java 需求（支持加载器约束）
 */
export async function getJavaRequirements(
  mcVersion: string,
  loader?: string | null,
): Promise<JavaRequirements> {
  return javaManager<JavaRequirements>(JAVA_ACTIONS.GET_JAVA_REQUIREMENTS, {
    mcVersion,
    loader: loader ?? null,
  })
}

/**
 * 检查指定 Java 是否兼容 MC 版本需求
 */
export async function checkJavaCompatible(
  javaPath: string,
  mcVersion: string,
  loader?: string | null,
): Promise<JavaCompatResult> {
  return javaManager<JavaCompatResult>(JAVA_ACTIONS.CHECK_JAVA_COMPATIBLE, {
    javaPath,
    mcVersion,
    loader: loader ?? null,
  })
}

/**
 * 纯函数：判断某 Java 主版本号是否落在 JavaRequirements 允许的范围内
 *
 * @param majorVersion Java 大版本号（如 8、17、21）
 * @param reqs         后端返回的 Java 需求；为 null 时视为无约束
 */
export function isJavaCompatible(majorVersion: number, reqs: JavaRequirements | null): boolean {
  if (!reqs) return true
  const { min_java_version: min, max_java_version: max } = reqs
  if (min && majorVersion < min) return false
  if (max && majorVersion > max) return false
  return true
}

/**
 * Java 下载进度事件名
 */
export const JAVA_DOWNLOAD_PROGRESS_EVENT = 'java-download-progress'

// 重新导出 Java 下载进度类型（便于 store/组件通过 tauri 命名空间访问）
export type { JavaDownloadProgress } from '@/types/java'

/**
 * 下载 Java Runtime（从 Mojang 官方 Java Runtime 索引）
 *
 * @param targetMajor 目标 Java 大版本号（如 21、17、8）
 * @returns 下载的 java.exe 完整路径
 *
 * 进度通过 `java-download-progress` 事件推送，监听 `JavaDownloadProgress` payload
 */
export async function downloadJava(targetMajor: number): Promise<string> {
  return javaManager<string>(JAVA_ACTIONS.DOWNLOAD_JAVA, { targetMajor })
}
