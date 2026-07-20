/**
 * 启动游戏相关 API
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 启动游戏
 *
 * 安全修复：移除 accessToken 参数，后端根据 uuid 自行从 auth_storage 获取 token
 * 前端只传 username 和 uuid，避免 token 在 IPC 请求体中明文传输
 */
export async function launchGame(params: {
  versionId: string
  javaPath?: string
  username: string
  uuid: string
  loginType?: string
  windowWidth?: number
  windowHeight?: number
  serverAddress?: string
  serverPort?: number
}): Promise<number> {
  return await invoke<number>('launch_game', {
    versionId: params.versionId,
    javaPath: params.javaPath ?? null,
    username: params.username,
    uuid: params.uuid,
    windowWidth: params.windowWidth ?? null,
    windowHeight: params.windowHeight ?? null,
    serverAddress: params.serverAddress ?? null,
    serverPort: params.serverPort ?? null,
    loginType: params.loginType ?? null,
  })
}

export interface LaunchProgress {
  stage: string
  stage_progress: number
  overall_progress: number
  message: string
}

/**
 * 获取启动进度
 */
export async function getLaunchProgress(): Promise<LaunchProgress | null> {
  return await invoke<LaunchProgress | null>('get_launch_progress')
}

/**
 * 取消启动
 */
export async function cancelLaunch(): Promise<void> {
  return await invoke<void>('cancel_launch')
}

/**
 * 停止游戏
 */
export async function stopGame(): Promise<void> {
  return await invoke<void>('stop_game')
}

/**
 * 获取当前运行的游戏PID
 */
export async function getRunningGame(): Promise<number | null> {
  return await invoke<number | null>('get_running_game')
}
