/**
 * 工具模块统一 API - 核心入口
 *
 * 后端 `tools_manager` IPC 命令通过 `action` 字段分发到不同子模块。
 * 本文件提供 `toolsManager` 调用入口与 `TOOLS_ACTIONS` 常量，
 * 各工具类别的类型定义与封装函数按域拆分到同目录其他文件。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 tools_manager IPC
 * @param action 操作名称（取自 TOOLS_ACTIONS 常量）
 * @param params 参数对象（可选）
 */
export async function toolsManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('tools_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `commands::tools::DISPATCHER` 注册的 action 一一对应。
 * 业务代码应优先使用此常量而非裸字符串，避免拼写错误。
 */
export const TOOLS_ACTIONS = {
  // 外部下载
  DOWNLOAD_FILE: 'download_file',
  GET_DOWNLOAD_DIR: 'get_download_dir',
  LIST_DOWNLOADS: 'list_downloads',
  DELETE_DOWNLOAD: 'delete_download',
  // 文件名获取
  FETCH_FILENAME: 'fetch_filename',
  // 清理游戏垃圾
  CLEANUP_SCAN: 'cleanup_scan',
  CLEANUP_EXECUTE: 'cleanup_execute',
  // 内存优化
  MEMORY_OPTIMIZE: 'memory_optimize',
  // Mod 依赖检测
  MOD_DEPENDENCY_CHECK: 'mod_dependency_check',
  // Mod 去重扫描
  MOD_DEDUP_SCAN: 'mod_dedup_scan',
  // 崩溃日志分析
  CRASH_ANALYZE: 'crash_analyze',
  // 截图批量管理
  SCREENSHOT_LIST: 'screenshot_list',
  SCREENSHOT_DELETE: 'screenshot_delete',
  // 资源包转换
  RESOURCEPACK_LIST: 'resourcepack_list',
  RESOURCEPACK_CONVERT: 'resourcepack_convert',
  // 版本 JSON 编辑
  VERSION_JSON_READ: 'version_json_read',
  VERSION_JSON_SAVE: 'version_json_save',
  // 存档管理
  ARCHIVE_LIST: 'archive_list',
  ARCHIVE_BACKUP: 'archive_backup',
  ARCHIVE_RESTORE: 'archive_restore',
  EXTRACT_SAVE_SEED: 'extract_save_seed',
  // 网络延迟测试
  NETWORK_LATENCY_TEST: 'network_latency_test',
  // 服务器状态检测
  SERVER_PING: 'server_ping',
  // TCP 端口连通性检测（Frp 等非 MC 协议服务）
  TCP_CHECK: 'tcp_check',
  // 列出本机监听端口（供 Frp 内网端口选择）
  LIST_OPEN_PORTS: 'list_open_ports',
  // 选择器子窗口（通用 HTML 渲染 + on_navigation 选择回调）
  OPEN_PICKER_WINDOW: 'open_picker_window',
  // NBT 数据查看
  NBT_PARSE: 'nbt_parse',
  // 合成配方生成器：数据包 zip 打包
  RECIPE_GENERATOR_EXPORT: 'recipe_generator_export',
} as const

/** action 名称类型 */
export type ToolsAction = typeof TOOLS_ACTIONS[keyof typeof TOOLS_ACTIONS]
