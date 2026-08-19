/**
 * easytier / Scaffolding IPC 封装（`online_manager` 经 action 分发）
 *
 * 返回结构直接来自后端（非 BusinessResult 包裹）：成功时 `success: true`，
 * 失败时后端抛错由 onlineManager 统一抛出。
 */
import { onlineManager, ONLINE_ACTIONS } from './core'
import type {
  EasyTierInstallStatus,
  EasyTierJoinParams,
  EasyTierJoinResult,
  EasyTierStatusResult,
  GithubProxy,
  ScaffoldingClientProbeParams,
  ScaffoldingClientProbeResult,
  ScaffoldingHostSetMcPortParams,
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

/** 查询当前 easytier 运行状态（joined/version/pid/rpcPortal） */
export function getEasyTierStatus(): Promise<EasyTierStatusResult> {
  return onlineManager<EasyTierStatusResult>(ONLINE_ACTIONS.EASYTIER_STATUS)
}

/** 查询 easytier 内核安装状态（installed/version/latestVersion/downloading） */
export function getEasyTierInstallStatus(): Promise<EasyTierInstallStatus> {
  return onlineManager<EasyTierInstallStatus>(ONLINE_ACTIONS.EASYTIER_INSTALL_STATUS)
}

/** 下载安装 easytier 内核最新版（进度经 `easytier-install-progress` 事件推送） */
export function installEasyTier(): Promise<{ success: boolean }> {
  return onlineManager<{ success: boolean }>(ONLINE_ACTIONS.EASYTIER_INSTALL)
}

/** 更新 easytier 内核到最新版（语义同 install） */
export function updateEasyTier(): Promise<{ success: boolean }> {
  return onlineManager<{ success: boolean }>(ONLINE_ACTIONS.EASYTIER_UPDATE)
}

/** 把前端筛选的 GitHub 镜像源传给后端（下载竞速选源用） */
export function setGithubProxies(proxies: GithubProxy[]): Promise<{ success: boolean }> {
  return onlineManager<{ success: boolean }>(ONLINE_ACTIONS.SET_GITHUB_PROXIES, proxies)
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

/** 房主手动指定 MC 端口（最高权重；null 清除手动覆盖，恢复自动探测） */
export function scaffoldingHostSetMcPort(
  params: ScaffoldingHostSetMcPortParams,
): Promise<{ success: boolean }> {
  return onlineManager<{ success: boolean }>(ONLINE_ACTIONS.SCAFFOLDING_HOST_SET_MC_PORT, params)
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

/** 房客周期轮询房主 MC 端口（轻量，无 join 分支，需已加入网络） */
export function scaffoldingClientPoll(
  params: ScaffoldingClientProbeParams,
): Promise<ScaffoldingClientProbeResult> {
  return onlineManager<ScaffoldingClientProbeResult>(ONLINE_ACTIONS.SCAFFOLDING_CLIENT_POLL, params)
}
