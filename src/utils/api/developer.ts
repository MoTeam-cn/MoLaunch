/**
 * 开发者模式 API（解锁/撤销触发 + 日志/缓存/存储/系统信息查询）
 *
 * 触发链：CreditsTab 隐藏字段连点 7 次解锁 → SettingsAdvanced 开关 → applyConfig
 * → Settings 侧边栏「开发者」项，撤销走 lockDeveloperMode()；
 * 解锁状态存 Windows 注册表 HKCU\Software\MoLaunch，developerMode 读写统一走 config API。
 */

import { SYSTEM_ACTIONS, systemManager } from './system-manager'
import { refreshConfig } from './config'

// ==================== 解锁 ====================

/** 查询开发者模式是否已解锁（用户在鸣谢法律信息中触发隐藏字段后解锁） */
export async function isDeveloperUnlocked(): Promise<boolean> {
  return systemManager<boolean>(SYSTEM_ACTIONS.IS_DEVELOPER_UNLOCKED)
}

/**
 * 解锁开发者模式（由 CreditsTab.vue 法律信息中的隐藏字段触发）
 *
 * 触发流程：隐藏字段连续点击 7 次 → 本函数 → SettingsAdvanced 显示开关卡片
 * （仅已解锁时）→ applyConfig({ developerMode: true/false }) 开启开关 →
 * Settings 侧边栏出现「开发者」项；撤销由 DevModeToggle.vue 底部按钮调用
 * lockDeveloperMode()（后端重置 DeveloperUnlocked/DeveloperMode/IgnoreTls 并关闭 DevTools）。
 * 存储：Windows 注册表 HKCU\Software\MoLaunch 下 DeveloperUnlocked / DeveloperMode 两个布尔值。
 */
export async function unlockDeveloperMode(): Promise<void> {
  await systemManager<void>(SYSTEM_ACTIONS.UNLOCK_DEVELOPER_MODE)
  // 解锁改写了注册表 DeveloperUnlocked，而 get_config 快照有全局缓存；
  // 清空缓存确保「进阶设置」页 DevModeToggle 重新读取到解锁状态
  // （否则缓存仍是旧值 developerUnlocked=false，开关卡片不显示）。
  refreshConfig()
}

/**
 * 撤销开发者模式解锁
 *
 * 后端同时重置 `DeveloperUnlocked` / `DeveloperMode` / `IgnoreTls` 三个注册表项，
 * 并关闭已打开的 DevTools。调用后开发者相关能力全部失效。
 *
 * 调用方应在二次确认后调用，并监听 `developer-mode-changed` 事件（payload=false）
 * 更新侧边菜单显隐。
 */
export async function lockDeveloperMode(): Promise<void> {
  await systemManager<void>(SYSTEM_ACTIONS.LOCK_DEVELOPER_MODE)
  // 撤销改写了注册表三个键，同样清空配置缓存，保证下次读取拿到最新状态。
  refreshConfig()
}

// ==================== 存储目录 ====================

export interface StorageDirs {
  /** 数据根目录（.Molaunch） */
  base: string
  /** 配置文件（config.ini 完整路径） */
  config: string
  /** 日志目录 */
  logs: string
  /** 运行路径缓存目录（.Molaunch/cache/） */
  cache: string
  /** 运行路径临时目录（.Molaunch/temp/） */
  temp: string
  /** 系统临时目录缓存（<temp>/MoLaunch/，含 TaskTemp 和 sdk） */
  cacheTemp: string
  /** Minecraft 运行缓存目录（%APPDATA%/.minecraft/，Java Runtime） */
  cacheApp: string
  /** AppData 全局共享根目录（%APPDATA%/.Molaunch/，跨实例共享） */
  appdataRoot: string
  /** 全局共享 TLS 证书目录（AppData/certs/） */
  appdataCerts: string
  /** 全局共享 frpc 厂商二进制目录（AppData/providers/） */
  appdataProviders: string
  /** 全局共享 FRP 认证 token 目录（AppData/frp_auth/，SDK DES 加密） */
  appdataFrpAuth: string
  /** 全局共享联机数据目录（AppData/online/） */
  appdataOnline: string
  /** 全局共享账号认证文件（AppData/auth.json） */
  appdataAuthFile: string
}

/** 获取所有存储目录路径 */
export async function getStorageDirs(): Promise<StorageDirs> {
  return systemManager<StorageDirs>(SYSTEM_ACTIONS.GET_STORAGE_DIRS)
}

