/**
 * 深色主题覆盖守护测试
 *
 * 目的：把「深色下出现浅色块 / 浅字看不清」这类问题挡在提交前。
 * 三条不变量：
 * 1. Tailwind v4 的中性色变量必须在 [data-theme='dark'] 里整体重映射（结构性保证所有
 *    bg / text / border / divide / ring / from / to / via 与 hover 变体都跟随深色）；
 * 2. 源码里用到的「带透明度」中性色（编译成字面色值，变量覆盖不到）必须有对应深色规则，
 *    除非它属于彩色条上的高光玻璃（白名单）；
 * 3. 样式文件里不得再出现白色背景硬编码（必须走语义 token 或工具类）。
 */
import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

/** 本文件位于 src/assets/styles/__tests__，上两级即 src */
const SRC = path.resolve(__dirname, '../..')
const STYLES = path.resolve(__dirname, '..')
const DARK_SCSS = fs.readFileSync(path.join(STYLES, 'theme-dark.scss'), 'utf8')

/** 彩色条上的高光玻璃：两种主题都保持白色，不属于「面板面」 */
const GLASS_HIGHLIGHTS = new Set(['bg-white/5', 'bg-white/10', 'bg-white/15', 'bg-white/20', 'bg-white/25', 'bg-white/30', 'bg-white/40', 'bg-white/70', 'bg-white/75', 'bg-white/85'])

/** 递归收集 src 下的源码（跳过测试与样式） */
function collectSources(dir: string, out: string[] = []): string[] {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      if (entry.name === '__tests__' || entry.name === 'styles') continue
      collectSources(full, out)
    } else if (/\.(vue|ts)$/.test(entry.name)) {
      out.push(full)
    }
  }
  return out
}

/** 源码里实际用到的、带透明度或纯色的中性工具类 */
function collectNeutralClasses(): Map<string, string[]> {
  const re = /\b((?:bg|from|via|to)-(?:white|gray|slate|zinc|neutral|stone)(?:-\d{2,3})?\/\d{1,3})\b/g
  const found = new Map<string, string[]>()
  for (const file of collectSources(SRC)) {
    const text = fs.readFileSync(file, 'utf8')
    for (const match of text.matchAll(re)) {
      const cls = match[1]
      const list = found.get(cls) ?? []
      list.push(path.relative(SRC, file).replace(/\\/g, '/'))
      found.set(cls, list)
    }
  }
  return found
}

/**
 * 解析 theme-dark.scss 里已出现的工具类选择器名（把 `bg-white\/80` 还原成 `bg-white/80`）
 *
 * 不用「把类名拼成选择器再去 includes」的写法：那需要自己转义特殊字符，
 * 漏转义就会误判（且属于不完整的字符串清洗）。直接解析已有选择器更可靠。
 */
function darkSelectors(): Set<string> {
  const found = new Set<string>()
  for (const match of DARK_SCSS.matchAll(/\.([a-zA-Z][a-zA-Z0-9_-]*(?:\\.[a-zA-Z0-9_-]+)*)/g)) {
    found.add(match[1].replace(/\\(.)/g, '$1'))
  }
  return found
}

describe('深色主题覆盖', () => {
  it('should 重映射全部中性色变量 when 深色主题', () => {
    const shades = ['50', '100', '200', '300', '400', '500', '600', '700', '800', '900']
    for (const shade of shades) {
      expect(DARK_SCSS, `--color-gray-${shade} 必须在深色主题重映射`).toContain(`--color-gray-${shade}:`)
    }
    // white 不能重映射：bg-primary-500 上的 text-white 会变深字
    expect(DARK_SCSS).not.toMatch(/--color-white:/)
  })

  it('should 提供深色规则 when 源码用到带透明度的中性色', () => {
    const covered = darkSelectors()
    const missing: string[] = []
    for (const [cls, files] of collectNeutralClasses()) {
      if (GLASS_HIGHLIGHTS.has(cls)) continue
      if (!covered.has(cls)) missing.push(`${cls}（${files[0]}）`)
    }
    expect(missing, '这些类编译成字面色值，必须在 theme-dark.scss 里补深色规则').toEqual([])
  })

  it('should 禁止浅色背景硬编码 when 组件内写 scoped 样式', () => {
    const offenders: string[] = []
    const lightBg = /(?:background(?:-color)?)\s*:\s*(?:#fff\b|#ffffff\b|white\b|#f2f3f5\b|#e5e6eb\b|#f7f8fa\b|#f5f6f8\b)/i
    for (const file of collectSources(SRC).filter((f) => f.endsWith('.vue'))) {
      const text = fs.readFileSync(file, 'utf8')
      for (const block of text.match(/<style[\s\S]*?<\/style>/g) ?? []) {
        block.split(/\r?\n/).forEach((line, index) => {
          if (lightBg.test(line)) offenders.push(`${path.relative(SRC, file).replace(/\\/g, '/')}:${index + 1} ${line.trim().slice(0, 80)}`)
        })
      }
    }
    expect(offenders, '组件内背景请改用 var(--surface-*) 语义 token').toEqual([])
  })

  it('should 禁止白色背景硬编码 when 书写样式文件', () => {
    const offenders: string[] = []
    for (const file of fs.readdirSync(STYLES).filter((f) => f.endsWith('.scss') && f !== 'tokens.scss')) {
      const text = fs.readFileSync(path.join(STYLES, file), 'utf8')
      text.split(/\r?\n/).forEach((line, index) => {
        if (/(?:background(?:-color)?)\s*:\s*(?:#fff\b|#ffffff\b|white\b|#f2f3f5\b|#e5e6eb\b|#f7f8fa\b|#f5f6f8\b|#c9cdd4\b)/i.test(line)) {
          offenders.push(`${file}:${index + 1} ${line.trim()}`)
        }
      })
    }
    expect(offenders, '白色背景请改用 var(--surface-0) 等语义 token').toEqual([])
  })
})
