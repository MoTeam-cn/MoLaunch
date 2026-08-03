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