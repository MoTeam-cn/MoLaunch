/**
 * 渐变文字工具 - 文档模型操作
 *
 * 文档是「行 → run」两层结构：纯文本读写、深拷贝与脏数据归一化都在这里。
 */
import { TEXT_FORMATS } from './types'
import type { GradientTextDocument, GradientTextRun } from './types'

export const DEFAULT_GRADIENT_COLORS = ['#A855F7', '#22C55E']

export const DEFAULT_GRADIENT_DOCUMENT: GradientTextDocument = {
  lines: [[{ text: 'MoLaunch', formats: [] }]],
}

export function cloneGradientDocument(document: GradientTextDocument): GradientTextDocument {
  return {
    lines: document.lines.map((line) => line.map((run) => ({ text: run.text, formats: [...run.formats] }))),
  }
}

/** 纯文本 → 文档：每行一个 run */
export function createDocumentFromPlainText(text: string): GradientTextDocument {
  return { lines: text.split('\n').map((line) => [{ text: line, formats: [] }]) }
}

/** 文档 → 纯文本 */
export function plainTextFromDocument(document: GradientTextDocument): string {
  return document.lines.map((line) => line.map((run) => run.text).join('')).join('\n')
}

/** 归一化：丢掉非法 run，格式按 TEXT_FORMATS 顺序去重，空文档补一行空 run */
export function normalizeGradientDocument(value: GradientTextDocument): GradientTextDocument {
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
