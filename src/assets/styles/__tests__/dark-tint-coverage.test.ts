/**
 * 彩色淡底与深字的深色覆盖回归
 *
 * 中性色（gray）由 Tailwind v4 变量在 [data-theme='dark'] 里整体重映射，
 * 不用逐条覆盖；彩色系不在重映射范围内，`bg-orange-50` 这类浅底必须逐条给。
 * 漏一条的表现就是暗色下悬停变白底、深字看不见，而且只在交互瞬间出现。
 *
 * 判据按「会不会出问题」分档：
 * - 底色与描边用 -200 及更浅 → 浅底叠深色，必须覆盖
 * - 文字用 -600 及更深 → 深字压在深底，必须提亮
 */

import { describe, expect, it } from 'vitest'
import fs from 'node:fs'
import path from 'node:path'

const SRC = path.resolve(__dirname, '../../..')
const DARK = fs.readFileSync(path.join(SRC, 'assets/styles/theme-dark.scss'), 'utf8')

const COLORS =
  'red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose'
const CLASS_RE = new RegExp(
  String.raw`\b((?:bg|text|border)-(?:${COLORS})-\d+(?:\/\d+)?)\b`,
  'g'
)

function walk(dir: string, depth = 0): string[] {
  if (depth > 6) return []
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    if (['node_modules', '.git', 'dist', '__tests__'].includes(entry.name)) return []
    const full = path.join(dir, entry.name)
    if (entry.isDirectory()) return walk(full, depth + 1)
    return /\.(vue|ts)$/.test(entry.name) && !/\.d\.ts$/.test(entry.name) ? [full] : []
  })
}

function coveredClasses(): Set<string> {
  const covered = new Set<string>()
  for (const match of DARK.matchAll(/\.((?:bg|text|border)-[a-z]+-\d+(?:\\?\/\d+)?)/g)) {
    covered.add(match[1].replace(/\\\//g, '/'))
  }
  return covered
}

describe('彩色淡底与深字的深色覆盖', () => {
  it('should cover every light tint and dark text when 源码用到', () => {
    const covered = coveredClasses()
    const used = new Map<string, string>()

    for (const file of walk(path.join(SRC))) {
      const text = fs.readFileSync(file, 'utf8')
      for (const match of text.matchAll(CLASS_RE)) {
        used.set(match[1], path.relative(SRC, file).replace(/\\/g, '/'))
      }
    }

    // 扫描结果非空才证明这条测试在干活
    expect(used.size, '没有扫到任何彩色类，测试本身失效了').toBeGreaterThan(50)

    const needsCover = (cls: string): boolean => {
      const shade = Number(cls.match(/-(\d+)(?:\/|$)/)?.[1] ?? 0)
      if (cls.startsWith('text-')) return shade >= 600
      return shade <= 300
    }

    const missing = [...used.entries()]
      .filter(([cls]) => needsCover(cls) && !covered.has(cls))
      .map(([cls, file]) => `${cls}（${file}）`)

    expect(missing, '这些彩色档位在暗色主题下会变成浅底或深字，必须在 theme-dark.scss 补规则').toEqual([])
  })

  it('should remap tint shades as variables when 变体类名匹配不到逐类规则', () => {
    // hover:/focus: 变体编译成独立类名，逐类规则匹配不到；
    // 只有把浅色档位本身重映射成深色值，悬停时才不会退回白底。
    const colors = [
      'red', 'orange', 'amber', 'yellow', 'lime', 'green', 'emerald', 'teal',
      'cyan', 'sky', 'blue', 'indigo', 'purple', 'pink',
    ]
    const missing = colors.flatMap((color) =>
      ['50', '100', '200']
        .filter((shade) => !DARK.includes(`--color-${color}-${shade}:`))
        .map((shade) => `${color}-${shade}`)
    )
    expect(missing, '这些彩色档位需要在 [data-theme=dark] 里重映射变量，否则 hover 变体会露出浅色').toEqual([])

    const primaryMissing = ['50', '100', '200'].filter(
      (shade) => !DARK.includes(`--color-primary-${shade}:`)
    )
    expect(primaryMissing, '品牌色的浅色档位同样要靠变量重映射').toEqual([])
  })

  it('should not rely on translucent tint when 组件动态拼接颜色', () => {
    const card = fs.readFileSync(path.join(SRC, 'components/common/LoaderCard.vue'), 'utf8')
    const offenders = card
      .split('\n')
      .map((line, index) => ({ line, index }))
      .filter(({ line }) => /-\$\{color\}-(50|100|200)\//.test(line))
      .map(({ index }) => index + 1)
    expect(offenders, '动态拼接的淡色档带透明度时极易漏覆盖，改用不透明档').toEqual([])
  })
})
