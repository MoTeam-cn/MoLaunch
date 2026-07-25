/**
 * 版本导出管理统一 API 入口
 *
 * 后端 `version_export_manager` IPC 命令通过 `action` 字段分发到不同子模块
 * （get_export_options / export_modpack / save_export_config / load_export_config 共 4 个 action），
 * 参照 `version_launch_manager` 等其他 manager 模式。
 *
 * 字段名约定：后端 Params 结构体使用 `#[serde(rename_all = "camelCase")]`，
 * 故前端 params 对象的字段名一律使用 camelCase。
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 调用 version_export_manager IPC
 * @param action 操作名称（取自 VERSION_EXPORT_ACTIONS 常量）
 * @param params 参数对象（字段名使用 camelCase）
 */
export async function versionExportManager<T = unknown>(
  action: string,
  params?: unknown,
): Promise<T> {
  return invoke<T>('version_export_manager', { req: { action, params: params ?? null } })
}

/**
 * 所有可用的 action 名称
 *
 * 与后端 `utils::version_export_manager::DISPATCHER` 注册的 action 一一对应。
 */
export const VERSION_EXPORT_ACTIONS = {
  GET_EXPORT_OPTIONS: 'get_export_options',
  EXPORT_MODPACK: 'export_modpack',
  SAVE_EXPORT_CONFIG: 'save_export_config',
  LOAD_EXPORT_CONFIG: 'load_export_config',
} as const

/** action 名称类型 */
export type VersionExportAction = typeof VERSION_EXPORT_ACTIONS[keyof typeof VERSION_EXPORT_ACTIONS]

// ============================================================
// 数据类型（与后端 types.rs 对应，字段名 camelCase）
// ============================================================

/** 单个导出选项 */
export interface ExportOption {
  /** 选项唯一标识（如 "basic"、"mods"、"resourcepacks"） */
  id: string
  /** 显示标题（如 "Mod"、"资源包"） */
  title: string
  /** 描述（可空） */
  description?: string | null
  /** 文件匹配规则（`|` 分隔，`!` 开头表排除） */
  rules?: string | null
  /** 仅用于判断是否显示（不参与导出），为空时用 rules 判断 */
  showRules?: string | null
  /** 是否默认勾选 */
  defaultChecked: boolean
  /** 是否被勾选（由用户操作，导出时回传） */
  checked: boolean
  /** 父选项 id（null=顶层，string=子选项） */
  parent?: string | null
  /** 是否可用（如必选项为 false） */
  enabled: boolean
  /** 是否可见（根据实际文件扫描结果决定） */
  visible: boolean
}

/**
 * 导出整合包格式
 *
 * 与导入支持的格式对齐（除 LauncherPack 外），后端 serde rename_all="camelCase"
 * 故前端使用小写驼峰字符串。
 */
export type ExportFormat = 'modrinth' | 'curseforge' | 'hmcl' | 'mmc' | 'mcbbs' | 'compress'

/**
 * 导出格式元信息（用于 UI 展示）
 */
export interface ExportFormatOption {
  /** 格式值（传给后端） */
  value: ExportFormat
  /** 显示标题 */
  label: string
  /** 简短描述 */
  description: string
  /** 文件扩展名（不含 `.`，如 'mrpack' / 'zip'） */
  extension: 'mrpack' | 'zip'
  /** 是否支持联网检查 mod 下载地址 */
  supportsOnlineCheck: boolean
}

/**
 * 所有支持的导出格式（与后端 ExportFormat 枚举一致）
 */
export const EXPORT_FORMAT_OPTIONS: ExportFormatOption[] = [
  {
    value: 'modrinth',
    label: 'Modrinth',
    description: '生成 modrinth.index.json + overrides/，可上传到 Modrinth，可被其他启动器导入',
    extension: 'mrpack',
    supportsOnlineCheck: true,
  },
  {
    value: 'curseforge',
    label: 'CurseForge',
    description: '生成 manifest.json + modlist.html + overrides/，可被 HMCL/MMC 等启动器导入',
    extension: 'zip',
    supportsOnlineCheck: true,
  },
  {
    value: 'hmcl',
    label: 'HMCL',
    description: '生成 modpack.json + minecraft/，所有 mod 直接打包（体积较大）',
    extension: 'zip',
    supportsOnlineCheck: false,
  },
  {
    value: 'mmc',
    label: 'MultiMC',
    description: '生成 mmc-pack.json + instance.cfg + .minecraft/，所有 mod 直接打包',
    extension: 'zip',
    supportsOnlineCheck: false,
  },
  {
    value: 'mcbbs',
    label: 'MCBBS',
    description: '生成 mcbbs.packmeta + overrides/，所有 mod 直接打包',
    extension: 'zip',
    supportsOnlineCheck: false,
  },
  {
    value: 'compress',
    label: '普通压缩包',
    description: '直接打包 .minecraft/ 目录，无 manifest 文件，需手动指定游戏版本',
    extension: 'zip',
    supportsOnlineCheck: false,
  },
]

