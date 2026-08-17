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
 * 人类可读的 Java 需求描述（如"需要 Java 17~21"、"至少需要 Java 17"）
 *
 * min/max 为 0 表示该方向无约束；两者为 0 时返回"无特殊要求"。
 */
export function describeJavaRequirement(reqs: JavaRequirements | null): string {
  if (!reqs) return ''
  const { min_java_version: min, max_java_version: max } = reqs
  if (min && max && min === max) return `需要 Java ${min}`
  if (min && max) return `需要 Java ${min}~${max}`
  if (min) return `至少需要 Java ${min}`
  if (max) return `最高兼容到 Java ${max}`
  return '无特殊 Java 要求'
}

/**
 * Java 大版本元数据（参考 Adoptium API 的当前版本分布）
 *
 * - 可用区间：8 ~ 26（8/11/16~26；26 为最新功能版，2026-03 发布，27 之后为 28）
 * - LTS：8 / 11 / 17 / 21 / 25（25 为最新 LTS）
 *
 * 仅用于：① 自定义输入框的上下限校验；② 标记 LTS。与「能否下载」无关。
 */
export const MIN_JAVA_MAJOR = 8
export const MAX_JAVA_MAJOR = 26
export const LTS_JAVA_MAJORS = [8, 11, 17, 21, 25]

/** 判断某 Java 大版本是否为 LTS（长期支持版） */
export function isLtsJavaMajor(major: number): boolean {
  return LTS_JAVA_MAJORS.includes(major)
}

/**
 * Mojang 官方 Runtime 实际提供的 Java 大版本（对齐官方 all.json 五档：8/16/17/21/25，
 */
export const OFFICIAL_JAVA_MAJORS = [25, 21, 17, 16, 8]

/**
 * 判断某 Java 大版本是否有对应的官方 Mojang Runtime（仅影响下载可用性提示，不阻断下载）
 */
export function hasOfficialRuntime(major: number): boolean {
  return OFFICIAL_JAVA_MAJORS.includes(major)
}

/**
 * 校验用户自定义输入的 Java 大版本号是否合法
 *
 * 规则：1~2 位纯数字（无空格/符号/小数点）、数值在 8~26 之间
 * （下限：Minecraft 需要 Java 8+；上限：参考 Adoptium 当前最新可用版本）。
 * 用于下载器自由输入框的前置校验，避免把违规值/不存在的版本直接交给后端。
 */
export function isJavaMajorValid(input: string): boolean {
  const trimmed = input.trim()
  if (!/^\d{1,2}$/.test(trimmed)) return false
  const n = Number(trimmed)
  return n >= MIN_JAVA_MAJOR && n <= MAX_JAVA_MAJOR
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
