/**
 * 渐变文字工具 - 本地状态与预设
 *
 * 状态（文档 / 色标 / 输出配置 / 预设）存在 localStorage；读取时逐字段校验，
 * 脏数据一律回落默认值，避免坏数据把页面顶掉。
 */
import { gradientFormatAdapters } from './adapters'
import { normalizeHexColor } from './color'
import { DEFAULT_GRADIENT_COLORS, DEFAULT_GRADIENT_DOCUMENT, cloneGradientDocument, normalizeGradientDocument } from './document'
import type { GradientFormatId, GradientTextDocument } from './types'

export type GradientPreset = {
  id: string
  name: string
  colors: string[]
  createdAt: string
}

export type GradientTextState = {
  version: 1
  document: GradientTextDocument
  colors: string[]
  adapterId: GradientFormatId
  vanillaCharacter: '&' | '§'
  simplifyGradients: boolean
  presets: GradientPreset[]
}

type StorageLike = Pick<Storage, 'getItem' | 'setItem'>

export const GRADIENT_TEXT_STORAGE_KEY = 'molaunch.lab.gradient-text.v1'

const PRESET_NAME_LIMIT = 80

export function createDefaultGradientTextState(): GradientTextState {
  return {
    version: 1,
    document: cloneGradientDocument(DEFAULT_GRADIENT_DOCUMENT),
    colors: [...DEFAULT_GRADIENT_COLORS],
    adapterId: 'vanilla',
    vanillaCharacter: '§',
    simplifyGradients: false,
    presets: [],
  }
}

function browserStorage(): StorageLike | null {
  return typeof window === 'undefined' ? null : window.localStorage
}

function sanitizeColors(value: unknown): string[] {
  if (!Array.isArray(value)) return []
  return value
    .map((color) => (typeof color === 'string' ? normalizeHexColor(color) : null))
    .filter((color): color is string => color !== null)
}

export function parseGradientPresets(value: unknown): GradientPreset[] {
  if (!Array.isArray(value)) return []
  const presets: GradientPreset[] = []
  value.forEach((entry, index) => {
    if (!entry || typeof entry !== 'object') return
    const record = entry as Partial<GradientPreset>
    const colors = sanitizeColors(record.colors)
    if (colors.length === 0) return
    const name = typeof record.name === 'string' ? record.name.trim() : ''
    presets.push({
      id: typeof record.id === 'string' && record.id ? record.id : `imported-${index}-${Date.now()}`,
      name: name ? name.slice(0, PRESET_NAME_LIMIT) : `Preset ${index + 1}`,
      colors,
      createdAt:
        typeof record.createdAt === 'string' && !Number.isNaN(Date.parse(record.createdAt))
          ? record.createdAt
          : new Date().toISOString(),
    })
  })
  return presets
}

/** 预设导出：只保留可迁移字段 */
export function serializeGradientPresets(presets: GradientPreset[]): string {
  return JSON.stringify(
    presets.map((preset) => ({ name: preset.name, colors: preset.colors })),
    null,
    2,
  )
}

function sanitizeState(value: unknown, fallback: GradientTextState): GradientTextState {
  if (!value || typeof value !== 'object') return fallback
  const record = value as Partial<GradientTextState>
  const knownAdapter = gradientFormatAdapters.some((adapter) => adapter.id === record.adapterId)
  const colors = sanitizeColors(record.colors)
  return {
    version: 1,
    document: record.document ? normalizeGradientDocument(record.document) : fallback.document,
    colors: colors.length > 0 ? colors : fallback.colors,
    adapterId: knownAdapter ? (record.adapterId as GradientFormatId) : fallback.adapterId,
    vanillaCharacter:
      record.vanillaCharacter === '&' || record.vanillaCharacter === '§'
        ? record.vanillaCharacter
        : fallback.vanillaCharacter,
    simplifyGradients: record.simplifyGradients === true,
    presets: parseGradientPresets(record.presets),
  }
}

export function loadGradientTextState(storage: StorageLike | null = browserStorage()): GradientTextState {
  const fallback = createDefaultGradientTextState()
  if (!storage) return fallback
  try {
    const raw = storage.getItem(GRADIENT_TEXT_STORAGE_KEY)
    if (!raw) return fallback
    return sanitizeState(JSON.parse(raw), fallback)
  } catch {
    return fallback
  }
}

export function saveGradientTextState(
  state: GradientTextState,
  storage: StorageLike | null = browserStorage(),
): void {
  if (!storage) return
  storage.setItem(GRADIENT_TEXT_STORAGE_KEY, JSON.stringify(sanitizeState(state, createDefaultGradientTextState())))
}
