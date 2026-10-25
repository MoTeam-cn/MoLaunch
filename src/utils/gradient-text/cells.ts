/**
 * 渐变文字工具 - 字符展开与上色（对应 specs/cleanroom/gradient-text.md §2）
 */
import { interpolateGradient } from './colors'
import { normalizeDocument } from './document'
import type { GradientCell, GradientTextDocument } from './types'

/**
 * 文档 + 色标 → 字符单元序列
 *
 * 1. 按码点切分每个 run，行与行之间插入换行单元
 * 2. 统计可见字符数（非空白）
 * 3. 插值出等量颜色，按出现顺序赋给可见字符
 */
export function buildCells(document: GradientTextDocument, colors: string[]): GradientCell[] {
  const lines = normalizeDocument(document).lines
  const cells: GradientCell[] = []

  lines.forEach((line, lineIndex) => {
    for (const run of line) {
      for (const glyph of Array.from(run.text)) {
        cells.push({ character: glyph, color: null, formats: run.formats, newline: false })
      }
    }
    if (lineIndex < lines.length - 1) {
      cells.push({ character: '\n', color: null, formats: [], newline: true })
    }
  })

  const visible = cells.filter((cell) => !cell.newline && !/\s/u.test(cell.character))
  const palette = interpolateGradient(colors, visible.length)
  visible.forEach((cell, index) => {
    cell.color = palette[index] ?? null
  })
  return cells
}
