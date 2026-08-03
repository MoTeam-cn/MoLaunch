/**
 * 插件权限元信息表：定义所有权限的说明 / 用处 / 风险等级 / 参数要求，作为单一数据源供 SettingsPlugins.vue 展示与文档生成复用。
 * 权限分类：始终允许（emit/log 无需声明）、普通权限（只读低风险）、高级权限（子进程等高风险，需 processPermissions）。
 */

/** 权限风险等级 */
export type PermissionRisk = 'low' | 'medium' | 'high'

/** 权限元信息 */
export interface PermissionMeta {
  /** 权限名（对应 manifest.json 的 permissions 数组项） */
  name: string
  /** 简短说明（一句话） */
  description: string
  /** 具体用途回答「插件拿这个权限能做什么」 */
  useCase: string
  /** 风险等级 */
  risk: PermissionRisk
  /** 是否始终允许（无需在 manifest 声明） */
  alwaysAllowed?: boolean
  /** 是否需要额外配置字段（如 spawnProcess 需要 processPermissions） */
  requiresExtraConfig?: string
}

/**
 * 全部权限元信息列表
 *
 * 顺序即为 UI 展示顺序：始终允许 → 普通 → 高级
 */
export const PERMISSION_REGISTRY: PermissionMeta[] = [
  // ========== 始终允许 ==========
  {
    name: 'emit',
    description: '发送自定义事件',
    useCase: '插件向启动器或其他插件发送自定义事件，事件名必须以 "plugin:" 前缀开头',
    risk: 'low',
    alwaysAllowed: true,
  },
  {
    name: 'log',
    description: '记录日志',
    useCase: '将插件运行日志写入启动器日志文件，便于调试和问题排查',
    risk: 'low',
    alwaysAllowed: true,
  },

  // ========== 普通权限（只读数据访问） ==========
  {
    name: 'getConfig',
    description: '读取启动器配置',
    useCase: '获取游戏目录、语言、主题色等非敏感配置项（API Key、Token 等敏感字段已过滤）',
    risk: 'low',
  },
  {
    name: 'listInstalledVersions',
    description: '读取已安装版本列表',
    useCase: '获取已安装的游戏版本 ID 数组，用于版本统计或展示',
    risk: 'low',
  },
  {
    name: 'listInstalledVersionsWithType',
    description: '读取已安装版本列表（含加载器类型）',
    useCase: '获取版本 ID + 加载器类型（vanilla / forge / fabric 等），用于按加载器分类统计',
    risk: 'low',
  },
  {
    name: 'listLaunchHistory',
    description: '读取启动历史记录',
    useCase: '获取最近 50 条启动记录（版本、用户名、启动时间、退出码），用于启动历史展示',
    risk: 'low',
  },
  {
    name: 'getSystemMemory',
    description: '读取系统内存信息',
    useCase: '获取系统总内存、已用内存、可用内存、使用率，用于系统监控展示',
    risk: 'low',
  },
  {
    name: 'getRunningGamePid',
    description: '获取运行中的游戏进程 ID',
    useCase: '查询当前是否有游戏在运行及其 PID，用于游戏状态监控',
    risk: 'low',
  },
  {
    name: 'getCacheStats',
    description: '读取缓存统计信息',
    useCase: '获取各缓存目录的文件数、占用大小、TTL，用于缓存监控或磁盘占用展示',
    risk: 'low',
  },

  // ========== 高级权限（高风险） ==========
  {
    name: 'spawnProcess',
    description: '执行子进程命令',
    useCase: '启动外部程序（如 java -version 检测版本、node 执行脚本、python 处理数据），提供高自定义化能力',
    risk: 'high',
    requiresExtraConfig: 'processPermissions',
  },
  {
    name: 'createWindow',
    description: '创建子窗口',
    useCase: '弹出独立窗口加载指定 URL 页面（如打开插件文档、配置页面、外部工具界面），URL 域名必须在白名单内',
    risk: 'high',
    requiresExtraConfig: 'windowPermissions',
  },
]

/** 始终允许的权限名列表（无需在 manifest 声明） */
export const ALWAYS_ALLOWED_PERMISSIONS = PERMISSION_REGISTRY
  .filter((p) => p.alwaysAllowed)
  .map((p) => p.name)

/** 普通权限列表（低/中风险，需要声明） */
export const NORMAL_PERMISSIONS = PERMISSION_REGISTRY.filter(
  (p) => !p.alwaysAllowed && p.risk !== 'high',
)

/** 高级权限列表（高风险，需要声明 + 额外配置） */
export const ADVANCED_PERMISSIONS = PERMISSION_REGISTRY.filter((p) => p.risk === 'high')

/** 风险等级对应的颜色和标签 */
export const RISK_STYLES: Record<PermissionRisk, { label: string; bg: string; text: string }> = {
  low: { label: '低风险', bg: 'bg-green-50', text: 'text-green-700' },
  medium: { label: '中风险', bg: 'bg-yellow-50', text: 'text-yellow-700' },
  high: { label: '高风险', bg: 'bg-red-50', text: 'text-red-700' },
}

/**
 * 根据权限名获取元信息
 */
export function getPermissionMeta(name: string): PermissionMeta | undefined {
  return PERMISSION_REGISTRY.find((p) => p.name === name)
}
