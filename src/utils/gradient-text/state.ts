/**
 * 渐变文字工具 - 本地状态与预设（对应 specs/cleanroom/gradient-text.md §8）
 *
 * 状态存在 localStorage；读取时逐字段校验，脏数据回落默认值。
 */
import { normalizeHexColor } from './colors'
import { DEFAULT_COLORS, DEFAULT_DOCUMENT, cloneDocument, normalizeDocument } from './document'
import { gradientFormatAdapters } from './formats'
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
  control: '&' | '§'
  simplifyGradient: boolean
  presets: GradientPreset[]
}

type StorageLike = Pick<Storage, 'getItem' | 'setItem'>

export const GRADIENT_TEXT_STORAGE_KEY = 'molaunch.lab.gradient-text.v1'

const NAME_LIMIT = 80

export function createDefaultState(): GradientTextState {
  return {
    version: 1,
    document: cloneDocument(DEFAULT_DOCUMENT),
    colors: [...DEFAULT_COLORS],
    adapterId: 'vanilla',
    control: '§',
    simplifyGradient: false,
    presets: [],
  }
}

function storageOrNull(): StorageLike | null {
  return typeof window === 'undefined' ? null : window.localStorage
}

function sanitizeColors(value: unknown): string[] {
  if (!Array.isArray(value)) return []
  return value
    .map((color) => (typeof color === 'string' ? normalizeHexColor(color) : null))
    .filter((color): color is string => color !== null)
}

/** 预设导入：只接受带有效色标的条目 */
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
      name: name ? name.slice(0, NAME_LIMIT) : `Preset ${index + 1}`,
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
  return JSON.stringify(presets.map((preset) => ({ name: preset.name, colors: preset.colors })), null, 2)
}

function sanitizeState(value: unknown, fallback: GradientTextState): GradientTextState {
  if (!value || typeof value !== 'object') return fallback
  const record = value as Partial<GradientTextState> & { vanillaCharacter?: '&' | '§'; simplifyGradients?: boolean }
  const colors = sanitizeColors(record.colors)
  const knownAdapter = gradientFormatAdapters.some((adapter) => adapter.id === record.adapterId)
  // 兼容旧字段名（control ← vanillaCharacter、simplifyGradient ← simplifyGradients）
  const control = record.control ?? record.vanillaCharacter
  const simplify = record.simplifyGradient ?? record.simplifyGradients
  return {
    version: 1,
    document: record.document ? normalizeDocument(record.document) : fallback.document,
    colors: colors.length > 0 ? colors : fallback.colors,
    adapterId: knownAdapter ? (record.adapterId as GradientFormatId) : fallback.adapterId,
    control: control === '&' || control === '§' ? control : fallback.control,
    simplifyGradient: simplify === true,
    presets: parseGradientPresets(record.presets),
  }
}

export function loadGradientTextState(storage: StorageLike | null = storageOrNull()): GradientTextState {
  const fallback = createDefaultState()
  if (!storage) return fallback
  try {
    const raw = storage.getItem(GRADIENT_TEXT_STORAGE_KEY)
    if (!raw) return fallback
    return sanitizeState(JSON.parse(raw), fallback)
  } catch {
    return fallback
  }
}

export function saveGradientTextState(state: GradientTextState, storage: StorageLike | null = storageOrNull()): void {
  if (!storage) return
  storage.setItem(GRADIENT_TEXT_STORAGE_KEY, JSON.stringify(sanitizeState(state, createDefaultState())))
}
