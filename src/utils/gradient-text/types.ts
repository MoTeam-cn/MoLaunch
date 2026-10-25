/**
 * 渐变文字工具 - 类型定义
 *
 * 文档模型：文本按行切分，每行由若干带格式的 run 组成；渐变颜色按字符展开，
 * 空白字符不消耗色标。
 */

/** 可叠加的文本格式（顺序即输出顺序） */
export const TEXT_FORMATS = ['bold', 'italic', 'underlined', 'strikethrough', 'obfuscated'] as const

export type TextFormat = (typeof TEXT_FORMATS)[number]

/** 文档中的一段同格式文本 */
export type GradientTextRun = {
  text: string
  formats: TextFormat[]
}

/** 整篇文档 */
export type GradientTextDocument = {
  lines: GradientTextRun[][]
}

/** 支持的输出格式标识 */
export type GradientFormatId =
  | 'vanilla'
  | 'vanilla-compatible'
  | 'standard'
  | 'cmi'
  | 'minimessage'
  | 'minimessage-gradient'
  | 'minedown'
  | 'snbt'
  | 'trchat'
  | 'taboolib'
  | 'taboolib-gradient'
  | 'rosegarden-gradient'
  | 'chat-colors'
  | 'motd'
  | 'bbcode'
  | 'json'
  | 'html'
  | 'csv'
  | 'terraria'

/** 输出选项 */
export type GradientOutputOptions = {
  /** Vanilla 系格式使用的控制字符 */
  vanillaCharacter: '&' | '§'
  /** 渐变类格式只保留首尾两个色标 */
  simplifyGradients: boolean
}

/** 输出格式元数据 */
export type GradientFormatAdapter = {
  id: GradientFormatId
  label: string
  sample: string
  mimeType: string
  extension: string
  supportsVanillaCharacter?: boolean
  supportsSimplify?: boolean
}

/** 展开后的单字符单元 */
export type GradientCharacter = {
  character: string
  color: string | null
  formats: TextFormat[]
  newline?: boolean
}

/** RGB 三元组（0-255） */
export type GradientColor = {
  red: number
  green: number
  blue: number
}
