import { VERSION_LAUNCH_ACTIONS, versionLaunchManager } from '../version-launch-manager'

/**
 * 导出启动脚本（.bat，使用绝对路径 Java + 版权信息）
 *
 * 安全修复：移除 accessToken 参数，后端根据 uuid 自行从 auth_storage 获取 token
 *
 * @param javaPath 用户指定的 Java 路径（可选，为空时后端按 MC 版本自动检测）
 *
 * 注：底层已聚合为 `version_launch_manager` 单一 IPC 入口，通过 `action` 字段分发。
 */
export async function exportLaunchScript(
  versionId: string,
  username: string,
  uuid: string,
  loginType: string,
  javaPath: string | null,
  savePath: string,
): Promise<void> {
  return versionLaunchManager<void>(VERSION_LAUNCH_ACTIONS.EXPORT_LAUNCH_SCRIPT, {
    versionId,
    username,
    uuid,
    loginType,
    javaPath,
    savePath,
  })
}