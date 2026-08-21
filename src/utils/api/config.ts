/**
 * 全局配置 API（统一读写入口 + 全局缓存）
 *
 * 配置更新统一走 applyConfig(patch)（由 ConfigPatch 取代历史 set_* 函数）；
 * 底层经 config_manager 聚合分发，get_config_path / save_config_to_file 仍走独立 invoke。
 */

import { CONFIG_ACTIONS, configManager } from './config-manager'
import type { GithubProxy } from '@/types/online'

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
  /** 关闭主窗口时的行为："ask"（每次询问）/ "tray"（保留托盘）/ "exit"（直接退出） */
  closeBehavior: string
  /** 实验性功能开关（开启后顶部导航显示「实验性」入口并初始化 SQLite 聊天存储） */
  experimentalEnabled: boolean
  /** 启动器界面 GPU 硬件加速（默认开启；关闭后 WebView2 走软件渲染，需重启生效） */
  useGpuAcceleration: boolean
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
  // 联机（api-server 地址 / 公共 easytier 节点）
  onlineApiServerUrl: string
  /** 虚拟网络内设备名（房客侧 easytier hostname；留空使用默认 mo-launch-guest） */
  onlineNetworkIdentity: string
  /** 公共 easytier 节点列表（--peers 参数；信令节点与中继节点均可；默认信令节点内置，前端不展示） */
  onlineEasytierPublicPeers: string[]
  /** 用户自定义 GitHub 镜像源（easytier 等外部下载竞速选源用，type: path / full） */
  onlineGithubProxies: GithubProxy[]
  // TLS 证书
  /** TLS 信任源模式：builtin / system / custom / system+custom / builtin+custom / all */
  tlsTrustMode: string
  /** 是否忽略 TLS 证书校验（开发者模式注册表键，仅开发者模式开启时生效） */
  ignoreTls: boolean
  // 正版购买提示（系统存储：Windows 注册表 / 其他系统全局共用文件）
  /** 游戏启动成功次数（正版购买提示计数） */
  launchCount: number
  /** 是否已永久忽略正版购买提示 */
  hintBuy: boolean
  /** 是否已永久忽略"去 GitHub 点 Star"提示 */
  hintStar: boolean
  // 用户协议（系统存储：全局首次启动门禁）
  /** 是否已同意《用户协议》 */
  userAgreed: boolean
  /** 已同意的《用户协议》版本号（0 表示从未同意） */
  userAgreedVersion: number
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
  /** 关闭主窗口时的行为："ask"（每次询问）/ "tray"（保留托盘）/ "exit"（直接退出） */
  closeBehavior?: string
  /** 实验性功能开关（开启后显示「实验性」入口并惰性初始化 SQLite 聊天存储） */
  experimentalEnabled?: boolean
  /** 启动器界面 GPU 硬件加速（关闭后 WebView2 走软件渲染，需重启生效） */
  useGpuAcceleration?: boolean
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
  // 联机（api-server 地址，空字符串后端会忽略不更新）
  onlineApiServerUrl?: string
  /** 虚拟网络内设备名（房客侧 easytier hostname；空字符串清空回退默认 mo-launch-guest） */
  onlineNetworkIdentity?: string
  /** 公共 easytier 节点列表（--peers 参数；空数组表示清空） */
  onlineEasytierPublicPeers?: string[]
  /** 用户自定义 GitHub 镜像源（空数组表示清空） */
  onlineGithubProxies?: GithubProxy[]
  // TLS 证书
  /** TLS 信任源模式：builtin / system / custom / system+custom / builtin+custom / all */
  tlsTrustMode?: string
  /** 是否忽略 TLS 证书校验（仅开发者模式开启时可生效） */
  ignoreTls?: boolean
  // 正版购买提示（系统存储，后端内部分流到 Windows 注册表 / 其他系统全局共用文件）
  /** 游戏启动成功次数（正版购买提示计数） */
  launchCount?: number
  /** 是否永久忽略正版购买提示 */
  hintBuy?: boolean
  /** 是否永久忽略"去 GitHub 点 Star"提示 */
  hintStar?: boolean
  // 用户协议（系统存储：全局首次启动门禁）
  /** 是否已同意《用户协议》 */
  userAgreed?: boolean
  /** 已同意的《用户协议》版本号 */
  userAgreedVersion?: number
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
