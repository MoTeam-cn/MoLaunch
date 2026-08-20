/**
 * 外部链接打开工具：统一校验 scheme（仅放行 http/https）后调用 Tauri shell 打开。
 * 供 ExternalLoginPanel / useFrpAuthCenter 等 open 调用复用，防止任意 scheme 被系统处理。
 */
import { open } from '@tauri-apps/plugin-shell'

/** 校验 URL 是否允许通过系统浏览器打开（仅放行 http/https） */
export function isSafeExternalUrl(url: string): boolean {
  return /^https?:\/\//i.test(url)
}

/** 通过系统浏览器打开外部链接；非法 scheme 抛错拒绝 */
export async function openExternal(url: string): Promise<void> {
  if (!isSafeExternalUrl(url)) {
    throw new Error('仅支持打开 http/https 链接')
  }
  await open(url)
}