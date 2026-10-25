/**
 * 渐变文字工具 - 数据模型（对应 specs/cleanroom/gradient-text.md §1）
 */

/** 可叠加的文本格式，输出顺序即此顺序 */
export const TEXT_FORMATS = ['bold', 'italic', 'underlined', 'strikethrough', 'obfuscated'] as const

export type TextFormat = (typeof TEXT_FORMATS)[number]

/** 输出格式标识（§5 表格的顺序） */
export const GRADIENT_FORMATS = [
  'vanilla',
  'vanilla-compatible',
  'standard',
  'cmi',
  'minimessage',
  'minimessage-gradient',
  'minedown',
  'snbt',
  'trchat',
  'taboolib',
  'taboolib-gradient',
  'rosegarden-gradient',
  'chat-colors',
  'motd',
  'bbcode',
  'json',
  'html',
  'csv',
  'terraria',
] as const

export type GradientFormatId = (typeof GRADIENT_FORMATS)[number]

/** 一段同格式文本 */
export type GradientTextRun = {
  text: string
  formats: TextFormat[]
}

/** 整篇文档：行 → run */
export type GradientTextDocument = {
  lines: GradientTextRun[][]
}

/** 展开后的字符单元（§2） */
export type GradientCell = {
  character: string
  color: string | null
  formats: TextFormat[]
  newline: boolean
}

/** 输出选项（§1） */
export type GradientOutputOptions = {
  /** 原版系格式使用的控制字符 */
  control: '&' | '§'
  /** 渐变类格式只保留首尾两个色标 */
  simplifyGradient: boolean
}

/** 输出格式元数据（供界面下拉与示例展示） */
export type GradientFormatInfo = {
  id: GradientFormatId
  label: string
  sample: string
  mimeType: string
  extension: string
  usesControl?: boolean
  supportsSimplify?: boolean
}
