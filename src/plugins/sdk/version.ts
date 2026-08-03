/**
 * SDK 版本域
 *
 * 通过 version_list_manager / version_launch_manager 查询版本与启动记录。
 */
import { VERSION_LAUNCH_ACTIONS, versionLaunchManager } from '@/utils/api/version-launch-manager'
import { VERSION_LIST_ACTIONS, versionListManager } from '@/utils/api/version-list-manager'

/** 读取已安装版本 ID 列表 */
export async function listInstalledVersions(): Promise<string[]> {
  return versionListManager<string[]>(VERSION_LIST_ACTIONS.LIST_INSTALLED_VERSIONS)
}

/** 读取已安装版本列表（带类型信息） */
export async function listInstalledVersionsWithType(): Promise<
  Array<{ id: string; version_type: string; logo: string }>
> {
  return versionListManager<Array<{ id: string; version_type: string; logo: string }>>(
    VERSION_LIST_ACTIONS.LIST_INSTALLED_VERSIONS_WITH_TYPE,
  )
}

/** 读取最近 50 条启动记录 */
export async function listLaunchHistory(): Promise<
  Array<{
    version_id: string
    username: string
    launch_time: string
    pid: number
    exit_code: number | null
  }>
> {
  return versionLaunchManager<
    Array<{
      version_id: string
      username: string
      launch_time: string
      pid: number
      exit_code: number | null
    }>
  >(VERSION_LAUNCH_ACTIONS.GET_LAUNCH_HISTORY)
}

/** 获取当前运行中的游戏 PID（null 表示无游戏运行） */
export async function getRunningGamePid(): Promise<number | null> {
  return versionLaunchManager<number | null>(VERSION_LAUNCH_ACTIONS.GET_RUNNING_GAME)
}