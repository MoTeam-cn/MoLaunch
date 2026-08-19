/**
 * 启动游戏相关 API
 *
 * 注：底层已聚合为 `version_launch_manager` 单一 IPC 入口，通过 `action` 字段分发
 * （原 6 个 launch 命令均通过该入口调用）。
 */

import { VERSION_LAUNCH_ACTIONS, versionLaunchManager } from './version-launch-manager'

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
  /** 临时追加的 JVM 参数（单次启动有效，不写入 setup.ini）
   *  用途：联机模块启动 MC 时追加 -Djava.net.preferIPv4Stack=true */
  extraJvmArgs?: string[]
}): Promise<number> {
  return versionLaunchManager<number>(VERSION_LAUNCH_ACTIONS.LAUNCH_GAME, {
    versionId: params.versionId,
    javaPath: params.javaPath ?? null,
    username: params.username,
    uuid: params.uuid,
    loginType: params.loginType ?? null,
    windowWidth: params.windowWidth ?? null,
    windowHeight: params.windowHeight ?? null,
    serverAddress: params.serverAddress ?? null,
    serverPort: params.serverPort ?? null,
    extraJvmArgs: params.extraJvmArgs ?? null,
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
  return versionLaunchManager<LaunchProgress | null>(VERSION_LAUNCH_ACTIONS.GET_LAUNCH_PROGRESS)
}

/**
 * 取消启动
 */
export async function cancelLaunch(): Promise<void> {
  return versionLaunchManager<void>(VERSION_LAUNCH_ACTIONS.CANCEL_LAUNCH)
}

/**
 * 停止游戏
 */
export async function stopGame(): Promise<void> {
  return versionLaunchManager<void>(VERSION_LAUNCH_ACTIONS.STOP_GAME)
}

/**
 * 获取当前运行的游戏PID
 */
export async function getRunningGame(): Promise<number | null> {
  return versionLaunchManager<number | null>(VERSION_LAUNCH_ACTIONS.GET_RUNNING_GAME)
}

/**
 * 启动历史记录（最近启动过的版本）
 *
 * 后端在内存中累积，重启启动器后清空。
 * 返回最近 50 条记录（按时间倒序，最近启动在前）。
 */
export interface LaunchHistoryEntry {
  /** 版本 ID */
  version_id: string
  /** 启动时使用的用户名 */
  username: string
  /** 启动时间（RFC3339 字符串） */
  launch_time: string
  /** 进程 ID */
  pid: number
  /** 退出码（null 表示仍在运行或异常终止未收集到） */
  exit_code: number | null
}

/**
 * 获取启动历史记录
 */
export async function getLaunchHistory(): Promise<LaunchHistoryEntry[]> {
  return versionLaunchManager<LaunchHistoryEntry[]>(VERSION_LAUNCH_ACTIONS.GET_LAUNCH_HISTORY)
}

/**
 * 启动参数预览结果（token 已脱敏，不包含 access_token / client_token）
 */
export interface LaunchArgsPreview {
  jvm_args: string[]
  game_args: string[]
  main_class: string
  classpath: string
  version_id: string
  game_dir: string
  assets_dir: string
  asset_index: string
  username: string
  uuid: string
  login_type: string
  server_url: string | null
  xuid: string
  /** 实际使用的 Java 路径 */
  java_path: string
}

/**
 * 预览启动参数（组装 JVM 参数但不启动游戏）
 *
 * 参数与 launchGame 一致；username/uuid/loginType 传当前登录账号，
 * 未登录时传空字符串（后端按离线兜底处理）。
 */
export async function previewLaunchArgs(params: {
  versionId: string
  javaPath?: string
  username: string
  uuid: string
  loginType?: string
  windowWidth?: number
  windowHeight?: number
  serverAddress?: string
  serverPort?: number
  extraJvmArgs?: string[]
}): Promise<LaunchArgsPreview> {
  return versionLaunchManager<LaunchArgsPreview>(VERSION_LAUNCH_ACTIONS.PREVIEW_LAUNCH_ARGS, {
    versionId: params.versionId,
    javaPath: params.javaPath ?? null,
    username: params.username,
    uuid: params.uuid,
    loginType: params.loginType ?? null,
    windowWidth: params.windowWidth ?? null,
    windowHeight: params.windowHeight ?? null,
    serverAddress: params.serverAddress ?? null,
    serverPort: params.serverPort ?? null,
    extraJvmArgs: params.extraJvmArgs ?? null,
  })
}
