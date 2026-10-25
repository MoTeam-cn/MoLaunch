/**
 * 渐变文字工具 - 文档操作（对应 specs/cleanroom/gradient-text.md §9）
 */
import { TEXT_FORMATS } from './types'
import type { GradientTextDocument, GradientTextRun } from './types'

/** 默认色标与默认文档 */
export const DEFAULT_COLORS = ['#A855F7', '#22C55E']

export const DEFAULT_DOCUMENT: GradientTextDocument = {
  lines: [[{ text: 'MoLaunch', formats: [] }]],
}

export function cloneDocument(document: GradientTextDocument): GradientTextDocument {
  return {
    lines: document.lines.map((line) => line.map((run) => ({ text: run.text, formats: [...run.formats] }))),
  }
}

/** 纯文本 → 文档：一行一个 run */
export function documentFromText(text: string): GradientTextDocument {
  return { lines: text.split('\n').map((line) => [{ text: line, formats: [] }]) }
}

/** 文档 → 纯文本 */
export function textFromDocument(document: GradientTextDocument): string {
  return document.lines.map((line) => line.map((run) => run.text).join('')).join('\n')
}

/** 归一化：丢弃非法 run、格式去重并按固定顺序排列、空文档补一行 */
export function normalizeDocument(value: GradientTextDocument): GradientTextDocument {
  const lines = Array.isArray(value?.lines) ? value.lines : []
  const normalized = lines.map((line) => {
    if (!Array.isArray(line)) return []
    return line
      .filter((run): run is GradientTextRun => typeof run?.text === 'string')
      .map((run) => ({
        text: run.text,
        formats: TEXT_FORMATS.filter((format) => Array.isArray(run.formats) && run.formats.includes(format)),
      }))
  })
  return { lines: normalized.length > 0 ? normalized : [[{ text: '', formats: [] }]] }
}
