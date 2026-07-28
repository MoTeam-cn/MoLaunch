/**
 * 设备 ID 前缀处理工具
 *
 * 联机模块的设备友好标识统一为 `mcsdk-xxxx-xxxx-xxxx-xxxx` 格式，
 * 但 UI 展示时不应暴露 `mcsdk-` 前缀，添加时自动补全。
 *
 * 约定：
 * - 展示层（WhitelistEditor 等）调用 `stripMcsdkPrefix` 去前缀显示
 * - 写入层（添加白名单、提交创建房间）调用 `ensureMcsdkPrefix` 补前缀
 * - 内部存储与后端交互始终使用完整 `mcsdk-` 前缀
 */

const MCSDK_PREFIX = 'mcsdk-'

/** 去除 `mcsdk-` 前缀（无前缀时原样返回） */
export function stripMcsdkPrefix(deviceId: string): string {
  if (deviceId.startsWith(MCSDK_PREFIX)) {
    return deviceId.slice(MCSDK_PREFIX.length)
  }
  return deviceId
}

/** 补全 `mcsdk-` 前缀（已有前缀时原样返回） */
export function ensureMcsdkPrefix(deviceId: string): string {
  if (deviceId.startsWith(MCSDK_PREFIX)) {
    return deviceId
  }
  return MCSDK_PREFIX + deviceId
}
