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
  parseGradientColors,
  plainTextFromDocument,
  type TextFormat,
} from '..'

const options = { vanillaCharacter: '&' as const, simplifyGradients: false }

describe('gradient-text', () => {
  it('should interpolate every color stop including both endpoints', () => {
    expect(interpolateGradient(['#000000', '#FFFFFF'], 3)).toEqual([
      '#000000',
      '#808080',
      '#FFFFFF',
    ])
    expect(interpolateGradient(['#FF0000', '#00FF00', '#0000FF'], 5)).toEqual([
      '#FF0000',
      '#808000',
      '#00FF00',
      '#008080',
      '#0000FF',
    ])
  })

  it('should not spend a gradient color on whitespace and support Unicode code points', () => {
    const characters = buildGradientCharacters(createDocumentFromPlainText('A 🦎'), [
      '#000000',
      '#FFFFFF',
    ])
    expect(characters.map((character) => [character.character, character.color])).toEqual([
      ['A', '#000000'],
      [' ', null],
      ['🦎', '#FFFFFF'],
    ])
  })

  it('should use one color stop as a solid text color', () => {
    const characters = buildGradientCharacters(createDocumentFromPlainText('Solid'), ['#12AB34'])
    expect(characters.map((character) => character.color)).toEqual([
      '#12AB34',
      '#12AB34',
      '#12AB34',
      '#12AB34',
      '#12AB34',
    ])
    expect(
      generateGradientOutput(createDocumentFromPlainText('A'), ['#12AB34'], 'vanilla', options),
    ).toBe('&#12AB34A')
  })

  it('should match Minecraft preview text shadow colors', () => {
    expect(getMinecraftTextShadow('#FF0000')).toBe('#800000')
    expect(getMinecraftTextShadow(null)).toBe('#000000')
  })

  it('should keep multiline text and formatting in generated output', () => {
    const document = {
      lines: [
        [{ text: 'A', formats: ['bold', 'italic'] as TextFormat[] }],
        [{ text: 'B', formats: [] as TextFormat[] }],
      ],
    }
    expect(
      generateGradientOutput(document, ['#123456', '#ABCDEF'], 'vanilla', options),
    ).toBe('&#123456&l&oA\n&#ABCDEFB')
    expect(generateGradientOutput(document, ['#123456', '#ABCDEF'], 'json', options)).toBe(
      '[{"text":"A","color":"#123456","bold":true,"italic":true},{"text":"\\n"},{"text":"B","color":"#ABCDEF"}]',
    )
  })

  it('should use the selected legacy control character and reset formatting with the next color', () => {
    const document = {
      lines: [
        [
          { text: 'A', formats: ['bold'] as TextFormat[] },
          { text: 'B', formats: [] as TextFormat[] },
        ],
      ],
    }
    expect(
      generateGradientOutput(document, ['#000000', '#FFFFFF'], 'vanilla', options),
    ).toBe('&#000000&lA&#FFFFFFB')
    expect(
      generateGradientOutput(document, ['#000000', '#FFFFFF'], 'vanilla', {
        vanillaCharacter: '§',
        simplifyGradients: false,
      }),
    ).toBe('§#000000§lA§#FFFFFFB')
  })

  it('should generate non-empty result for every registered adapter', () => {
    const document = createDocumentFromPlainText('Lab')
    expect(gradientFormatAdapters).toHaveLength(19)
    for (const adapter of gradientFormatAdapters) {
      const output = generateGradientOutput(document, ['#AA00FF', '#00FFAA'], adapter.id, options)
      expect(output).not.toBe('')
    }
  })

  it('should parse HEX, RGB, and CSS gradient color input', () => {
    expect(
      parseGradientColors('linear-gradient(90deg, #a0b, rgb(12, 34, 56), #ABCDEF)'),
    ).toEqual(['#AA00BB', '#ABCDEF', '#0C2238'])
  })

  it('should round-trip document and plain text', () => {
    const text = '第一行\n第二行'
    expect(plainTextFromDocument(createDocumentFromPlainText(text))).toBe(text)
  })
})
