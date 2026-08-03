/**
 * 设备 ID 前缀处理：联机设备标识统一为 `mcsdk-` 前缀格式。
 * UI 展示去前缀（stripMcsdkPrefix），写入/存储始终补全完整前缀（ensureMcsdkPrefix）。
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
