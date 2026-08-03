/**
 * 版本个性化、Mod 管理、选中版本、文件补全、脚本导出 API
 *
 * 底层按域聚合为 version_list_manager / version_mods_manager / version_launch_manager /
 * version_install_manager 多个统一 IPC 入口，经 action 分发。
 * 对外组合导出各域切片，保持原有 API 签名不变。
 */
export * from './personalization/types'
export * from './personalization/personal'
export * from './personalization/script'
export * from './personalization/mods'
export * from './personalization/preload'
export * from './personalization/version'