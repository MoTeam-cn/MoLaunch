/**
 * 工具模块 - 资源包可视化编辑器
 *
 * 对应后端 `tools_manager` 的 rp_open / rp_read / rp_write / rp_export action。
 * M1 查看器闭环：打开包（zip/folder）→ 结构树 → 按文件类型读取预览。
 * M2 编辑闭环：写回单文件（rp_write）+ 打包导出 zip（rp_export）。
 */

import { TOOLS_ACTIONS, toolsManager } from './core'

/** 资源包结构树节点 */
export interface RpTreeNode {
  name: string
  /** 相对包根的路径（正斜杠） */
  rel_path: string
  /** dir / file */
  kind: string
  /** 文件类型：mcmeta / png / model / lang / json / ogg / text / other */
  file_type: string
  /** 大小（字节，目录为子树合计） */
  size: number
  /** 是否为动画纹理（存在同名 .png.mcmeta） */
  animated: boolean
  children: RpTreeNode[]
}

/** 打开资源包结果 */
export interface RpOpenResult {
  /** 工作目录（zip 会话为临时目录，folder 会话为原目录） */
  work_dir: string
  is_zip: boolean
  name: string
  /** zip / folder */
  format: string
  size: number
  /** pack.png base64 data URI */
  icon_data_url: string | null
  /** 原包路径（zip 会话为源 zip 路径；保存回原包用） */
  src_path: string | null
  pack_format: number | null
  /** pack_format 对应的 MC 版本范围描述 */
  mc_version: string | null
  description: string | null
  tree: RpTreeNode
  /** 失败原因（成功时为空） */
  error: string
}

/** 读取单文件结果 */
export interface RpReadResult {
  /** text（原文）/ data_uri（base64 data URI） */
  kind: string
  content: string
  error: string
}

/** 批量读取模型与关联纹理结果（3D 预览） */
export interface RpReadManyResult {
  /** 起始文件相对路径 */
  root: string
  /** rel_path -> 文件内容（model/blockstate 为 text，关联纹理为 data_uri） */
  files: Record<string, RpReadResult>
  /** 失败原因（成功时为空） */
  error: string
}

/** pack_format 版本查询结果 */
export interface RpPackFormatInfoResult {
  /** 对应 MC 版本范围描述（如 "1.20.5–1.21.x"），未知为 "未知版本" */
  mc_version: string
  /** 是否为已知的 pack_format（用于前端校验提示） */
  known: boolean
  /** 失败原因（成功时为空） */
  error: string
}

/** MC 版本推导 pack_format 结果（复用皮肤资源包版本映射） */
export interface RpVersionPackFormatResult {
  /** 推导出的 pack_format 数值 */
  pack_format: number
  /** 是否为可解析的版本号 */
  known: boolean
  /** 失败原因（成功时为空） */
  error: string
}

/** 写回单文件参数 */
export interface RpWriteParams {
  /** 工作目录（rp_open 返回） */
  work_dir: string
  /** 包内相对路径（正斜杠） */
  rel_path: string
  /** text（UTF-8 文本）/ base64（图片等二进制，允许带 data URI 前缀） */
  kind: string
  content: string
}

/** 写回单文件结果 */
export interface RpWriteResult {
  success: boolean
  message: string
}

/** 导出资源包参数 */
export interface RpExportParams {
  /** 工作目录（rp_open 返回） */
  work_dir: string
  /** 导出目标路径：保存回原包传原 zip 路径，另存为传用户选择路径 */
  path: string
  /** 导出格式：zip */
  format: string
  /** 原 zip 路径（zip 会话的源包，复制其注释等附加属性） */
  src_path?: string | null
}

/** 导出资源包结果 */
export interface RpExportResult {
  success: boolean
  output_path: string
  message: string
}

/** 打开资源包（zip/folder），可传上一会话的 work_dir 供后端清理临时目录 */
export function rpOpen(path: string, previousWorkDir?: string): Promise<RpOpenResult> {
  return toolsManager<RpOpenResult>(TOOLS_ACTIONS.RP_OPEN, {
    path,
    previous_work_dir: previousWorkDir ?? null,
  })
}

/** 读取包内单文件（png/ogg → data URI，json/lang/文本 → 原文） */
export function rpRead(workDir: string, relPath: string): Promise<RpReadResult> {
  return toolsManager<RpReadResult>(TOOLS_ACTIONS.RP_READ, {
    work_dir: workDir,
    rel_path: relPath,
  })
}

/** 批量读取模型与关联纹理（blockstate/模型 + parent 链 + 被引用纹理 png，供 3D 预览） */
export function rpReadMany(workDir: string, relPath: string): Promise<RpReadManyResult> {
  return toolsManager<RpReadManyResult>(TOOLS_ACTIONS.RP_READ_MANY, {
    work_dir: workDir,
    rel_path: relPath,
  })
}

/** 查询 pack_format 对应的 MC 版本（表单输入时实时联动校验） */
export function rpPackFormatInfo(packFormat: number): Promise<RpPackFormatInfoResult> {
  return toolsManager<RpPackFormatInfoResult>(TOOLS_ACTIONS.RP_PACK_FORMAT_INFO, {
    pack_format: packFormat,
  })
}

/** 由 MC 版本推导 pack_format（下拉选版本后自动回填） */
export function rpVersionPackFormat(mcVersion: string): Promise<RpVersionPackFormatResult> {
  return toolsManager<RpVersionPackFormatResult>(TOOLS_ACTIONS.RP_VERSION_PACK_FORMAT, {
    mc_version: mcVersion,
  })
}

/** 写回包内单文件（text 原文 / base64 二进制） */
export function rpWrite(params: RpWriteParams): Promise<RpWriteResult> {
  return toolsManager<RpWriteResult>(TOOLS_ACTIONS.RP_WRITE, params)
}

/** 导出资源包为 zip（保存回原包或另存为新路径） */
export function rpExport(params: RpExportParams): Promise<RpExportResult> {
  return toolsManager<RpExportResult>(TOOLS_ACTIONS.RP_EXPORT, params)
}