/** 根据 format 值查找格式元信息 */
export function findExportFormat(value: ExportFormat): ExportFormatOption {
  return EXPORT_FORMAT_OPTIONS.find(o => o.value === value) ?? EXPORT_FORMAT_OPTIONS[0]
}

/** 导出请求参数 */
export interface ExportModpackParams {
  /** 版本 ID（如 "1.20.1-Forge"） */
  versionId: string
  /** 整合包名称 */
  packName: string
  /** 整合包版本号（如 "1.0.0"） */
  packVersion: string
  /** 用户勾选的导出选项（含 checked 状态） */
  options: ExportOption[]
  /** 是否联网检查 mod 下载地址（true=联网，false=直接打包文件） */
  checkHostedAssets?: boolean
  /** 仅从 Modrinth 查询（true=跳过 CurseForge） */
  modrinthUploadMode?: boolean
  /** 导出文件保存路径（由前端文件对话框选择） */
  configPackPath?: string | null
  /** 导出格式（默认 modrinth） */
  format?: ExportFormat
}

/** 导出结果 */
export interface ExportModpackResult {
  success: boolean
  filePath: string
  fileSize: number
  /** 打包的文件总数 */
  fileCount: number
  /** 联网获取到下载地址的 mod 数 */
  modCount: number
}

// ============================================================
// 导出进度事件（与后端 export::EXPORT_PROGRESS_EVENT 对应）
// ============================================================

/** 导出进度事件名（前后端约定，listen 时使用） */
export const EXPORT_PROGRESS_EVENT = 'export-progress'

/** 导出进度阶段（与后端 ExportStage 枚举一致，serde rename_all="camelCase"） */
export type ExportStage = 'init' | 'scan' | 'network' | 'zip' | 'done' | 'failed'

/** 导出进度事件 payload（与后端 ExportProgress 结构体一致） */
export interface ExportProgress {
  /** 当前阶段 */
  stage: ExportStage
  /** 总进度百分比（0-100） */
  percent: number
  /** 当前操作描述（如"扫描文件 234/567"） */
  message: string
  /** 版本 ID（用于前端区分是哪个版本的导出任务） */
  versionId: string
}

/** 配置文件保存请求 */
export interface SaveConfigParams {
  configPath: string
  packName: string
  packVersion: string
  checkHostedAssets: boolean
  modrinthUploadMode: boolean
  packPath?: string | null
  options: ExportOption[]
}

/** 配置文件读取结果 */
export interface LoadConfigResult {
  packName: string
  packVersion: string
  checkHostedAssets: boolean
  modrinthUploadMode: boolean
  packPath?: string | null
  /** 从配置文件读取的勾选状态（"id=true|id=false" 列表） */
  rulesOverride: string[]
}

// ============================================================
// 高层 API（按 action 封装强类型函数）
// ============================================================

/** 获取当前版本可用的导出选项列表 */
export async function getExportOptions(versionId: string): Promise<ExportOption[]> {
  return versionExportManager<ExportOption[]>(VERSION_EXPORT_ACTIONS.GET_EXPORT_OPTIONS, {
    versionId,
  })
}

/** 执行整合包导出 */
export async function exportModpack(params: ExportModpackParams): Promise<ExportModpackResult> {
  return versionExportManager<ExportModpackResult>(VERSION_EXPORT_ACTIONS.EXPORT_MODPACK, params)
}

/** 保存导出配置到 .ini 文件 */
export async function saveExportConfig(params: SaveConfigParams): Promise<void> {
  await versionExportManager(VERSION_EXPORT_ACTIONS.SAVE_EXPORT_CONFIG, params)
}

/** 从 .ini 文件读取导出配置 */
export async function loadExportConfig(configPath: string): Promise<LoadConfigResult> {
  return versionExportManager<LoadConfigResult>(VERSION_EXPORT_ACTIONS.LOAD_EXPORT_CONFIG, {
    configPath,
  })
}
