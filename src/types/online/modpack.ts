/**
 * 联机功能类型定义 - 整合包元数据域
 *
 * 联机大厅阶段 3/4 新增。房主创建房间时关联本地已安装整合包，
 * 加入方拉取房间详情后据此判断是否需要一键安装。
 */

/**
 * 整合包元数据（联机大厅阶段 3 新增）
 *
 * 房主创建房间时关联本地已安装整合包，上报给 api-server。
 * 加入方拉取房间详情后据此判断是否需要一键安装。
 *
 * **安全设计**：不包含 `downloadUrl` 字段。加入方通过现有 `getProjectVersions`
 * IPC 反查平台 API 获取下载链接，避免 api-server 成为 URL 分发中心。
 *
 * 对应后端 `minecraft::online::signaling::ModpackMeta`。
 */
export interface ModpackMeta {
  /** 来源平台（`curseforge` / `modrinth`） */
  source: string
  /** CF project id 或 MR project id */
  projectId: string
  /** CF file id 或 MR version id */
  fileId: string
  /** 整合包对应的 MC 版本（如 `1.12.2`） */
  mcVersion: string
  /** 整合包自身版本号（如 `2.9.3`） */
  modpackVersion?: string
  /** 整合包名称 */
  name: string
  /** 加载器类型（`forge` / `fabric` / `neoforge` / `quilt`） */
  loader?: string
  /** 加载器版本号 */
  loaderVersion?: string
  /** 整合包文件大小（字节，仅展示用） */
  fileSize?: number
  /** mods 文件数（仅展示用） */
  fileCount?: number
  /** manifest.json SHA-256，用于加入方校验本地是否已装同款 */
  manifestHash?: string
}

/**
 * 本地整合包元数据文件（`versions/{id}/modpack.meta.json`）
 *
 * 整合包安装完成时由后端写入，创建联机房间时读取并转换为 `ModpackMeta` 上报。
 * 与 `ModpackMeta` 字段一致，额外含 `installedAt` 本地记录（不上报）。
 *
 * 对应后端 `minecraft::version::modpack_meta::ModpackMetaFile`。
 */
export interface ModpackMetaFile {
  source: string
  projectId: string
  fileId: string
  mcVersion: string
  modpackVersion?: string
  name: string
  loader?: string
  loaderVersion?: string
  fileSize?: number
  fileCount?: number
  manifestHash?: string
  /** 安装时间（Unix 秒，仅本地记录，不上报） */
  installedAt: number
}

/**
 * 校验本地是否已安装指定整合包的结果（联机大厅阶段 4 新增）
 *
 * 对应后端 `commands::version::list::CheckLocalModpackResult`。
 * 加入方加入房间后据此判断是否需要一键安装房主要求的整合包。
 */
export interface CheckLocalModpackResult {
  /** 是否已安装 */
  installed: boolean
  /** 匹配的 version_id（`installed=false` 时为 undefined） */
  versionId?: string
}
