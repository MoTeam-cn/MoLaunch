/**
 * 渐变文字工具 - 输出格式适配器
 *
 * 19 种输出格式的元数据（标签 / 示例 / MIME / 扩展名）。
 * 各格式的生成逻辑见 core.ts 的 formatByAdapter。
 */
import type { GradientFormatAdapter } from './types'

export const gradientFormatAdapters: readonly GradientFormatAdapter[] = [
  {
    id: 'vanilla',
    label: 'Vanilla',
    sample: '&#RRGGBB',
    mimeType: 'text/plain',
    extension: 'txt',
    supportsVanillaCharacter: true,
  },
  {
    id: 'vanilla-compatible',
    label: 'Vanilla compatible',
    sample: '&x&R&R&G&G&B&B',
    mimeType: 'text/plain',
    extension: 'txt',
    supportsVanillaCharacter: true,
  },
  {
    id: 'standard',
    label: 'Standard HEX',
    sample: '#RRGGBB',
    mimeType: 'text/plain',
    extension: 'txt',
  },
  { id: 'cmi', label: 'CMI', sample: '{#RRGGBB}', mimeType: 'text/plain', extension: 'txt' },
  {
    id: 'minimessage',
    label: 'MiniMessage',
    sample: '<#RRGGBB>',
    mimeType: 'text/plain',
    extension: 'txt',
  },
  {
    id: 'minimessage-gradient',
    label: 'MiniMessage gradient',
    sample: '<gradient:#RRGGBB:#RRGGBB>',
    mimeType: 'text/plain',
    extension: 'txt',
    supportsSimplify: true,
  },
  {
    id: 'minedown',
    label: 'MineDown',
    sample: '&#RRGGBB&',
    mimeType: 'text/plain',
    extension: 'txt',
  },
  {
    id: 'snbt',
    label: 'Stringified NBT',
    sample: "{text:'T',color:'#RRGGBB'}",
    mimeType: 'application/json',
    extension: 'snbt',
  },
  { id: 'trchat', label: 'TrChat', sample: '&{#RRGGBB}', mimeType: 'text/plain', extension: 'txt' },
  {
    id: 'taboolib',
    label: 'TabooLib',
    sample: '&{#RRGGBB}',
    mimeType: 'text/plain',
    extension: 'txt',
  },
  {
    id: 'taboolib-gradient',
    label: 'TabooLib gradient',
    sample: '[Text](gradient=#RRGGBB,#RRGGBB)',
    mimeType: 'text/plain',
    extension: 'txt',
    supportsSimplify: true,
  },
  {
    id: 'rosegarden-gradient',
    label: 'RoseGarden gradient',
    sample: '<g:#RRGGBB:#RRGGBB>Text',
    mimeType: 'text/plain',
    extension: 'txt',
    supportsSimplify: true,
  },
  {
    id: 'chat-colors',
    label: 'Chat Colors',
    sample: '[#RRGGBB]',
    mimeType: 'text/plain',
    extension: 'txt',
  },
  { id: 'motd', label: 'MOTD', sample: '\\u00A7x', mimeType: 'text/plain', extension: 'txt' },
  {
    id: 'bbcode',
    label: 'BBCode',
    sample: '[color=#RRGGBB]Text[/color]',
    mimeType: 'text/plain',
    extension: 'txt',
  },
  {
    id: 'json',
    label: 'JSON text component',
    sample: '{"text":"T","color":"#RRGGBB"}',
    mimeType: 'application/json',
    extension: 'json',
  },
  {
    id: 'html',
    label: 'HTML',
    sample: '<span style="color: #RRGGBB">Text</span>',
    mimeType: 'text/html',
    extension: 'html',
  },
  { id: 'csv', label: 'CSV', sample: '#RRGGBB,T', mimeType: 'text/csv', extension: 'csv' },
  {
    id: 'terraria',
    label: 'Terraria',
    sample: '[c/RRGGBB:T]',
    mimeType: 'text/plain',
    extension: 'txt',
  },
]
