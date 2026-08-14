/**
 * 渐变文字工具 - 本地持久化
 *
 * 状态（文档 / 颜色 / 输出配置 / 预设）存取于 localStorage，
 * 读取时全量校验兜底，避免脏数据影响页面。
 */
import {
  cloneGradientDocument,
  DEFAULT_GRADIENT_COLORS,
  DEFAULT_GRADIENT_DOCUMENT,
  normalizeGradientDocument,
  normalizeHexColor,
} from './core'
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
  vanillaCharacter: '&' | '§'
  simplifyGradients: boolean
  presets: GradientPreset[]
}

type StorageLike = Pick<Storage, 'getItem' | 'setItem'>

export const GRADIENT_TEXT_STORAGE_KEY = 'molaunch.lab.gradient-text.v1'

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

export function loadGradientTextState(
  storage: StorageLike | null = getBrowserStorage(),
): GradientTextState {
  const fallback = createDefaultGradientTextState()
  if (!storage) return fallback
  try {
    const raw = storage.getItem(GRADIENT_TEXT_STORAGE_KEY)
    if (!raw) return fallback
    return sanitizeGradientTextState(JSON.parse(raw), fallback)
  } catch {
    return fallback
  }
}

export function saveGradientTextState(
  state: GradientTextState,
  storage: StorageLike | null = getBrowserStorage(),
): void {
  if (!storage) return
  storage.setItem(GRADIENT_TEXT_STORAGE_KEY, JSON.stringify(sanitizeGradientTextState(state)))
}

export function parseGradientPresets(value: unknown): GradientPreset[] {
  if (!Array.isArray(value)) return []
  return value.flatMap((preset, index) => {
    if (!preset || typeof preset !== 'object') return []
    const record = preset as Partial<GradientPreset>
    const colors = sanitizeColors(record.colors)
    if (!colors.length) return []
    return [
      {
        id:
          typeof record.id === 'string' && record.id
            ? record.id
            : `imported-${index}-${Date.now()}`,
        name:
          typeof record.name === 'string' && record.name.trim()
            ? record.name.trim().slice(0, 80)
            : `Preset ${index + 1}`,
        colors,
        createdAt:
          typeof record.createdAt === 'string' && !Number.isNaN(Date.parse(record.createdAt))
            ? record.createdAt
            : new Date().toISOString(),
      },
    ]
  })
}

export function serializeGradientPresets(presets: GradientPreset[]): string {
  return JSON.stringify(
    presets.map(({ name, colors }) => ({ name, colors })),
    null,
    2,
  )
}

function sanitizeGradientTextState(
  value: unknown,
  fallback: GradientTextState = createDefaultGradientTextState(),
): GradientTextState {
  if (!value || typeof value !== 'object') return fallback
  const record = value as Partial<GradientTextState>
  const adapterId = gradientFormatAdapters.some((adapter) => adapter.id === record.adapterId)
    ? (record.adapterId as GradientFormatId)
    : fallback.adapterId
  const colors = sanitizeColors(record.colors)
  return {
    version: 1,
    document: record.document ? normalizeGradientDocument(record.document) : fallback.document,
    colors: colors.length ? colors : fallback.colors,
    adapterId,
    vanillaCharacter:
      record.vanillaCharacter === '&' || record.vanillaCharacter === '§'
        ? record.vanillaCharacter
        : fallback.vanillaCharacter,
    simplifyGradients: record.simplifyGradients === true,
    presets: parseGradientPresets(record.presets),
  }
}

function sanitizeColors(value: unknown): string[] {
  if (!Array.isArray(value)) return []
  return value
    .map((color) => (typeof color === 'string' ? normalizeHexColor(color) : null))
    .filter((color): color is string => color !== null)
}

function getBrowserStorage(): StorageLike | null {
  return typeof window === 'undefined' ? null : window.localStorage
}