// ==================== 系统信息 ====================

export interface SystemInfo {
  /** 应用版本 */
  appVersion: string
  /** 操作系统（windows/macos/linux） */
  os: string
  /** 系统架构（x86_64/aarch64） */
  arch: string
  /** 是否 64 位 */
  is64bit: boolean
  /** 总内存（字节） */
  totalMemory: number
  /** 已用内存（字节） */
  usedMemory: number
  /** 可用内存（字节） */
  availableMemory: number
  /** 内存使用率（百分比） */
  memoryUsagePercent: number
}

/** 获取系统信息 */
export async function getSystemInfo(): Promise<SystemInfo> {
  return systemManager<SystemInfo>(SYSTEM_ACTIONS.GET_SYSTEM_INFO)
}

// ==================== 更新检测分支 ====================
//
// 覆盖更新检查请求携带的 channel 参数，用于跨分支更新检测调试。
// set 需开发者模式已开启（后端 require_dev_mode 校验）。

/** 更新检测分支值（'auto' 表示跟随版本后缀自动推导） */
export type UpdateBranch = 'auto' | 'stable' | 'beta' | 'alpha'

/** 获取更新检测分支覆盖 */
export async function getUpdateBranch(): Promise<UpdateBranch> {
  return systemManager<UpdateBranch>(SYSTEM_ACTIONS.GET_UPDATE_BRANCH)
}

/** 设置更新检测分支覆盖（影响检查更新时的 channel 参数） */
export async function setUpdateBranch(branch: UpdateBranch): Promise<void> {
  return systemManager<void>(SYSTEM_ACTIONS.SET_UPDATE_BRANCH, { branch })
}

// ==================== 缓存统计 ====================

/** 单个缓存子目录的统计信息 */
export interface CacheStat {
  /** 显示名称（如 "图片缓存"） */
  name: string
  /** 所属类别（"cache" / "cacheTemp" / "cacheApp"） */
  category: string
  /** 子目录相对路径（如 "images" / "TaskTemp" / "runtime"） */
  subDir: string
  /** 完整路径 */
  path: string
  /** 文件数量（递归统计） */
  fileCount: number
  /** 占用字节数（递归统计） */
  totalSize: number
  /** 自动清理 TTL（小时），null 表示不清理 */
  ttlHours: number | null
}

/** 缓存统计结果（按类别分组） */
export interface CacheStatsResult {
  /** 运行路径缓存（.Molaunch/cache/） */
  cache: CacheStat[]
  /** 系统临时目录缓存（<temp>/MoLaunch/） */
  cacheTemp: CacheStat[]
  /** AppData 缓存（%APPDATA%/.minecraft/） */
  cacheApp: CacheStat[]
}

/** 获取所有缓存目录的统计信息（文件数、占用大小、TTL） */
export async function getCacheStats(): Promise<CacheStatsResult> {
  return systemManager<CacheStatsResult>(SYSTEM_ACTIONS.GET_CACHE_STATS)
}

// ==================== 日志查看 ====================

/** 获取今日日志文件完整路径 */
export async function getLogPath(): Promise<string> {
  return systemManager<string>(SYSTEM_ACTIONS.GET_LOG_PATH)
}

/** 获取所有日志文件名列表（最新的在前） */
export async function listLogFiles(): Promise<string[]> {
  return systemManager<string[]>(SYSTEM_ACTIONS.LIST_LOG_FILES)
}

/** 读取指定日志文件内容 */
export async function readLogFile(filename: string): Promise<string> {
  return systemManager<string>(SYSTEM_ACTIONS.READ_LOG_FILE, { filename })
}

// ==================== HTTP 请求日志 ====================

/** HTTP 日志条目（结构化，供表格展示） */
export interface HttpLogEntry {
  /** 时间戳（本地时间，`YYYY-MM-DD HH:MM:SS.mmm`） */
  timestamp: string
  /** HTTP 方法（GET/POST/PUT/DELETE） */
  method: string
  /** 请求路径（不含 base_url，如 `/v3/auth/refresh`） */
  path: string
  /** HTTP 状态码 */
  status: number
  /** 响应中的 `req_id`（可能为空） */
  reqId: string
}

