/**
 * easytier / Scaffolding IPC 封装（`online_manager` 经 action 分发）
 *
 * 返回结构直接来自后端（非 BusinessResult 包裹）：成功时 `success: true`，
 * 失败时后端抛错由 onlineManager 统一抛出。
 */
import { onlineManager, ONLINE_ACTIONS } from './core'
import type {
  EasyTierJoinParams,
  EasyTierJoinResult,
  ScaffoldingClientProbeParams,
  ScaffoldingClientProbeResult,
  ScaffoldingHostStartParams,
  ScaffoldingHostStartResult,
} from '@/types/online'

/** 拉起 easytier-core 加入虚拟网络 */
export function joinEasyTier(params: EasyTierJoinParams): Promise<EasyTierJoinResult> {
  return onlineManager<EasyTierJoinResult>(ONLINE_ACTIONS.EASYTIER_JOIN, params)
}

/** 停止当前 easytier 子进程 */
export function stopEasyTier(): Promise<{ success: boolean }> {
  return onlineManager<{ success: boolean }>(ONLINE_ACTIONS.EASYTIER_STOP)
}

/** 房主一站式启动：探测 MC 端口 → 联机中心 → easytier */
export function scaffoldingHostStart(
  params: ScaffoldingHostStartParams,
): Promise<ScaffoldingHostStartResult> {
  return onlineManager<ScaffoldingHostStartResult>(ONLINE_ACTIONS.SCAFFOLDING_HOST_START, params)
}

/** 停止联机中心与 easytier */
export function scaffoldingHostStop(): Promise<{ success: boolean }> {
  return onlineManager<{ success: boolean }>(ONLINE_ACTIONS.SCAFFOLDING_HOST_STOP)
}

/** 房客解析房间码 → 加入网络 → 探测房主 MC 服务 */
export function scaffoldingClientProbe(
  params: ScaffoldingClientProbeParams,
): Promise<ScaffoldingClientProbeResult> {
  return onlineManager<ScaffoldingClientProbeResult>(
    ONLINE_ACTIONS.SCAFFOLDING_CLIENT_PROBE,
    params,
  )
}
