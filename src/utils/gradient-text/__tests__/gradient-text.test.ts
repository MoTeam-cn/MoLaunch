/**
 * 渐变文字验收测试（对应 specs/cleanroom/gradient-text.md §7 验收用例）
 */
import { describe, expect, it } from 'vitest'
import {
  buildCells,
  documentFromText,
  generateGradientOutput,
  getMinecraftTextShadow,
  gradientFormatAdapters,
  interpolateGradient,
  normalizeHexColor,
  parseGradientColors,
  parseGradientPresets,
  textFromDocument,
  type TextFormat,
} from '..'

const options = { control: '&' as const, simplifyGradient: false }

describe('gradient-text', () => {
  it('should 归一化色值 when 输入三位或六位十六进制', () => {
    expect(normalizeHexColor('#abc')).toBe('#AABBCC')
    expect(normalizeHexColor(' 12ef9a ')).toBe('#12EF9A')
    expect(normalizeHexColor('#12345')).toBeNull()
    expect(normalizeHexColor('red')).toBeNull()
  })

  it('should 等距插值且端点必取 when 色标多于一个', () => {
    expect(interpolateGradient(['#000000', '#FFFFFF'], 3)).toEqual(['#000000', '#808080', '#FFFFFF'])
    expect(interpolateGradient(['#FF0000', '#0000FF'], 2)).toEqual(['#FF0000', '#0000FF'])
    expect(interpolateGradient(['#123456'], 3)).toEqual(['#123456', '#123456', '#123456'])
    expect(interpolateGradient([], 3)).toEqual([])
  })

  it('should 跳过空白且按码点切分 when 展开文档', () => {
    const cells = buildCells(documentFromText('Mo 🦎'), ['#111111', '#222222'])
    expect(cells.map((cell) => [cell.character, cell.color])).toEqual([
      ['M', '#111111'],
      ['o', '#1A1A1A'],
      [' ', null],
      ['🦎', '#222222'],
    ])
  })

  it('should 输出纯色 when 只有一个色标', () => {
    expect(generateGradientOutput(documentFromText('A'), ['#0F0F0F'], 'vanilla', options)).toBe('&#0F0F0FA')
  })

  it('should 使用指定控制字符 when 切换 control', () => {
    expect(
      generateGradientOutput(documentFromText('AB'), ['#000000', '#FFFFFF'], 'vanilla', {
        control: '§',
        simplifyGradient: false,
      }),
    ).toBe('§#000000A§#FFFFFFB')
  })

  it('should 计算原版阴影色 when 给定前景色', () => {
    expect(getMinecraftTextShadow('#FF0000')).toBe('#800000')
    expect(getMinecraftTextShadow('#00FF00')).toBe('#008000')
    expect(getMinecraftTextShadow('#808080')).toBe('#202020')
    expect(getMinecraftTextShadow(null)).toBe('#000000')
    expect(getMinecraftTextShadow('nope')).toBe('#000000')
  })

  it('should 保留换行与格式 when 生成 vanilla 与 json', () => {
    const document = {
      lines: [
        [{ text: 'A', formats: ['bold', 'italic'] as TextFormat[] }],
        [{ text: 'B', formats: [] as TextFormat[] }],
      ],
    }
    expect(generateGradientOutput(document, ['#111111', '#222222'], 'vanilla', options)).toBe(
      '&#111111&l&oA\n&#222222B',
    )
    expect(generateGradientOutput(document, ['#111111', '#222222'], 'json', options)).toBe(
      '[{"text":"A","color":"#111111","bold":true,"italic":true},{"text":"\\n"},{"text":"B","color":"#222222"}]',
    )
  })

  it('should 只保留首尾色标 when 开启渐变简化', () => {
    expect(
      generateGradientOutput(documentFromText('ABC'), ['#000000', '#808080', '#FFFFFF'], 'minimessage-gradient', {
        control: '&',
        simplifyGradient: true,
      }),
    ).toBe('<gradient:#000000:#FFFFFF>ABC</gradient>')
  })

  it('should 为每种格式产出非空结果 when 遍历注册表', () => {
    expect(gradientFormatAdapters).toHaveLength(19)
    for (const adapter of gradientFormatAdapters) {
      expect(generateGradientOutput(documentFromText('Lab'), ['#AA00FF', '#00FFAA'], adapter.id, options)).not.toBe('')
    }
  })

  it('should 提取色标 when 输入 CSS 渐变文本', () => {
    expect(parseGradientColors('linear-gradient(90deg, #abc, rgb(1, 2, 3), #12EF9A)')).toEqual([
      '#AABBCC',
      '#12EF9A',
      '#010203',
    ])
  })

  it('should 与纯文本互转 when 往返文档', () => {
    const text = '第一行\n第二行'
    expect(textFromDocument(documentFromText(text))).toBe(text)
  })

  it('should 丢弃非法预设 when 解析导入数据', () => {
    const presets = parseGradientPresets([{ name: '  ', colors: ['#abc'] }, { colors: [] }, 'x', null])
    expect(presets).toHaveLength(1)
    expect(presets[0].name).toBe('Preset 1')
    expect(presets[0].colors).toEqual(['#AABBCC'])
  })
})
