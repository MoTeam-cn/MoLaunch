/**
 * 渐变文字工具 - 文档操作（对应 specs/cleanroom/gradient-text.md §9）
 */
import { TEXT_FORMATS } from './types'
import type { GradientTextDocument, GradientTextRun, TextFormat } from './types'

/** 默认色标与默认文档 */
export const DEFAULT_COLORS = ['#A855F7', '#22C55E']

export const DEFAULT_DOCUMENT: GradientTextDocument = {
  lines: [[{ text: 'MoLaunch', formats: [] }]],
}

/** 逐 run 映射，保持行结构 */
function mapRuns(
  document: GradientTextDocument,
  project: (run: GradientTextRun) => GradientTextRun,
): GradientTextDocument {
  return { lines: document.lines.map((line) => line.map(project)) }
}

function emptyDocument(): GradientTextDocument {
  return { lines: [[{ text: '', formats: [] }]] }
}

/** 格式归一：只保留已知格式，顺序按 TEXT_FORMATS */
function knownFormats(formats: unknown): TextFormat[] {
  if (!Array.isArray(formats)) return []
  return TEXT_FORMATS.filter((format) => formats.includes(format))
}

function normalizeRun(run: GradientTextRun): GradientTextRun {
  return { text: run.text, formats: knownFormats(run.formats) }
}

export function cloneDocument(document: GradientTextDocument): GradientTextDocument {
  return mapRuns(document, (run) => ({ text: run.text, formats: [...run.formats] }))
}

/** 纯文本 → 文档：一行一个 run */
export function documentFromText(text: string): GradientTextDocument {
  return { lines: text.split('\n').map((line) => [{ text: line, formats: [] }]) }
}

/** 文档 → 纯文本 */
export function textFromDocument(document: GradientTextDocument): string {
  return document.lines
    .map((line) => line.reduce((merged, run) => merged + run.text, ''))
    .join('\n')
}

/** 归一化：丢弃非法 run、格式去重并按固定顺序排列、空文档补一行 */
export function normalizeDocument(value: GradientTextDocument): GradientTextDocument {
  const lines = Array.isArray(value?.lines) ? value.lines : []
  const normalized = lines.map((line) =>
    Array.isArray(line)
      ? line.filter((run): run is GradientTextRun => typeof run?.text === 'string').map(normalizeRun)
      : [],
  )
  return { lines: normalized.length > 0 ? normalized : emptyDocument().lines }
}
