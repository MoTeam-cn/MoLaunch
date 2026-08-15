/**
 * 工具模块 - 资源包可视化编辑器
 *
 * 对应后端 `tools_manager` 的 rp_open / rp_read action。
 * M1 查看器闭环：打开包（zip/folder）→ 结构树 → 按文件类型读取预览。
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
