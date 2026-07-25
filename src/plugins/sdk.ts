/**
 * 插件 SDK
 *
 * 提供给插件调用的有限后端 API 包装。
 * 插件通过 `import { pluginSdk } from '@/plugins/sdk'` 获取 SDK 实例。
 *
 * 设计原则：
 * - 仅暴露安全的、只读的 API，不暴露任何可能破坏启动器状态的命令
 * - 所有写操作都通过事件系统，由启动器核心模块决定是否执行
 * - spawnProcess 为高级权限，仅外部插件可用（通过沙箱桥接注入 pluginId）
 *
 * 注：底层 IPC 已聚合为 13 个 manager 单一入口（system_manager / config_manager
 * / version_list_manager / version_launch_manager 等），本文件通过对应 manager
 * 调用，不再直接使用 `invoke('xxx')`。
 */

import { CONFIG_ACTIONS, configManager } from '@/utils/api/config-manager'
import { SYSTEM_ACTIONS, systemManager } from '@/utils/api/system-manager'
import { VERSION_LAUNCH_ACTIONS, versionLaunchManager } from '@/utils/api/version-launch-manager'
import { VERSION_LIST_ACTIONS, versionListManager } from '@/utils/api/version-list-manager'

/** 缓存统计条目（与后端 CacheStat 对应） */
export interface CacheStatEntry {
  name: string
  category: string
  subDir: string
  path: string
  fileCount: number
  totalSize: number
  ttlHours: number | null
}

/** 缓存统计结果（与后端 CacheStatsResult 对应） */
export interface CacheStatsResult {
  cache: CacheStatEntry[]
  cacheTemp: CacheStatEntry[]
  cacheApp: CacheStatEntry[]
}

/** 子进程执行结果（与后端 ProcessResult 对应） */
export interface ProcessResult {
  /** 退出码（null 表示超时或被信号终止） */
  exitCode: number | null
  /** 标准输出（截断到 1MB） */
  stdout: string
  /** 标准错误（截断到 1MB） */
  stderr: string
  /** 是否超时 */
  timedOut: boolean
  /** 执行耗时（毫秒） */
  durationMs: number
}

/** spawnProcess 选项 */
export interface SpawnProcessOptions {
  /** 工作目录（可选） */
  cwd?: string
}

/** createWindow 选项 */
export interface CreateWindowOptions {
  /** 窗口标签（插件内唯一标识） */
  label: string
  /** 要打开的 URL */
  url: string
  /** 窗口标题 */
  title: string
  /** 窗口宽度（可选，覆盖 manifest 默认值） */
  width?: number
  /** 窗口高度（可选，覆盖 manifest 默认值） */
  height?: number
}

/**
 * 插件可调用的 SDK 接口
 */
export interface PluginSdk {
  /**
   * 读取启动器配置
   *
   * 仅允许读取非敏感字段（如 gameDir / language / primaryColor），
   * 不允许读取 apiKey / token 等敏感信息。
   */
  getConfig(): Promise<Record<string, unknown>>

  /**
   * 读取已安装版本列表
   *
   * 返回版本 ID 数组，插件可用于显示统计信息。
   */
  listInstalledVersions(): Promise<string[]>

  /**
   * 读取已安装版本列表（带类型信息）
   *
   * 返回版本 ID + 加载器类型（vanilla / forge / fabric / neoforge / optifine / liteloader）。
   * 供「版本统计」等需要按加载器分类的插件使用。
   */
  listInstalledVersionsWithType(): Promise<
    Array<{ id: string; version_type: string; logo: string }>
  >

  /**
   * 读取启动历史记录
   *
   * 返回最近 50 条启动记录（按时间倒序）。
   * 历史记录仅在后端内存中累积，重启启动器后清空。
   */
  listLaunchHistory(): Promise<
    Array<{
      version_id: string
      username: string
      launch_time: string
      pid: number
      exit_code: number | null
    }>
  >

  /**
   * 读取系统内存信息
   *
   * 返回 total / used / available / usage_percent（单位：字节，百分比 0-100）。
   */
  getSystemMemory(): Promise<{
    total: number
    used: number
    available: number
    usage_percent: number
  }>

  /**
   * 获取当前运行中的游戏 PID（null 表示无游戏运行）
   */
  getRunningGamePid(): Promise<number | null>

  /**
   * 读取缓存统计信息
   *
   * 返回所有缓存目录的文件数、占用大小、TTL。
   * 供缓存监控、磁盘占用展示等插件使用。
   */
  getCacheStats(): Promise<CacheStatsResult>

