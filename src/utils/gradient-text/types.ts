/**
 * 渐变文字工具 - 类型定义
 *
 * 文档模型：文本按行划分，每行为 run 列表（run 携带格式），
 * 渐变颜色在字符级展开（空白字符豁免）。
 */
export const TEXT_FORMATS = ['bold', 'italic', 'underlined', 'strikethrough', 'obfuscated'] as const

export type TextFormat = (typeof TEXT_FORMATS)[number]

export type GradientTextRun = {
  text: string
  formats: TextFormat[]
}

export type GradientTextDocument = {
  lines: GradientTextRun[][]
}

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

export type GradientOutputOptions = {
  vanillaCharacter: '&' | '§'
  simplifyGradients: boolean
}

export type GradientFormatAdapter = {
  id: GradientFormatId
  label: string
  sample: string
  mimeType: string
  extension: string
  supportsVanillaCharacter?: boolean
  supportsSimplify?: boolean
}

export type GradientCharacter = {
  character: string
  color: string | null
  formats: TextFormat[]
  newline?: boolean
}

export type GradientColor = {
  red: number
  green: number
  blue: number
}
