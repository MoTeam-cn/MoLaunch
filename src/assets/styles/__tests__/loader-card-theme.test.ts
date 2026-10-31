/**
 * 加载器卡片在暗色主题下的取色回归测试
 *
 * 选中与悬停都用淡色底。这个档位在 theme-dark.scss 里有深色取值；
 * 一旦改回带透明度的写法又漏了覆盖，浅色底会叠在深色主题上，
 * 文字几乎看不见——这种问题只在悬停的一瞬间出现，很难靠人测发现。
 */

import { describe, expect, it } from 'vitest'
import fs from 'node:fs'
import path from 'node:path'

const SRC = path.resolve(__dirname, '../../..')
const CARD = fs.readFileSync(path.join(SRC, 'components/common/LoaderCard.vue'), 'utf8')
const DARK = fs.readFileSync(path.join(SRC, 'assets/styles/theme-dark.scss'), 'utf8')

const COLORS = ['orange', 'purple', 'blue', 'green', 'teal']

describe('加载器卡片暗色取色', () => {
  it('should not use translucent tint classes when 组件用动态颜色拼接', () => {
    const offenders = CARD.split('\n')
      .map((line, index) => ({ line, index }))
      .filter(({ line }) => /-\$\{color\}-(50|100)\//.test(line))
      .map(({ index }) => index + 1)
    expect(offenders, '这些行动态拼接了带透明度的淡色类，暗色下会漏覆盖').toEqual([])
  })

  it('should have dark override when 卡片用到的每个颜色档位', () => {
    const missing = COLORS.filter(
      (color) => !DARK.includes(`[data-theme='dark'] .bg-${color}-50,`)
    )
    expect(missing, '这些颜色的淡底在 theme-dark.scss 里没有深色取值').toEqual([])
  })

  it('should cover translucent orange when 动态 hover 退回透明档', () => {
    expect(DARK).toContain('.bg-orange-50\\/50')
  })
})