  /**
   * 执行子进程命令（高级权限）
   *
   * **仅外部插件可用**：需要 manifest 声明 `spawnProcess` 权限 + `processPermissions` 配置。
   * 内置插件调用会抛出错误（内置插件有直接的后端访问能力，不需要此方法）。
   *
   * 安全限制：
   * - command 必须在 manifest.processPermissions.allowedCommands 白名单内
   * - 超时默认 30 秒，最大 5 分钟
   * - stdout/stderr 各截断到 1MB
   * - 非 shell 执行，防止注入
   *
   * @param command 要执行的命令（如 "java" / "node" / "python"）
   * @param args 命令参数数组
   * @param options 选项（cwd 工作目录）
   */
  spawnProcess(
    command: string,
    args: string[],
    options?: SpawnProcessOptions,
  ): Promise<ProcessResult>

  /**
   * 创建子窗口（高级权限）
   *
   * **仅外部插件可用**：需要 manifest 声明 `createWindow` 权限 + `windowPermissions` 配置。
   * 内置插件调用会抛出错误。
   *
   * 安全限制：
   * - URL 域名必须在 manifest.windowPermissions.allowedDomains 白名单内
   * - 单个插件最多同时存在 5 个窗口
   * - 窗口 label 使用 `plugin-<id>-<label>` 格式，避免与内置窗口冲突
   *
   * @param options 窗口选项（label / url / title / width / height）
   */
  createWindow(options: CreateWindowOptions): Promise<void>

  /**
   * 发送自定义事件
   *
   * 插件可向启动器发送自定义事件，由其他模块或插件监听。
   * 事件名必须以 "plugin:" 前缀开头，避免与启动器内置事件冲突。
   */
  emit(event: string, payload?: unknown): void

  /**
   * 记录日志（写入启动器日志文件）
   */
  log(level: 'info' | 'warn' | 'error', message: string): void
}

/**
 * 插件 SDK 实现
 *
 * 当前版本仅实现前端可完成的 API，后端命令预留扩展点。
 */
class PluginSdkImpl implements PluginSdk {
  async getConfig(): Promise<Record<string, unknown>> {
    // 调用 config_manager 的 get_config action，仅返回非敏感字段
    // 当前版本返回全部字段，后续可加白名单过滤
    const entries = await configManager<Array<{ key: string; value: unknown }>>(
      CONFIG_ACTIONS.GET_CONFIG,
      { keys: null },
    )
    const result: Record<string, unknown> = {}
    for (const e of entries) {
      // 过滤敏感字段
      if (e.key === 'curseforgeApiKey' || e.key === 'curseforgeEnabled') continue
      result[e.key] = e.value
    }
    return result
  }

  async listInstalledVersions(): Promise<string[]> {
    return versionListManager<string[]>(VERSION_LIST_ACTIONS.LIST_INSTALLED_VERSIONS)
  }

  async listInstalledVersionsWithType() {
    return versionListManager<Array<{ id: string; version_type: string; logo: string }>>(
      VERSION_LIST_ACTIONS.LIST_INSTALLED_VERSIONS_WITH_TYPE,
    )
  }

  async listLaunchHistory() {
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

  async getSystemMemory() {
    return systemManager<{
      total: number
      used: number
      available: number
      usage_percent: number
    }>(SYSTEM_ACTIONS.GET_SYSTEM_MEMORY)
  }

  async getRunningGamePid(): Promise<number | null> {
    return versionLaunchManager<number | null>(VERSION_LAUNCH_ACTIONS.GET_RUNNING_GAME)
  }

  async getCacheStats(): Promise<CacheStatsResult> {
    return systemManager<CacheStatsResult>(SYSTEM_ACTIONS.GET_CACHE_STATS)
  }

  async spawnProcess(
    _command: string,
    _args: string[],
    _options?: SpawnProcessOptions,
  ): Promise<ProcessResult> {
    // 内置插件直接调用会抛出错误
    // 外部插件的 spawnProcess 请求由 PluginSandbox 特殊拦截处理（注入 pluginId）
    throw new Error(
      'spawnProcess 仅外部插件可用（需通过沙箱桥接注入 pluginId 上下文）',
    )
  }

  async createWindow(_options: CreateWindowOptions): Promise<void> {
    // 内置插件直接调用会抛出错误
    // 外部插件的 createWindow 请求由 PluginSandbox 特殊拦截处理（注入 pluginId）
    throw new Error(
      'createWindow 仅外部插件可用（需通过沙箱桥接注入 pluginId 上下文）',
    )
  }

  emit(event: string, payload?: unknown): void {
    // 强制事件名前缀，避免与内置事件冲突
    if (!event.startsWith('plugin:')) {
      console.warn(`[PluginSdk] 事件名必须以 "plugin:" 开头: ${event}`)
      return
    }
    window.dispatchEvent(new CustomEvent(event, { detail: payload }))
  }

  log(level: 'info' | 'warn' | 'error', message: string): void {
    const prefix = '[Plugin]'
    switch (level) {
      case 'info':
        console.log(prefix, message)
        break
      case 'warn':
        console.warn(prefix, message)
        break
      case 'error':
        console.error(prefix, message)
        break
    }
  }
}

/** 插件 SDK 单例 */
export const pluginSdk: PluginSdk = new PluginSdkImpl()