/**
 * 读取 HTTP 请求日志（联机 API 调用追踪）
 *
 * @param date 日期字符串（`YYYY-MM-DD`），不传表示今天
 * @param limit 最多返回条数（从末尾截取最新的），不传表示全部
 */
export async function readHttpLogs(
  date?: string,
  limit?: number,
): Promise<HttpLogEntry[]> {
  return systemManager<HttpLogEntry[]>(SYSTEM_ACTIONS.READ_HTTP_LOGS, { date, limit })
}

/** 列出所有 HTTP 日志文件名（`http_*.log`，最新的在前） */
export async function listHttpLogFiles(): Promise<string[]> {
  return systemManager<string[]>(SYSTEM_ACTIONS.LIST_HTTP_LOG_FILES)
}

// ==================== 深链接（deeplink） ====================
//
// molaunch:// 协议注册管理。安装版由 NSIS 安装时自动注册；
// 便携版（未安装）需在此手动注册/卸载。

/** molaunch:// 协议注册状态 */
export interface DeeplinkStatus {
  /** 协议当前是否已注册 */
  registered: boolean
  /** 注册表中登记的 exe 路径（未注册为 null） */
  registeredExe: string | null
  /** 当前运行 exe 路径（获取失败为 null） */
  currentExe: string | null
  /** 当前平台是否支持运行时注册/卸载（macOS 不支持） */
  platformSupported: boolean
  /** 人类可读说明 */
  message: string
}

/** 查询 molaunch:// 协议当前注册状态 */
export async function getDeeplinkStatus(): Promise<DeeplinkStatus> {
  return systemManager<DeeplinkStatus>(SYSTEM_ACTIONS.GET_DEEPLINK_STATUS)
}

/** 注册 molaunch:// 协议（幂等，便携版/开发环境手动触发） */
export async function registerDeeplink(): Promise<DeeplinkStatus> {
  return systemManager<DeeplinkStatus>(SYSTEM_ACTIONS.REGISTER_DEEPLINK)
}

/** 卸载 molaunch:// 协议（幂等，注册后协议链接将提示无应用处理） */
export async function unregisterDeeplink(): Promise<DeeplinkStatus> {
  return systemManager<DeeplinkStatus>(SYSTEM_ACTIONS.UNREGISTER_DEEPLINK)
}

// ==================== TLS 证书管理 ====================

/** 自定义证书信息（list_custom_certs 返回项） */
export interface CustomCertInfo {
  /** 文件名（certs 目录下的相对名称，如 `my-root.pem`） */
  filename: string
  /** 证书 Subject CN（解析失败时回退为文件名） */
  subject: string
  /** 证书过期时间（PEM 解析失败时为空字符串） */
  notAfter: string
}

/** 列出 certs 目录下所有 `.pem` 文件 */
export async function listCustomCerts(): Promise<CustomCertInfo[]> {
  return systemManager<CustomCertInfo[]>(SYSTEM_ACTIONS.LIST_CUSTOM_CERTS)
}

/** 添加自定义证书（从源路径复制 PEM 文件到 certs 目录） */
export async function addCustomCert(path: string): Promise<void> {
  return systemManager<void>(SYSTEM_ACTIONS.ADD_CUSTOM_CERT, { path })
}

/** 删除自定义证书（按文件名删除 certs 目录下对应文件） */
export async function removeCustomCert(filename: string): Promise<void> {
  return systemManager<void>(SYSTEM_ACTIONS.REMOVE_CUSTOM_CERT, { filename })
}

// ==================== DevTools 控制 ====================
//
// 调用前要求开发者模式已解锁且已开启，后端 require_dev_mode() 双层校验。
// 普通用户即使绕过前端按钮直接调 IPC 也无法触发。

/** 打开主窗口的 WebView2 DevTools（开发者模式开启时可调用） */
export async function openDevTools(): Promise<void> {
  return systemManager<void>(SYSTEM_ACTIONS.OPEN_DEVTOOLS)
}

/** 关闭主窗口的 WebView2 DevTools */
export async function closeDevTools(): Promise<void> {
  return systemManager<void>(SYSTEM_ACTIONS.CLOSE_DEVTOOLS)
}

/** 查询主窗口的 DevTools 是否已打开（开发者模式未开启时返回 false） */
export async function isDevToolsOpen(): Promise<boolean> {
  return systemManager<boolean>(SYSTEM_ACTIONS.IS_DEVTOOLS_OPEN)
}
