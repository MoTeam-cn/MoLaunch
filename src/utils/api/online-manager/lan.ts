/**
 * MC 局域网伪装 + 端口探测 IPC 封装（`online_manager` 经 action 分发）
 */
import { onlineManager, ONLINE_ACTIONS } from './core'
import type {
  LanFakeStartParams,
  LanFakeStartResult,
  LanPortProbeResult,
  RunningMcPortResult,
} from '@/types/online'

/** 启动 MC 局域网伪装服务（TCP 转发 + UDP 周期广播） */
export function lanFakeServerStart(params: LanFakeStartParams): Promise<LanFakeStartResult> {
  return onlineManager<LanFakeStartResult>(ONLINE_ACTIONS.LAN_FAKE_SERVER_START, params)
}

/** 停止 MC 局域网伪装服务 */
export function lanFakeServerStop(): Promise<{ success: boolean }> {
  return onlineManager<{ success: boolean }>(ONLINE_ACTIONS.LAN_FAKE_SERVER_STOP)
}

/** 监听局域网发现广播，解析 `[MOTD]xx[/MOTD][AD]port[/AD]` */
export function lanPortProbe(timeoutMs?: number): Promise<LanPortProbeResult> {
  return onlineManager<LanPortProbeResult>(ONLINE_ACTIONS.LAN_PORT_PROBE, { timeoutMs })
}

/** 按当前游戏进程 PID 回查监听端口 */
export function getRunningMcPort(): Promise<RunningMcPortResult> {
  return onlineManager<RunningMcPortResult>(ONLINE_ACTIONS.GET_RUNNING_MC_PORT)
}
