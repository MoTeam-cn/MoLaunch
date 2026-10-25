/**
 * 渐变文字核心逻辑单测
 */
import { describe, expect, it } from 'vitest'
import {
  buildGradientCharacters,
  createDocumentFromPlainText,
  generateGradientOutput,
  getMinecraftTextShadow,
  gradientFormatAdapters,
  interpolateGradient,
  normalizeHexColor,
  parseGradientColors,
  parseGradientPresets,
  plainTextFromDocument,
  type TextFormat,
} from '..'

const legacy = { vanillaCharacter: '&' as const, simplifyGradients: false }

describe('gradient-text', () => {
  it('should 归一化十六进制颜色 when 输入三位或六位色值', () => {
    expect(normalizeHexColor('#abc')).toBe('#AABBCC')
    expect(normalizeHexColor(' 12ef9A ')).toBe('#12EF9A')
    expect(normalizeHexColor('not-a-color')).toBeNull()
    expect(normalizeHexColor('#12345')).toBeNull()
  })

  it('should 在色标之间线性插值 when 色标多于一个', () => {
    expect(interpolateGradient(['#000000', '#FFFFFF'], 3)).toEqual(['#000000', '#808080', '#FFFFFF'])
    expect(interpolateGradient(['#FF0000', '#0000FF'], 2)).toEqual(['#FF0000', '#0000FF'])
    expect(interpolateGradient(['#123456'], 4)).toEqual(['#123456', '#123456', '#123456', '#123456'])
    expect(interpolateGradient([], 3)).toEqual([])
  })

  it('should 跳过空白字符且按码点切分 when 展开文档', () => {
    const cells = buildGradientCharacters(createDocumentFromPlainText('Mo 🦎'), ['#111111', '#222222'])
    expect(cells.map((cell) => [cell.character, cell.color])).toEqual([
      ['M', '#111111'],
      ['o', '#1A1A1A'],
      [' ', null],
      ['🦎', '#222222'],
    ])
  })

  it('should 输出纯色 when 只有一个色标', () => {
    expect(generateGradientOutput(createDocumentFromPlainText('A'), ['#0F0F0F'], 'vanilla', legacy)).toBe(
      '&#0F0F0FA',
    )
  })

  it('should 计算原版阴影色 when 给定前景色', () => {
    expect(getMinecraftTextShadow('#FF0000')).toBe('#800000')
    expect(getMinecraftTextShadow('#00FF00')).toBe('#008000')
    expect(getMinecraftTextShadow(null)).toBe('#000000')
    expect(getMinecraftTextShadow('#808080')).toBe('#202020')
  })

  it('should 保留多行与格式 when 生成 vanilla 与 json', () => {
    const document = {
      lines: [
        [{ text: 'A', formats: ['bold', 'italic'] as TextFormat[] }],
        [{ text: 'B', formats: [] as TextFormat[] }],
      ],
    }
    expect(generateGradientOutput(document, ['#111111', '#222222'], 'vanilla', legacy)).toBe(
      '&#111111&l&oA\n&#222222B',
    )
    expect(generateGradientOutput(document, ['#111111', '#222222'], 'json', legacy)).toBe(
      '[{"text":"A","color":"#111111","bold":true,"italic":true},{"text":"\\n"},{"text":"B","color":"#222222"}]',
    )
  })

  it('should 使用选定的控制字符 when 切换 vanilla 字符', () => {
    const document = { lines: [[{ text: 'AB', formats: [] as TextFormat[] }]] }
    expect(
      generateGradientOutput(document, ['#000000', '#FFFFFF'], 'vanilla', {
        vanillaCharacter: '§',
        simplifyGradients: false,
      }),
    ).toBe('§#000000A§#FFFFFFB')
  })

  it('should 折叠成首尾两色 when 开启渐变简化', () => {
    const document = createDocumentFromPlainText('ABC')
    expect(
      generateGradientOutput(document, ['#000000', '#808080', '#FFFFFF'], 'minimessage-gradient', {
        vanillaCharacter: '&',
        simplifyGradients: true,
      }),
    ).toBe('<gradient:#000000:#FFFFFF>ABC</gradient>')
  })

  it('should 为每种注册格式产出非空结果', () => {
    expect(gradientFormatAdapters).toHaveLength(19)
    const document = createDocumentFromPlainText('Lab')
    for (const adapter of gradientFormatAdapters) {
      expect(generateGradientOutput(document, ['#AA00FF', '#00FFAA'], adapter.id, legacy)).not.toBe('')
    }
  })

  it('should 从任意文本中提取色标 when 输入 CSS 渐变', () => {
    expect(parseGradientColors('linear-gradient(90deg, #abc, rgb(1, 2, 3), #12EF9A)')).toEqual([
      '#AABBCC',
      '#12EF9A',
      '#010203',
    ])
  })

  it('should 与纯文本互转 when 往返文档', () => {
    const text = '第一行\n第二行'
    expect(plainTextFromDocument(createDocumentFromPlainText(text))).toBe(text)
  })

  it('should 丢弃非法预设 when 解析导入数据', () => {
    const presets = parseGradientPresets([{ name: '  ', colors: ['#abc'] }, { colors: [] }, 'nope', null])
    expect(presets).toHaveLength(1)
    expect(presets[0].name).toBe('Preset 1')
    expect(presets[0].colors).toEqual(['#AABBCC'])
  })
})
