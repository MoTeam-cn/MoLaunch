/**
 * 全局配置 API（统一读写入口 + 全局缓存）
 *
 * 重构后所有配置更新统一走 `applyConfig(patch)`，仅传需要改的字段。
 * 此前分散的 17 个 set_* 函数已移除，由 ConfigPatch 的对应字段取代。
 *
 * 从 system.ts 拆分而来：系统操作/下载进度查询仍保留在 system.ts。
 *
 * 注：底层 `get_config` / `apply_config` 2 个命令已聚合为 `config_manager`
 * 单一 IPC 入口，通过 `action` 字段分发。
 * `get_config_path` / `save_config_to_file` 仍走独立 invoke（不在聚合范围）。
 */

import { CONFIG_ACTIONS, configManager } from './config-manager'
import type { IceServerEntry } from '@/types/online'

// ==================== 配置快照与补丁类型 ====================

/**
 * 配置快照：返回所有配置字段的当前值。
 *
 * 对应后端 `ConfigSnapshot` 结构体（camelCase 序列化）。
 * CurseForge 的 apiKey 从内存缓存读取（已解密）。
 */
export interface ConfigSnapshot {
  // 代理
  proxyMode: string
  proxyType: string
  proxyUrl: string
  /** IP 协议版本偏好："v4"（强制 IPv4）/ "auto"（自动测试）/ "any"（跟随 DNS） */
  ipVersion: string
  // 下载
  mirrorUrl: string | null
  downloadSource: string
  metaSource: string
  maxDownloadSpeed: number
  maxDownloadThreads: number
  chunkCount: number
  /** Modrinth CDN 直连开关（开发者模式可见，默认 false） */
  modrinthCdnRawEnabled: boolean
  // 内存
  memoryMode: string
  minMemory: number
  maxMemory: number
  // 启动器
  gameDir: string
  isolationMode: number
  logLevel: number
  selectedVersion: string | null
  /** 游戏默认界面语言（写入 options.txt 的 lang 字段，默认 "zh_cn"） */
  gameLanguage: string
  /** 主题主色 HEX（如 "#165dff"），驱动 primary-* 色阶 */
  primaryColor: string
  // 社区资源
  communitySource: number
  communityFilenameFormat: number
  communityModLocalNameStyle: number
  communityIgnoreQuilt: boolean
  // CurseForge（已解密）
  curseforgeEnabled: boolean
  curseforgeApiKey: string
  // 启动高级选项
  launchDisableJlw: boolean
  launchDisableLua: boolean
  launchUseDedicatedGpu: boolean
  // 外部下载工具
  externalDownloadDir: string | null
  /** Java 路径（从 INI [Java] path 读取，不进 AppConfig） */
  javaPath: string | null
  // 开发者模式（从注册表读，developerUnlocked 为只读）
  developerUnlocked: boolean
  developerMode: boolean
  // 联机（api-server 地址 + 用户自定义 TURN 服务器列表）
  onlineApiServerUrl: string
  /** 用户自定义 TURN 服务器列表（阶段三子任务 7 新增） */
  onlineCustomTurnServers: IceServerEntry[]
  // TLS 证书
  /** TLS 信任源模式：builtin / system / custom / system+custom / builtin+custom / all */
  tlsTrustMode: string
  /** 是否忽略 TLS 证书校验（开发者模式注册表键，仅开发者模式开启时生效） */
  ignoreTls: boolean
}

/**
 * 配置补丁：所有字段可选，仅传需要更新的字段。
 * 未传的字段保持原值不变。
 *
 * 对应后端 `ConfigPatch` 结构体（camelCase 序列化）。
 */
export interface ConfigPatch {
  // 代理
  proxyMode?: string
  proxyType?: string
  proxyUrl?: string
  /** IP 协议版本偏好："v4" / "auto" / "any" */
  ipVersion?: string
  // 下载
  downloadSource?: string                                  // "official" / "mirror" / "smart"
  metaSource?: string                                      // "official" / "mirror" / "smart"
  maxDownloadSpeed?: number
  maxDownloadThreads?: number
  chunkCount?: number
  mirrorUrl?: string | null                               // null 表示清空
  /** Modrinth CDN 直连开关（开发者模式可见，默认 false） */
  modrinthCdnRawEnabled?: boolean
  // 内存
  memoryMode?: string                                      // "auto" / "custom"
  minMemory?: number
  maxMemory?: number
  // 启动器
  gameDir?: string
  isolationMode?: number
  logLevel?: number
  selectedVersion?: string | null                          // null 表示清空选中
  /** 游戏默认界面语言："zh_cn" 等 MC 代码 / "none"（不设置）/ "auto"（旧配置兼容） */
  gameLanguage?: string
  /** 主题主色 HEX（如 "#165dff"） */
  primaryColor?: string
  // 社区资源（INI 明文）
  communitySource?: number                                 // 0/1/2
  communityFilenameFormat?: number                         // 0-4
  communityModLocalNameStyle?: number                      // 0/1
  communityIgnoreQuilt?: boolean
  // CurseForge（加密存储，后端内部分流到 secure_storage）
  curseforgeEnabled?: boolean
  curseforgeApiKey?: string
  // 启动高级选项
  launchDisableJlw?: boolean
  launchDisableLua?: boolean
  launchUseDedicatedGpu?: boolean
  // 外部下载工具（null 表示清空，回退默认 .Molaunch/Download/）
  externalDownloadDir?: string | null
  /** Java 路径（独立存储于 INI [Java] path，不进 AppConfig） */
  javaPath?: string
  // 开发者模式（注册表存储，后端内部分流到 registry，仅已解锁时可生效）
  developerMode?: boolean
  // 联机（api-server 地址，空字符串后端会忽略不更新；自定义 TURN 列表）
  onlineApiServerUrl?: string
  /**
   * 用户自定义 TURN 服务器列表（阶段三子任务 7 新增）
   *
   * 传空数组表示清空所有自定义 TURN；不传（undefined）表示不更新。
   */
  onlineCustomTurnServers?: IceServerEntry[]
  // TLS 证书
  /** TLS 信任源模式：builtin / system / custom / system+custom / builtin+custom / all */
  tlsTrustMode?: string
  /** 是否忽略 TLS 证书校验（仅开发者模式开启时可生效） */
  ignoreTls?: boolean
}

