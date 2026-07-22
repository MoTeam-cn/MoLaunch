/**
 * 外部插件 API 封装
 *
 * 与后端 commands/plugins/mod.rs 对应：
 * - listExternalPlugins()：扫描 `<base_dir>/plugins/` 下的外部插件
 * - readExternalPluginFile()：读取插件文件内容（路径限制在插件目录内）
 * - installExternalPluginFromDir()：从源目录安装插件
 * - installExternalPluginFromZip()：从 ZIP 字节数组安装插件
 * - uninstallExternalPlugin()：卸载插件（删除目录）
 */

import { invoke } from '@tauri-apps/api/core'

/** 外部插件清单（对应后端 ExternalPluginManifest） */
export interface ExternalPluginManifest {
  id: string
  name: string
  description: string
  version: string
  author: string
  /** HTML 入口文件相对路径（相对插件目录，如 "index.html"） */
  entry: string
  /** 权限白名单（SDK 方法名数组） */
  permissions: string[]
}

/** 已扫描到的外部插件（含目录路径） */
export interface ExternalPluginEntry extends ExternalPluginManifest {
  /** 插件目录绝对路径 */
  dir: string
}

/**
 * 列出所有已安装的外部插件
 */
export async function listExternalPlugins(): Promise<ExternalPluginEntry[]> {
  return await invoke<ExternalPluginEntry[]>('list_external_plugins')
}

/**
 * 读取外部插件文件内容
 *
 * 安全限制：file_path 必须是相对路径，且解析后必须位于插件目录内。
 */
export async function readExternalPluginFile(
  pluginId: string,
  filePath: string,
): Promise<string> {
  return await invoke<string>('read_external_plugin_file', {
    pluginId,
    filePath,
  })
}

/**
 * 从源目录安装外部插件
 *
 * 返回安装后的插件 ID。
 */
export async function installExternalPluginFromDir(sourceDir: string): Promise<string> {
  return await invoke<string>('install_external_plugin_from_dir', { sourceDir })
}

/**
 * 从 ZIP 文件路径安装外部插件
 *
 * ZIP 结构支持：
 * - 扁平结构（根直接包含 manifest.json）
 * - 单根目录结构（ZIP 内有一个根目录，其下包含 manifest.json）
 *
 * 返回安装后的插件 ID。
 */
export async function installExternalPluginFromZip(zipPath: string): Promise<string> {
  return await invoke<string>('install_external_plugin_from_zip', { zipPath })
}

/**
 * 卸载外部插件（删除插件目录）
 */
export async function uninstallExternalPlugin(pluginId: string): Promise<void> {
  return await invoke<void>('uninstall_external_plugin', { pluginId })
}

/**
 * 导出示例插件模板到指定路径
 * @param destPath 目标路径（ZIP 文件路径或文件夹路径）
 * @param asZip true 导出 ZIP，false 导出文件夹
 */
export async function exportPluginSample(destPath: string, asZip: boolean): Promise<void> {
  return await invoke<void>('export_plugin_sample', { destPath, asZip })
}

/**
 * 读取内置示例布局文件内容
 * @param format 布局格式：'json' / 'xml' / 'html'
 */
export async function readLayoutSample(format: string): Promise<string> {
  return await invoke<string>('read_layout_sample', { format })
}
