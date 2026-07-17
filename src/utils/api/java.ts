/**
 * Java 检测、校验、下载 API
 */

import { invoke } from '@tauri-apps/api/core'
import type { JavaRuntime, JavaRequirements, JavaCompatResult } from '@/types/java'

/**
 * 获取设备 ID
 */
export async function getDeviceId(): Promise<string> {
  return await invoke<string>('get_device_id')
}

/**
 * 检测 Java
 */
export async function detectJava(): Promise<JavaRuntime> {
  return await invoke<JavaRuntime>('detect_java')
}

/**
 * 列出所有 Java
 */
export async function listJava(): Promise<JavaRuntime[]> {
  return await invoke<JavaRuntime[]>('list_java')
}

/**
 * 获取 MC 版本的 Java 需求（支持加载器约束）
 */
export async function getJavaRequirements(
  mcVersion: string,
  loader?: string | null,
): Promise<JavaRequirements> {
  return await invoke<JavaRequirements>('get_java_requirements', {
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
  return await invoke<JavaCompatResult>('check_java_compatible', {
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
  return await invoke<string>('download_java', { targetMajor })
}
