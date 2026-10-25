/**
 * 渐变文字工具 - 字符级展开
 *
 * 先把文档摊平成字符单元，再按「可见字符数」插值出色带并逐字上色；
 * 空白字符不消耗色标，换行以独立单元表示。
 */
import { interpolateGradient } from './color'
import { normalizeGradientDocument } from './document'
import type { GradientCharacter, GradientTextDocument } from './types'

const WHITESPACE = /\s/u

/** 文档 + 色标 → 逐字符单元（含换行单元） */
export function buildGradientCharacters(
  document: GradientTextDocument,
  colors: string[],
): GradientCharacter[] {
  const lines = normalizeGradientDocument(document).lines
  const cells: GradientCharacter[] = []

  lines.forEach((line, lineIndex) => {
    for (const run of line) {
      for (const glyph of Array.from(run.text)) {
        cells.push({ character: glyph, color: null, formats: run.formats })
      }
    }
    if (lineIndex < lines.length - 1) {
      cells.push({ character: '\n', color: null, formats: [], newline: true })
    }
  })

  const visibleCount = cells.filter((cell) => !cell.newline && !WHITESPACE.test(cell.character)).length
  const gradient = interpolateGradient(colors, visibleCount)
  let cursor = 0
  for (const cell of cells) {
    if (cell.newline || WHITESPACE.test(cell.character)) continue
    cell.color = gradient[cursor] ?? null
    cursor += 1
  }
  return cells
}