// ==================== 配置缓存与读写（带全局缓存）====================

/**
 * 单个配置项（扁平化 key-value 对）
 *
 * `get_config` IPC 返回 `ConfigEntry[]`，每项形如 `{ key: "proxyMode", value: "none" }`。
 */
export interface ConfigEntry {
  key: string
  value: unknown
}

/**
 * 全局配置缓存。
 *
 * - 首次 `getConfigMap()` 请求后端并写入此处
 * - 切换侧栏时各组件直接读缓存，不再重复 IPC
 * - `applyConfig(patch)` 保存成功后用 patch 同步更新缓存
 * - `refreshConfig()` 清空缓存强制下次重新请求
 */
let configCache: ConfigSnapshot | null = null
let configPromise: Promise<ConfigEntry[]> | null = null

/**
 * 读取配置（扁平化数组格式，带全局缓存）。
 *
 * - 不传 `keys`：返回全部字段
 * - 传 `keys`：仅返回指定字段（camelCase 名称）
 * - 传 `force=true`：强制清空缓存重新请求后端
 *
 * 返回 `ConfigEntry[]`，格式为 `[{ key, value }, ...]`。
 */
export async function getConfig(keys?: string[], force?: boolean): Promise<ConfigEntry[]> {
  // 强制刷新：清空缓存
  if (force) {
    configCache = null
    configPromise = null
  }

  // 无缓存：发起请求（合并并发请求为单个 Promise）
  if (!configCache) {
    if (!configPromise) {
      configPromise = configManager<ConfigEntry[]>(CONFIG_ACTIONS.GET_CONFIG, {
        keys: keys && keys.length > 0 ? keys : null,
      })
        .then((entries) => {
          // 将数组转换为对象缓存
          const snap: Record<string, unknown> = {}
          for (const e of entries) {
            snap[e.key] = e.value
          }
          configCache = snap as unknown as ConfigSnapshot
          return entries
        })
        .finally(() => {
          configPromise = null
        })
    }
    return configPromise.then((entries) => {
      if (keys && keys.length > 0) {
        const filter = new Set(keys)
        return entries.filter((e) => filter.has(e.key))
      }
      return entries
    })
  }

  // 有缓存：从缓存构建数组
  const entries: ConfigEntry[] = []
  const cache = configCache as unknown as Record<string, unknown>
  const filter = keys && keys.length > 0 ? new Set(keys) : null
  for (const [k, v] of Object.entries(cache)) {
    if (!filter || filter.has(k)) {
      entries.push({ key: k, value: v })
    }
  }
  return Promise.resolve(entries)
}

/**
 * 读取配置（对象格式，带全局缓存）。
 *
 * 返回 `ConfigSnapshot` 对象（`{ proxyMode: "none", ... }`），方便组件按字段名访问。
 * 首次调用请求后端并缓存，后续切换侧栏直接读缓存，不再重复 IPC。
 *
 * - 传 `force=true`：强制清空缓存重新请求后端
 */
export async function getConfigMap(force?: boolean): Promise<ConfigSnapshot> {
  if (force) {
    configCache = null
    configPromise = null
  }
  if (configCache) {
    return configCache
  }
  // 触发 getConfig 的请求并等待缓存写入
  await getConfig()
  return configCache as unknown as ConfigSnapshot
}

/**
 * 统一配置更新命令（与 `getConfig` 格式对称）
 *
 * 前端调用时传 `ConfigPatch` 对象（`{ communitySource: 0 }`），内部转为
 * `ConfigEntry[]` 数组（`[{ key: 'communitySource', value: 0 }]`）再 IPC，
 * 与 `getConfig` 返回的格式完全一致，前后端对称。
 *
 * 仅传需要更新的字段，后端在单次事务内完成字段赋值与联动。
 * 保存成功后自动同步更新本地缓存，无需手动刷新。
 *
 * `mirrorUrl` / `selectedVersion` 使用 `null` 表示清空（与 `getConfig` 返回格式一致）。
 */
export async function applyConfig(patch: ConfigPatch): Promise<void> {
  // 把对象转为 ConfigEntry[] 数组，与 getConfig 返回格式对称
  const entries: ConfigEntry[] = []
  for (const [key, value] of Object.entries(patch)) {
    if (value !== undefined) {
      entries.push({ key, value })
    }
  }
  await configManager<void>(CONFIG_ACTIONS.APPLY_CONFIG, { entries })
  // 同步更新缓存（乐观更新，避免下次 getConfig 读到旧值）
  if (configCache) {
    Object.assign(configCache as object, patch)
  }
}

/**
 * 清空配置缓存，强制下次 `getConfig()` / `getConfigMap()` 重新请求后端。
 *
 * 适用于外部修改了配置文件、或需要确保读到最新值的场景。
 */
export function refreshConfig(): void {
  configCache = null
  configPromise = null
}
