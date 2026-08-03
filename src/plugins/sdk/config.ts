/**
 * SDK 配置域
 *
 * 通过 config_manager 读取非敏感配置项。
 */
import { CONFIG_ACTIONS, configManager } from '@/utils/api/config-manager'

/** 读取启动器配置（过滤敏感字段） */
export async function fetchSdkConfig(): Promise<Record<string, unknown>> {
  const entries = await configManager<Array<{ key: string; value: unknown }>>(
    CONFIG_ACTIONS.GET_CONFIG,
    { keys: null },
  )
  const result: Record<string, unknown> = {}
  for (const e of entries) {
    // 过滤敏感字段
    if (e.key === 'curseforgeApiKey' || e.key === 'curseforgeEnabled') continue
    result[e.key] = e.value
  }
  return result
}